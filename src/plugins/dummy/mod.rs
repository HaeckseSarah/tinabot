use crate::Config;
use crate::logger::{LogLevel, Logger};
use crate::lua::LuaFunctionRegistry;
use crate::tina::{Event, Plugin};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{OnceCell, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct DummyPlugin {
    config: OnceCell<Arc<Config>>,
    logger: OnceCell<Arc<Logger>>,
    event_tx: OnceCell<mpsc::Sender<Event>>,
    cancel_token: CancellationToken,
}

impl DummyPlugin {
    pub fn new() -> Self {
        Self {
            config: OnceCell::new(),
            logger: OnceCell::new(),
            event_tx: OnceCell::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    fn logger(&self) -> &Logger {
        self.logger.get().expect("Plugin not initialized!").as_ref()
    }

    fn log_error(&self, msg: &str) {
        self.logger().log(LogLevel::Error, self.id(), msg);
    }
    fn log_info(&self, msg: &str) {
        self.logger().log(LogLevel::Info, self.id(), msg);
    }
    fn log_debug(&self, msg: &str) {
        self.logger().log(LogLevel::Debug, self.id(), msg);
    }

    fn event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Plugin not initialized!")
    }

    async fn emit_event(&self, event_type: &str, payload: Option<serde_json::Value>) {
        let event = Event {
            id: Uuid::new_v4(),
            source: self.id().to_string(),
            event_type: event_type.to_string(),
            payload: payload,
            metadata: None,
        };

        let _ = self.event_tx().send(event).await;
    }

    async fn send_message(&self, message: String) {
        self.emit_event("message", Some(serde_json::json!({"message": message})))
            .await;
    }
}

#[async_trait]
impl Plugin for DummyPlugin {
    fn id(&self) -> &str {
        "dummyPlugin"
    }

    async fn boot<'lua>(
        &self,
        config: Arc<Config>,
        logger: Arc<Logger>,
        event_tx: mpsc::Sender<Event>,
        lua_registry: &mut LuaFunctionRegistry<'lua>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.set(config).map_err(|_| "config already set!")?;
        self.logger.set(logger).map_err(|_| "Logger already set!")?;
        self.event_tx
            .set(event_tx)
            .map_err(|_| "Event-Channel already set!")?;

        lua_registry.register_function("ping", |_lua, msg: String| {
            println!("{}", msg.as_str());
            Ok(())
        });

        self.log_info("Booted successfully.");

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
