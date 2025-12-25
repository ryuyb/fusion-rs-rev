mod db;
mod error;
mod logger;
mod models;
mod repositories;
mod schema;

use logger::{LoggerConfig, init_logger};
use crate::logger::LogFormat;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the advanced logger with default configuration
    let mut config = LoggerConfig::default();
    config.level = "trace".to_string();
    config.file.enabled = true;
    config.file.append = false;
    config.file.format = LogFormat::Json;
    let handle = init_logger(config)?;

    println!("Hello, world!");
    tracing::trace!("Hello, world!");
    tracing::debug!("Hello, world!");
    tracing::info!("Hello, world!");
    tracing::warn!("Hello, world!");
    tracing::error!("Hello, world!");

    // Create and enter a span
    let span = tracing::info_span!("my_operation", user_id = 123);
    {
        let _enter = span.enter();

        tracing::info!("Inside the span");
        tracing::debug!("Doing some work...");
    }
    
    // The span will be exited when _enter is dropped

    tracing::info!("Outside the span");

    handle.set_level("info")?;
    tracing::debug!("Hello, world after change!");
    tracing::info!("Hello, world after change!");

    Ok(())
}
