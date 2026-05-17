use crate::PluginContext;
use crate::event::Event;
use crate::scripting::ScriptRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

// Callback functions provided by the kernel
//pub type ConfigLookup = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;
pub type LogFn = Arc<dyn Fn(&i32, &str, &str) + Send + Sync>; // (Level, Target, Message)
pub type EventTx = Sender<Event>;

/// A namespaced configuration proxy
#[derive(Clone)]
pub struct PluginConfig {
    plugin_id: String,
    global_lookup: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
}

impl PluginConfig {
    pub fn new(
        plugin_id: &str,
        global_lookup: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.to_uppercase(),
            global_lookup,
        }
    }

    /// Requesting "VAR" searches for "TINA_{PLUGIN_ID}_VAR".
    pub fn get(&self, key: &str) -> Option<String> {
        // check env
        let env_key = format!(
            "TINA_{}_{}",
            self.plugin_id.to_uppercase(),
            key.to_uppercase()
        );

        if let Ok(env_value) = std::env::var(&env_key) {
            return Some(env_value);
        }

        // check config
        let toml_key = format!(
            "plugin.{}.{}",
            self.plugin_id.to_lowercase(),
            key.to_lowercase()
        );

        if let Some(value) = (self.global_lookup)(&toml_key) {
            return Some(value);
        }

        None
    }
}

/// The basic trait that all native plugins must implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the unique identifier of the plugin
    fn id(&self) -> &str;

    /// invoked by the kernel during the boot phase
    /// initialize plugin and register script functions
    async fn boot<'lua>(
        &self,
        context: PluginContext,
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Starts the main loop of the plugin
    async fn run(&self);

    /// gracefully shutdown plugin.
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}
