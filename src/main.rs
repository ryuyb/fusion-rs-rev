mod config;
mod db;
mod error;
mod logger;
mod models;
mod repositories;
mod schema;
mod services;

use config::{ConfigLoader, Environment};
use logger::init_logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from files and environment variables
    let loader = ConfigLoader::new()?;
    let settings = loader.load()?;

    // Convert LoggerSettings to LoggerConfig and initialize the logger
    let logger_config = settings.logger.clone().into_logger_config()?;
    let _handle = init_logger(logger_config)?;

    // Log application startup information
    tracing::info!(
        app_name = %settings.application.name,
        app_version = %settings.application.version,
        environment = %Environment::from_env().as_str(),
        "Application starting"
    );

    // Log server configuration
    tracing::info!(
        host = %settings.server.host,
        port = %settings.server.port,
        request_timeout = %settings.server.request_timeout,
        keep_alive_timeout = %settings.server.keep_alive_timeout,
        "Server configuration loaded"
    );

    // Log database configuration (without sensitive URL details)
    tracing::info!(
        max_connections = %settings.database.max_connections,
        min_connections = %settings.database.min_connections,
        connection_timeout = %settings.database.connection_timeout,
        "Database configuration loaded"
    );

    // Log logger configuration
    tracing::info!(
        level = %settings.logger.level,
        console_enabled = %settings.logger.console.enabled,
        file_enabled = %settings.logger.file.enabled,
        "Logger configuration loaded"
    );

    tracing::info!("Configuration loaded successfully");

    Ok(())
}
