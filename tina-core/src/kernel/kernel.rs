use crate::Config;
use crate::logger::{LogLevel, Logger};
use crate::lua::LuaWrapper;
use dummy_plugin::DummyPlugin;
use std::sync::Arc;
use tina_plugin_api::{Event, LogFn, Plugin, PluginConfig};
use tokio::sync::{Mutex, OnceCell, RwLock, mpsc};

pub struct Kernel {
    config: Arc<Config>,
    logger: Arc<Logger>,
    event_rx: Mutex<Option<mpsc::Receiver<Event>>>,
    event_tx: OnceCell<mpsc::Sender<Event>>,
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
    lua: LuaWrapper,
}

impl Kernel {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        Self {
            config: config.clone(),
            logger: logger.clone(),
            event_rx: Mutex::new(None),
            event_tx: OnceCell::new(),
            plugins: RwLock::new(Vec::new()),
            lua: LuaWrapper::new(config.clone(), logger.clone()),
        }
    }

    fn get_event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Event channel not initialized!")
    }

    fn create_plugin_instance(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        match name.to_lowercase().trim() {
            "dummy" => Some(Arc::new(DummyPlugin::new())),
            // "twitch" => Some(Arc::new(TwitchPlugin::new())),
            // "obs" => Some(Arc::new(ObsPlugin::new())),
            _ => {
                self.logger.log(
                    LogLevel::Warn,
                    "Kernel",
                    &format!("Unknown plugin in configuration skipped: {}", name),
                );
                None
            }
        }
    }

    async fn boot_plugin(
        &self,
        plugin: Arc<dyn Plugin>,
        config_lookup: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
        log_fn: LogFn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Setup the isolated mLua registry proxy
        let mut script_registry = self.lua.create_registry(plugin.id());

        // 2. Wrap the configuration into the secure namespace container
        let plugin_config = PluginConfig::new(plugin.id(), config_lookup);

        // 3. Boot the plugin instance safely
        plugin
            .boot(
                plugin_config,
                log_fn,
                self.get_event_tx().clone(),
                &mut script_registry,
            )
            .await?;

        // 4. Commit the newly registered script bindings back into the Lua runtime
        self.lua.register_functions(script_registry).await?;

        Ok(())
    }

    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger
            .log(LogLevel::Info, "Kernel", "Initializing Kernel...");

        // initialize event channels
        let (event_tx, event_rx) = mpsc::channel::<Event>(100);
        self.event_tx
            .set(event_tx)
            .map_err(|_| "Event channel already set!")?;

        let mut rx_lock = self.event_rx.lock().await;
        *rx_lock = Some(event_rx);

        // keep track of initialized plugins
        let mut booted_plugins: Vec<Arc<dyn Plugin>> = Vec::new();

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
        if let Some(enabled_plugins_list) = self.config.get("TINA_ENABLED_PLUGINS") {
            for plugin_name in enabled_plugins_list.split(',') {
                if let Some(plugin_instance) = self.create_plugin_instance(plugin_name) {
                    self.boot_plugin(
                        plugin_instance.clone(),
                        config_lookup.clone(),
                        log_fn.clone(),
                    )
                    .await?;

                    booted_plugins.push(plugin_instance);
                }
            }
        }

        let mut plugins_write = self.plugins.write().await;
        *plugins_write = booted_plugins;

        self.lua.load_scripts()?;
        Ok(())
    }

    pub async fn run(&self) {
        let mut rx_opt = self.event_rx.lock().await;
        let mut event_rx = rx_opt
            .take()
            .expect("Kernel-Receiver already started or not initialized!");

        // run plugins
        let plugins_read = self.plugins.read().await;
        for plugin in plugins_read.iter() {
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

        self.logger
            .log(LogLevel::Info, "Kernel", "Running Kernel...");

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    match event.event_type.as_str() {
                        "killSignal" => {
                            // TODO check permissions
                            self.logger.log(
                                LogLevel::Debug,
                                "Kernel",
                                &format!("Received killSignal from: {}", event.source),
                            );
                            break;
                        }
                        _ => {
                            self.logger.log(
                                LogLevel::Debug,
                                "Kernel",
                                &format!("Received event: {:?}", event),
                            );
                            let _ = self.lua.test_lua();
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    self.logger.log(LogLevel::Info, "Kernel", "ctrl+c detected. Shuting down...");
                    break; // exit main loop
                }

                else => break,
            }
        }

        if let Err(e) = self.shutdown().await {
            self.logger.log(
                LogLevel::Error,
                "Kernel",
                &format!("Shutdown error: {:?}", e),
            );
        }
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger
            .log(LogLevel::Info, "Kernel", "stopping all plugins...");

        let plugins_read = self.plugins.read().await;

        for plugin in plugins_read.iter() {
            self.logger.log(
                LogLevel::Info,
                "Kernel",
                &format!("Call shutdown() on plugin: {}", plugin.id()),
            );

            if let Err(e) = plugin.shutdown().await {
                self.logger.log(
                    LogLevel::Error,
                    "Kernel",
                    &format!("Plugin '{}' error on shutdown: {:?}", plugin.id(), e),
                );
            }
        }

        self.logger.log(LogLevel::Info, "Kernel", "Bye Bye ");
        Ok(())
    }
}
