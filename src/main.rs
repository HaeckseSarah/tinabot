mod app;
use crate::app::{Args, Config, Logger};
use crate::app::kernel::Kernel;
use clap::Parser;
use std::sync::Arc;


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
