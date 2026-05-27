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
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Arc::new(Config::load(&args));
    let logger = Arc::new(Logger::new(
        &config.get("log.level").unwrap_or("warn".to_string()),
        config.get("log.file"),
    ));

    let kernel = Kernel::new(config.clone(), logger.clone());

    kernel.init().await?;
    kernel.run(args.target, args.command).await;
    Ok(())
}
