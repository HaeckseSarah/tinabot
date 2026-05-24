use tina_plugin_api::{Plugin, PluginContext, ScriptRegistry};

use async_trait::async_trait;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

pub struct TwitchPlugin {
    context: OnceCell<PluginContext>, // Nur noch eine Cell für den gesamten Kontext!
    cancel_token: CancellationToken,
}

impl TwitchPlugin {
    pub fn new() -> Self {
        Self {
            context: OnceCell::new(),
            cancel_token: CancellationToken::new(),
        }
    }
    fn ctx(&self) -> &PluginContext {
        self.context.get().expect("Plugin not booted!")
    }
}

#[async_trait]
impl Plugin for TwitchPlugin {
    fn id(&self) -> &str {
        "twitch"
    }

    async fn boot<'lua>(
        &self,
        context: PluginContext,
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .context
            .set(context)
            .map_err(|_| "Context already set!")?;

        Ok(())
    }

    async fn run(&self) {
        self.ctx().log_info("Twitch plugin started.");

        loop {
            tokio::select! {
                // token to exit main loop
                _ = self.cancel_token.cancelled() => {
                    self.ctx().log_info("initiatiing shutdown...");
                    break;
                }
            }
        }
    }

    /// cancel main loop and gracefully shutdown this plugin    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ctx().log_info("shutdown");
        self.cancel_token.cancel();
        Ok(())
    }
}
