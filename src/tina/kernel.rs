use super::Event;
use super::Plugin;
use crate::config::Config;
use crate::logger::{LogLevel, Logger};
use crate::plugins::DummyPlugin;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell, RwLock, mpsc};

pub struct Kernel {
    config: Arc<Config>,
    logger: Arc<Logger>,
    event_rx: Mutex<Option<mpsc::Receiver<Event>>>,
    event_tx: OnceCell<mpsc::Sender<Event>>,
    plugins: RwLock<Vec<Arc<dyn Plugin>>>,
}

impl Kernel {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        Self {
            config,
            logger,
            event_rx: Mutex::new(None),
            event_tx: OnceCell::new(),
            plugins: RwLock::new(Vec::new()),
        }
    }

    fn get_event_tx(&self) -> &mpsc::Sender<Event> {
        self.event_tx.get().expect("Event channel not initialized!")
    }

    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger
            .log(LogLevel::Info, "Kernel", "Initializing Kernel...");

        let (event_tx, event_rx) = mpsc::channel::<Event>(100);
        self.event_tx
            .set(event_tx)
            .map_err(|_| "Event channel already set!")?;

        let mut rx_lock = self.event_rx.lock().await;
        *rx_lock = Some(event_rx);

        let mut booted_plugins: Vec<Arc<dyn Plugin>> = Vec::new();
        let dummy_plugin = DummyPlugin::new();
        dummy_plugin
            .boot(
                self.config.clone(),
                self.logger.clone(),
                self.get_event_tx().clone(),
            )
            .await?;
        booted_plugins.push(Arc::new(dummy_plugin));

        let mut plugins_write = self.plugins.write().await;
        *plugins_write = booted_plugins;
        Ok(())
    }

    pub async fn run(&self) {
        let mut rx_opt = self.event_rx.lock().await;
        let mut event_rx = rx_opt
            .take()
            .expect("Kernel-Receiver already started or not initialized!");

        // run plugins
        let plugins_read = self.plugins.read().await;
        for plugin in plugins_read.iter() {
            let plugin_clone = plugin.clone();

            self.logger.log(
                LogLevel::Info,
                "Kernel",
                &format!("Start Plugin-Task: {}", plugin_clone.id()),
            );

            tokio::spawn(async move {
                plugin_clone.run().await;
            });
        }

        self.logger
            .log(LogLevel::Info, "Kernel", "Running Kernel...");

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    match event.event_type.as_str() {
                        "killSignal" => {
                            // TODO check permissions
                            self.logger.log(
                                LogLevel::Debug,
                                "Kernel",
                                &format!("Received killSignal from: {}", event.source),
                            );
                            break;
                        }
                        _ => {
                            self.logger.log(
                                LogLevel::Debug,
                                "Kernel",
                                &format!("Received event: {:?}", event),
                            );
                        }
                    }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            self.logger.log(LogLevel::Info, "Kernel", "ctrl+c detected. Shuting down...");
                            break; // exit main loop
                        }

                        else => break,
            }
        }

        if let Err(e) = self.shutdown().await {
            self.logger.log(
                LogLevel::Error,
                "Kernel",
                &format!("Shutdown error: {:?}", e),
            );
        }
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger
            .log(LogLevel::Info, "Kernel", "stopping all plugins...");

        let plugins_read = self.plugins.read().await;

        for plugin in plugins_read.iter() {
            self.logger.log(
                LogLevel::Info,
                "Kernel",
                &format!("Call shutdown() on plugin: {}", plugin.id()),
            );

            if let Err(e) = plugin.shutdown().await {
                self.logger.log(
                    LogLevel::Error,
                    "Kernel",
                    &format!("Plugin '{}' error on shutdown: {:?}", plugin.id(), e),
                );
            }
        }

        self.logger.log(LogLevel::Info, "Kernel", "Bye Bye ");
        Ok(())
    }
}
