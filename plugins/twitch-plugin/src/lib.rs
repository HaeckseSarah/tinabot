mod auth;
mod keychain;
mod token;

use std::sync::Arc;

use mlua::LuaSerdeExt;
use tina_plugin_api::{Plugin, PluginContext, ScriptRegistry};

use async_trait::async_trait;
use tokio::sync::OnceCell;

use crate::token::Token;
use std::pin::Pin;
use tokio_util::sync::CancellationToken;
use twitch_api::{HelixClient, twitch_oauth2::ClientId};

// set client id at compile time!
const DEFAULT_CLIENT_ID: Option<&str> = option_env!("CLI_CLIENT_ID");

pub struct TwitchPlugin {
    context: OnceCell<PluginContext>, // Nur noch eine Cell für den gesamten Kontext!
    cancel_token: CancellationToken,
    broadcaster_token: OnceCell<Arc<Token>>,
    bot_token: OnceCell<Arc<Token>>,
    broadcaster_id: OnceCell<String>,
    bot_user_id: OnceCell<String>,
    helix_client: Arc<HelixClient<'static, reqwest::Client>>,
}

impl TwitchPlugin {
    /// Creates a new uninitialized instance
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            context: OnceCell::new(),
            cancel_token,
            broadcaster_token: OnceCell::new(),
            bot_token: OnceCell::new(),
            broadcaster_id: OnceCell::new(),
            bot_user_id: OnceCell::new(),
            helix_client: Arc::new(HelixClient::new()),
        }
    }

    fn ctx(&self) -> &PluginContext {
        self.context.get().expect("Plugin not booted!")
    }

    fn broadcaster_token(&self) -> &Token {
        self.broadcaster_token.get().expect("Plugin not booted!")
    }

    fn bot_token(&self) -> &Token {
        self.bot_token.get().expect("Plugin not booted!")
    }

    fn broadcaster_id(&self) -> &String {
        self.broadcaster_id.get().expect("Plugin not booted!")
    }

    fn bot_user_id(&self) -> &String {
        self.bot_user_id.get().expect("Plugin not booted!")
    }

    fn helix_client(&self) -> &HelixClient<'static, reqwest::Client> {
        &self.helix_client
    }

    fn client_id(&self) -> ClientId {
        let id = self
            .ctx()
            .config_get("client_id")
            .or(DEFAULT_CLIENT_ID.map(|s| s.to_string()))
            .expect("Twitch Client_id not found. Please set in config");
        ClientId::new(id)
    }

    fn get_token<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Token> + Send + 'a>> {
        Box::pin(async move {
            match Token::from_user(username, self.client_id()).await {
                Some(user_token) => user_token,
                None => {
                    println!("User {} not authenticated. Please authenticate", username);
                    auth::run(self.client_id()).await.unwrap();
                    self.get_token(username).await
                }
            }
        })
    }

    async fn register_script_send_message(&self, script_registry: &mut ScriptRegistry<'_>) {
        let broadcaster_id = self.broadcaster_id().clone();
        let bot_user_id = self.bot_user_id().clone();
        let bot_token = self.bot_token().clone();
        let helix_client = self.helix_client().clone();

        script_registry.register_function("send_message", move |lua, msg: String| {
            let broadcaster_id = broadcaster_id.clone();
            let bot_user_id = bot_user_id.clone();
            let bot_token = bot_token.clone();
            let helix_client = helix_client.clone();

            let response = tokio::task::block_in_place(move || {
                futures::executor::block_on(async move {
                    let token = bot_token.get_token().await;
                    helix_client
                        .send_chat_message(broadcaster_id, bot_user_id, msg.as_str(), &token)
                        .await
                        .unwrap()
                })
            });

            let lua_value = lua.to_value(&response)?;
            Ok(Some(lua_value))
        });
    }

    async fn register_script_get_channel_information(
        &self,
        script_registry: &mut ScriptRegistry<'_>,
    ) {
        let broadcaster_token = self.broadcaster_token().clone();
        let helix_client = self.helix_client().clone();

        script_registry.register_function("get_channel_information", move |lua, id: String| {
            let broadcaster_token = broadcaster_token.clone();
            let helix_client = helix_client.clone();

            let channel_data = tokio::task::block_in_place(move || {
                futures::executor::block_on(async move {
                    let token = broadcaster_token.get_token().await;
                    helix_client
                        .get_channel_from_login(id.as_str(), &token)
                        .await
                        .unwrap()
                })
            });

            if let Some(info) = channel_data {
                let lua_value = lua.to_value(&info)?;
                Ok(Some(lua_value))
            } else {
                Ok(None)
            }
        });
    }

    async fn register_scripts(&self, script_registry: &mut ScriptRegistry<'_>) {
        self.register_script_send_message(script_registry).await;
        self.register_script_get_channel_information(script_registry)
            .await;
    }

    /*

    */
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
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .context
            .set(context)
            .map_err(|_| "Context already set!")?;

        let broadcaster_token = match self.ctx().config_get("broadcaster_user") {
            Some(value) => {
                let broadcaster_token = self.get_token(&value).await;
                Arc::new(broadcaster_token)
            }
            None => {
                return Err("Please configure a twitch broadcast user"
                    .to_string()
                    .into());
            }
        };

        self.broadcaster_token
            .set(broadcaster_token.clone())
            .map_err(|_| "Broadcaster Token already set!")?;

        self.broadcaster_id
            .set(broadcaster_token.get_token().await.user_id.to_string())?;

        let bot_token = match self.ctx().config_get("bot_user") {
            Some(value) => {
                let bot_token = self.get_token(&value).await;
                Arc::new(bot_token)
            }
            None => broadcaster_token.clone(),
        };

        self.bot_token
            .set(bot_token.clone())
            .map_err(|_| "Bot Token already set!")?;

        self.bot_user_id
            .set(bot_token.get_token().await.user_id.to_string())?;

        self.register_scripts(script_registry).await;

        self.ctx().log_info("booted");

        Ok(())
    }

    /// starts main loop
    async fn run(&self, _command: Option<String>) {
        /*
        if let Some(cmd) = command {
            match cmd.as_str() {
                //"auth" => {
                //self.run_auth().await.unwrap();
                //}
                "test" => {
                    self.run_test().await.unwrap();
                }
                _ => {
                    self.ctx().log_error(&format!("unknown command: {}", cmd));
                }
            }
            return;
        }
        */

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
