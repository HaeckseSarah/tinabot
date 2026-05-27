mod keychain;

use tina_plugin_api::{EventValue, Plugin, PluginContext, ScriptRegistry};

use async_trait::async_trait;
//use reqwest::Client;
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;
use twitch_oauth2::client::Client;
use twitch_oauth2::tokens::UserToken;
use twitch_oauth2::{ClientId, DeviceUserTokenBuilder, Scope};

// set client id at compile time!
const DEFAULT_CLIENT_ID: Option<&str> = option_env!("CLI_CLIENT_ID");

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

    async fn run_auth(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let client_id = self
            .ctx()
            .config_get("client_id")
            .or(DEFAULT_CLIENT_ID.map(|s| s.to_string()))
            .expect("Client_ID not found. Please set in config");

        let mut builder =
            DeviceUserTokenBuilder::new(client_id, vec![Scope::ChatRead, Scope::ChatEdit]);

        let code = builder.start(&client).await?;

        println!("Please go to {}", code.verification_uri);

        let token = builder.wait_for_code(&client, tokio::time::sleep).await?;
        println!("Token: {:?}", token);

        Ok(())
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
                    self.run_auth().await.unwrap();
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
