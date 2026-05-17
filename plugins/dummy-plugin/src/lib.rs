use tina_plugin_api::{EventValue, Plugin, PluginContext, ScriptRegistry};

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

/// A plugin used for testing
pub struct DummyPlugin {
    context: OnceCell<PluginContext>, // Nur noch eine Cell für den gesamten Kontext!
    cancel_token: CancellationToken,
}

impl DummyPlugin {
    /// Creates a new uninitialized instance
    pub fn new() -> Self {
        Self {
            context: OnceCell::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Internal helper to retrieve the plugin context.   
    fn ctx(&self) -> &PluginContext {
        self.context.get().expect("Plugin not booted!")
    }

    /// Registers all API bindings this plugin wants to expose to the Lua runtime environment.
    fn register_scripts(&self, script_registry: &mut ScriptRegistry<'_>) {
        self.register_script_ping(script_registry);
    }

    /// Binds the `ping` function to Lua,
    /// letting scripts print messages to standard output.
    /// todo: put into external file
    fn register_script_ping(&self, script_registry: &mut ScriptRegistry<'_>) {
        script_registry.register_function("ping", |_lua, msg: String| {
            println!("Ping: {}", msg.as_str());
            Ok(())
        });
    }

    /// Routes evaluated stdin strings to their respective internal plugin actions.    
    async fn handle_input(&self, input: &str) {
        match input {
            "exit" => {
                self.ctx().send_event("shutdown", None).await;
            }
            _ => {
                let mut payload = HashMap::new();
                payload.insert("message".to_string(), EventValue::String(input.to_string()));
                self.ctx().send_event("message", Some(payload)).await;
            }
        }
    }
}

#[async_trait]
impl Plugin for DummyPlugin {
    /// Returns the unique string identity used to reference this specific plugin    
    fn id(&self) -> &str {
        "dummy"
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

        self.register_scripts(script_registry);

        Ok(())
    }
    /// starts main loop
    async fn run(&self) {
        self.ctx().log_info("Dummy plugin started.");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        loop {
            println!("tina>");
            input.clear();

            tokio::select! {
                // token to exit main loop
                _ = self.cancel_token.cancelled() => {
                    self.ctx().log_info("initiatiing shutdown...");
                    break;
                }

                result = reader.read_line(&mut input) => {
                    match result {
                        Ok(0) => {
                            break;
                        }

                        Ok(_) => {
                            let trimmed_input = input.trim();
                            if trimmed_input.is_empty() { continue; }
                            self.handle_input(trimmed_input).await;
                        }

                        Err(e) => {
                            self.ctx().log_error(&format!("error on reading from stdin: {}", e));
                            break;
                        }
                    }
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
