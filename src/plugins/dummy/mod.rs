use crate::Config;
use crate::logger::{LogLevel, Logger};
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

    fn event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Plugin not initialized!")
    }
}

#[async_trait]
impl Plugin for DummyPlugin {
    fn id(&self) -> &str {
        "dummyPlugin"
    }

    async fn boot(
        &self,
        config: Arc<Config>,
        logger: Arc<Logger>,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.set(config).map_err(|_| "config already set!")?;
        self.logger.set(logger).map_err(|_| "Logger already set!")?;
        self.event_tx
            .set(event_tx)
            .map_err(|_| "Event-Channel already set!")?;

        // Ab jetzt ist der Logger einsatzbereit
        self.logger()
            .log(LogLevel::Info, self.id(), "Booted successfully.");

        Ok(())
    }

    async fn run(&self) {
        self.logger()
            .log(LogLevel::Info, self.id(), "Dummy plugin started.");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut input = String::new();

        loop {
            if self.cancel_token.is_cancelled() {
                self.logger().log(LogLevel::Debug, self.id(), "Kill me");
                break;
            }

            println!("tina>");
            input.clear();

            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    self.logger().log(LogLevel::Info, self.id(), "initiatiing shutdown...");
                    break;
                }

                result = reader.read_line(&mut input) => {
                    match result {
                        Ok(0) => {
                            break;
                        }
                    Ok(_) => {
                        let trimmed = input.trim();
                        if trimmed.is_empty() { continue; }
                        if trimmed == "exit" {
                            let event = Event {
                                id: Uuid::new_v4(),
                                source: self.id().to_string(),
                                event_type: "killSignal".to_string(),
                                payload: None,
                                metadata: None,
                            };

                            let _ = self.event_tx().send(event).await;
                            break;
                        }

                        let event = Event {
                            id: Uuid::new_v4(),
                            source: self.id().to_string(),
                            event_type: "onMessage".to_string(),
                            payload: Some(serde_json::json!({"message": input.trim()})),
                            metadata: None,
                        };

                        let _ = self.event_tx().send(event).await;
                        }
                    Err(e) => {
                        self.logger().log(LogLevel::Error, self.id(), &format!("error on reading from stdin: {}", e));
                        break;
                        }
                    }
                }
            }
        }
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger().log(LogLevel::Info, self.id(), "shutdown");
        self.cancel_token.cancel();

        Ok(())
    }
}
