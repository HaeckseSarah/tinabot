use std::sync::Arc;
mod cli;
mod config;
mod kernel;
mod logger;
mod lua;

use crate::cli::Args;
use crate::config::Config;
use crate::kernel::Kernel;
use crate::logger::Logger;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Arc::new(Config::load(&args));
    let logger = Arc::new(Logger::new(
        &config.get("TINA_LOG_LEVEL", "warn").to_string(),
        Some(config.get("TINA_LOG_FILE", "tina.log").to_string()),
    ));

    let kernel = Kernel::new(config.clone(), logger.clone());
    kernel.init().await?;
    kernel.run().await;
    Ok(())
}
