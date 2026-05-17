use crate::event::Event;
use crate::scripting::ScriptRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

// Callback functions provided by the kernel
pub type ConfigLookup = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;
pub type LogFn = Arc<dyn Fn(&i32, &str, &str) + Send + Sync>; // (Level, Target, Message)
pub type EventTx = Sender<Event>;

/// The basic trait that all native plugins must implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the unique identifier of the plugin
    fn id(&self) -> &str;

    /// invoked by the kernel during the boot phase
    /// initialize plugin and register script functions
    async fn boot<'lua>(
        &self,
        config_get: ConfigLookup,
        log: LogFn,
        event_tx: EventTx,
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Starts the main loop of the plugin
    async fn run(&self);

    /// gracefully shutdown plugin.
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}
