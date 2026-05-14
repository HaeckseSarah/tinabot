mod app;
use app::{Config, LogLevel, Logger};
use clap::Parser;

use crate::app::Args;
fn main() {
    let args = Args::parse();
    let config = Config::load(&args);
    let logger = Logger::new(
        &config.get("TINA_LOG_LEVEL", "warn").to_string(),
        Some(config.get("TINA_LOG_FILE", "tina.log").to_string()),
    );

    logger.log(LogLevel::Debug, "main", "You should not see this message");
    logger.log(LogLevel::Warn, "main", "Hello World!");
}
