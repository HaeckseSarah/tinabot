use tina_plugin_api::{ConfigLookup, Event, EventTx, EventValue, LogFn, Plugin, ScriptRegistry};

use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{OnceCell, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct DummyPlugin {
    config_get: OnceCell<ConfigLookup>,
    log: OnceCell<LogFn>,
    event_tx: OnceCell<EventTx>,
    cancel_token: CancellationToken,
}

impl DummyPlugin {
    pub fn new() -> Self {
        Self {
            config_get: OnceCell::new(),
            log: OnceCell::new(),
            event_tx: OnceCell::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    fn event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Plugin not initialized!")
    }

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

    async fn emit_event(&self, event_type: &str, payload: Option<HashMap<String, EventValue>>) {
        let event = Event {
            id: Uuid::new_v4(),
            source: self.id().to_string(),
            event_type: event_type.to_string(),
            payload: payload,
        };

        let _ = self.event_tx().send(event).await;
    }

    async fn send_message(&self, message: String) {
        let mut payload = HashMap::new();
        payload.insert("message".to_string(), EventValue::String(message));
        self.emit_event("message", Some(payload)).await;
    }
}

#[async_trait]
impl Plugin for DummyPlugin {
    fn id(&self) -> &str {
        "dummyPlugin"
    }

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

        script_registry.register_function("ping", |_lua, msg: String| {
            println!("{}", msg.as_str());
            Ok(())
        });

        Ok(())
    }

    async fn run(&self) {
        self.log_info("Dummy plugin started.");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        loop {
            if self.cancel_token.is_cancelled() {
                self.log_debug("token is cancelled");
                break;
            }

            println!("tina>");
            input.clear();

            tokio::select! {
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
                            if trimmed_input == "exit" {
                                self.emit_event("killSignal",None).await;
                                break;
                            }

                            self.send_message(trimmed_input.to_string()).await;
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

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.log_info("shutdown");
        self.cancel_token.cancel();

        Ok(())
    }
}
