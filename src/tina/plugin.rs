use super::event::Event;
use crate::Config;
use crate::Logger;
use crate::lua::LuaFunctionRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str; //plugin name

    async fn boot<'lua>(
        &self,
        config: Arc<Config>,
        logger: Arc<Logger>,
        event_tx: mpsc::Sender<Event>,
        lua_registry: &mut LuaFunctionRegistry<'lua>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    // start the plugin
    async fn run(&self);

    // gracefully shutdown the plugin
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}
