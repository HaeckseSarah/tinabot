use std::sync::Arc;
use crate::config::Config;
use crate::logger::{Logger, LogLevel};

pub struct Kernel {
    config: Arc<Config>,
    logger: Arc<Logger>,
}

impl Kernel {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        Self { config, logger }
    }

    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.logger.log(LogLevel::Info, "Kernel", "Initializing Kernel...");
        Ok(())
    }

    pub async fn run(&self) {
        self.logger.log(LogLevel::Info, "Kernel", "Running Kernel...");

    }
}
