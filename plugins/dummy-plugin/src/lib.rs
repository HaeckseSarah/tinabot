use tina_plugin_api::{ConfigLookup, Event, EventTx, EventValue, LogFn, Plugin, ScriptRegistry};

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{OnceCell, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// A plugin used for testing
pub struct DummyPlugin {
    config_get: OnceCell<ConfigLookup>,
    log: OnceCell<LogFn>,
    event_tx: OnceCell<EventTx>,
    cancel_token: CancellationToken,
}

impl DummyPlugin {
    /// Creates a new uninitialized instance
    pub fn new() -> Self {
        Self {
            config_get: OnceCell::new(),
            log: OnceCell::new(),
            event_tx: OnceCell::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Internal helper to retrieve the event sender channel.   
    fn event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Plugin not initialized!")
    }

    // logging helper functions
    fn log(&self, level: &i32, msg: &str) {
        let log = self.log.get().expect("Plugin not booted!");
        log(level, self.id(), msg);
    }
    fn log_error(&self, msg: &str) {
        self.log(&3, msg);
    }
    fn log_info(&self, msg: &str) {
        self.log(&1, msg);
    }
    fn log_debug(&self, msg: &str) {
        self.log(&0, msg);
    }

    /// Asynchronously send an event to the core.
    async fn send_event(&self, event_type: &str, payload: Option<HashMap<String, EventValue>>) {
        let event = Event {
            id: Uuid::new_v4(),
            source: self.id().to_string(),
            event_type: event_type.to_string(),
            payload: payload,
        };

        let _ = self.event_tx().send(event).await;
    }

    /// Helper function for sending an message-event
    async fn send_message(&self, message: String) {
        let mut payload = HashMap::new();
        payload.insert("message".to_string(), EventValue::String(message));
        self.send_event("message", Some(payload)).await;
    }

    /// Registers all API bindings this plugin wants to expose to the Lua runtime environment.
    fn register_scripts(&self, script_registry: &mut ScriptRegistry<'_>) {
        self.register_script_ping(script_registry);
    }

    /// Binds the `ping` function to Lua,
    /// letting scripts print messages to standard output.
    fn register_script_ping(&self, script_registry: &mut ScriptRegistry<'_>) {
        script_registry.register_function("ping", |_lua, msg: String| {
            println!("{}", msg.as_str());
            Ok(())
        });
    }

    /// Routes evaluated stdin strings to their respective internal plugin actions.    
    async fn handle_input(&self, input: &str) {
        match input {
            "exit" => {
                self.send_event("killSignal", None).await;
            }
            _ => {
                self.send_message(input.to_string()).await;
            }
        }
    }
}

#[async_trait]
impl Plugin for DummyPlugin {
    /// Returns the unique string identity used to reference this specific plugin    
    fn id(&self) -> &str {
        "dummyPlugin"
    }

    /// Initializes the plugin.
    async fn boot<'lua>(
        &self,
        config_get: ConfigLookup,
        log: LogFn,
        event_tx: EventTx,
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self
            .config_get
            .set(config_get)
            .map_err(|_| "config already set!")?;
        let _ = self.log.set(log).map_err(|_| "Logger already set!")?;
        let _ = self
            .event_tx
            .set(event_tx)
            .map_err(|_| "Event-Channel already set!")?;

        self.register_scripts(script_registry);

        Ok(())
    }
    /// starts main loop
    async fn run(&self) {
        self.log_info("Dummy plugin started.");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        loop {
            println!("tina>");
            input.clear();

            tokio::select! {
                // token to exit main loop
                _ = self.cancel_token.cancelled() => {
                    self.log_info("initiatiing shutdown...");
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
                            self.log_error(&format!("error on reading from stdin: {}", e));
                            break;
                        }
                    }
                }
            }
        }
    }

    /// cancel main loop and gracefully shutdown this plugin    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.log_info("shutdown");
        self.cancel_token.cancel();

        Ok(())
    }
}
