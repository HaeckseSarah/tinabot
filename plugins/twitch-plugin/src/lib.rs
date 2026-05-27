use tina_plugin_api::{EventValue, Plugin, PluginContext, ScriptRegistry};

use async_trait::async_trait;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

pub struct TwitchPlugin {
    context: OnceCell<PluginContext>, // Nur noch eine Cell für den gesamten Kontext!
    cancel_token: CancellationToken,
}

impl TwitchPlugin {
    /// Creates a new uninitialized instance
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            context: OnceCell::new(),
            cancel_token,
        }
    }

    /// Internal helper to retrieve the plugin context.   
    fn ctx(&self) -> &PluginContext {
        self.context.get().expect("Plugin not booted!")
    }
}

#[async_trait]
impl Plugin for TwitchPlugin {
    /// Returns the unique string identity used to reference this specific plugin    
    fn id(&self) -> &str {
        "twitch"
    }

    /// Initializes the plugin.
    async fn boot<'lua>(
        &self,
        context: PluginContext,
        _script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .context
            .set(context)
            .map_err(|_| "Context already set!")?;
        self.ctx().log_info("booted");
        Ok(())
    }
    /// starts main loop
    async fn run(&self, command: Option<String>) {
        if let Some(cmd) = command {
            match cmd.as_str() {
                "auth" => {
                    //todo;
                }
                _ => {
                    self.ctx().log_error(&format!("unknown command: {}", cmd));
                }
            }
            return;
        }

        self.ctx().log_info("Twitch plugin started.");

        loop {
            tokio::select! {
                // token to exit main loop
                _ = self.cancel_token.cancelled() => {
                    self.ctx().log_info("initiating shutdown...");
                    self.shutdown().await.unwrap();
                    break;
                }
            }
        }
    }

    /// cancel main loop and gracefully shutdown this plugin    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
