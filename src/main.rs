mod api;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod repositories;
mod schema;
mod services;
mod state;

pub use state::AppState;

use api::routes::create_router;
use config::{ConfigLoader, Environment};
use db::establish_async_connection_pool;
use logger::init_logger;
use tokio::net::TcpListener;
use tokio::signal;

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

    // Initialize database connection pool
    tracing::info!("Initializing database connection pool...");
    let pool = establish_async_connection_pool(&settings.database).await?;
    tracing::info!("Database connection pool initialized");

    // Create application state with services
    let state = AppState::new(pool);
    tracing::info!("Application state created");

    // Create router with all routes and middleware
    let router = create_router(state);
    tracing::info!("Router configured");

    // Bind to the configured address
    let address = settings.server.address();
    let listener = TcpListener::bind(&address).await.map_err(|e| {
        tracing::error!(error = %e, address = %address, "Failed to bind to address");
        anyhow::anyhow!("Failed to bind to {}: {}", address, e)
    })?;

    tracing::info!(address = %address, "Server listening");

    // Start the server with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");

    Ok(())
}

/// Waits for a shutdown signal (Ctrl+C or SIGTERM).
///
/// This function returns when either signal is received, allowing
/// the server to perform graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}
