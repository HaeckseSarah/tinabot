use super::BaseRegistry;
use super::dispatcher::Dispatcher;
use super::dispatcher::QueueConfig;
use crate::Config;
use crate::kernel::plugin_helper::PluginHelper;
use crate::logger::{LogLevel, Logger};
use crate::lua::Wrapper as LuaWrapper;
use tina_plugin_api::{LogFn, Plugin, PluginConfig, PluginContext};

use std::process::exit;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct Kernel {
    config: Arc<Config>,
    logger: Arc<Logger>,
    plugins: Arc<BaseRegistry<dyn Plugin>>,
    lua_wrapper: Arc<LuaWrapper>,
    event_dispatcher: OnceCell<Arc<Dispatcher>>,
}

impl Kernel {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        Self {
            config: config.clone(),
            logger: logger.clone(),
            plugins: Arc::new(BaseRegistry::new()),
            lua_wrapper: Arc::new(LuaWrapper::new(config.clone(), logger.clone())),
            event_dispatcher: OnceCell::new(),
        }
    }

    fn event_dispatcher(&self) -> Arc<Dispatcher> {
        self.event_dispatcher
            .get()
            .expect("dispatcher not found!")
            .clone()
    }

    /// fixme: clean up!
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // dispatcher
        let dispatcher = Dispatcher::new(self.lua_wrapper.clone(), self.logger.clone());

        let _ = self.event_dispatcher.set(dispatcher);

        // load plugins
        // create config wrapper function for plugins
        let config_clone = self.config.clone();
        let config_lookup = Arc::new(move |key: &str| {
            config_clone.get(key) // Gibt Option<String>
        });

        // create log wrapper function for plugins
        let logger_clone = self.logger.clone();
        let log_fn: LogFn = Arc::new(move |level, target, message| {
            let log_level = LogLevel::try_from(*level).unwrap_or(LogLevel::Error);
            logger_clone.log(log_level, target, message);
        });

        // check config for enabled plugins and load
        let enabled_plugins = PluginHelper::get_enabled_plugins(self.config.clone());

        for plugin_name in enabled_plugins {
            let dispatcher = self.event_dispatcher().clone();
            let lua_wrapper = dispatcher.get_lua_wrapper();

            if let Some(plugin_instance) = PluginHelper::create_plugin_instance(&plugin_name) {
                let mut script_registry = lua_wrapper.create_registry(plugin_instance.id());

                let plugin_config = PluginConfig::new(plugin_instance.id(), config_lookup.clone());
                let plugin_context = PluginContext::new(
                    plugin_instance.id(),
                    plugin_config,
                    self.event_dispatcher().get_event_tx().clone(),
                    log_fn.clone(),
                    self.event_dispatcher().get_filter_registry().clone(),
                );

                plugin_instance
                    .boot(plugin_context, &mut script_registry)
                    .await?;

                // register lua functions
                self.event_dispatcher()
                    .get_lua_wrapper()
                    .register_functions(script_registry, Some("p"))
                    .await?;

                self.plugins
                    .add(plugin_instance.id().to_string(), plugin_instance);
            }
        }

        // queues
        self.event_dispatcher()
            .add_queue("default".to_string(), QueueConfig { parallel: true });
        let queues = self.config.clone().get_table::<QueueConfig>("queue");
        for (name, conf) in queues {
            self.event_dispatcher().add_queue(name, conf);
        }

        // load lua scripts
        self.event_dispatcher()
            .get_lua_wrapper()
            .load_scripts()
            .await?;

        Ok(())
    }

    pub async fn run(&self) {
        // run plugins
        let plugins_read = self.plugins.keys_values();

        for (_, plugin) in plugins_read.iter() {
            let plugin_clone = plugin.clone();
            self.logger.log(
                LogLevel::Info,
                "Kernel",
                &format!("Start Plugin-Task: {}", plugin_clone.id()),
            );

            tokio::spawn(async move {
                plugin_clone.run().await;
            });
        }

        self.event_dispatcher().run().await;

        if let Err(e) = self.shutdown().await {
            self.logger.log(
                LogLevel::Error,
                "Kernel",
                &format!("Shutdown error: {:?}", e),
            );
        }
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
