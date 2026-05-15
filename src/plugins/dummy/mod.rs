use crate::Config;
use crate::logger::{LogLevel, Logger};
use crate::tina::Event;
use crate::tina::Plugin;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tokio::sync::mpsc;
use uuid::Uuid;

use std::io;
pub struct DummyPlugin {
    config: OnceCell<Arc<Config>>,
    logger: OnceCell<Arc<Logger>>,
    event_tx: OnceCell<mpsc::Sender<Event>>,
}

impl DummyPlugin {
    pub fn new() -> Self {
        Self {
            config: OnceCell::new(),
            logger: OnceCell::new(),
            event_tx: OnceCell::new(),
            //cancel_token: CancellationToken::new(),
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
        "dummy"
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
            .log(LogLevel::Info, "DummyPlugin", "Booted successfully.");

        Ok(())
    }

    async fn run(&self) {
        self.logger()
            .log(LogLevel::Info, "DummyPlugin", "Dummy plugin started.");

        loop {
            println!("tina>");
            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let event = Event {
                id: Uuid::new_v4(),
                source: self.id().to_string(),
                event_type: "onMessage".to_string(),
                payload: serde_json::json!({"message": input.trim()}),
                metadata: std::collections::HashMap::new(),
            };
            let _ = self.event_tx().send(event).await;
        }
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger()
            .log(LogLevel::Info, "DummyPlugin", "shutdown");
        Ok(())
    }
}
