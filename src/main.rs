mod api;
pub mod cli;
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
use cli::{Cli, Commands, ConfigurationMerger, CommandHandler};
use clap::Parser;
use config::{Environment, settings::Settings};
use db::establish_async_connection_pool;
use logger::init_logger;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Load base configuration from files and environment variables
    let merger = match ConfigurationMerger::from_config_path(cli.config.as_ref()) {
        Ok(merger) => merger,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Merge CLI arguments with configuration
    let settings = match merger.merge_cli_args(&cli) {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Configuration merge error: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize logger early for CLI commands (before command validation)
    let logger_config = match settings.logger.clone().into_logger_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Logger configuration error: {}", e);
            std::process::exit(1);
        }
    };
    let _handle = match init_logger(logger_config) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Logger initialization error: {}", e);
            std::process::exit(1);
        }
    };

    // Create command handler for dispatching CLI commands
    let command_handler = CommandHandler::new(settings.clone());

    // Validate CLI arguments and configuration
    if let Err(e) = command_handler.validate_command_args(&cli) {
        tracing::error!("Command validation error: {}", e);
        std::process::exit(1);
    }

    // Dispatch based on command type
    match cli.command.as_ref() {
        Some(Commands::Serve { dry_run, .. }) => {
            // Handle serve command (including dry-run)
            if *dry_run {
                match command_handler.handle_serve(true).await {
                    Ok(()) => {
                        tracing::info!("Dry-run validation completed successfully");
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::error!("Dry-run validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            // Continue with normal server startup below
        }
        Some(Commands::Migrate { dry_run, rollback }) => {
            // Handle migrate command and exit
            match command_handler.handle_migrate(*dry_run, *rollback).await {
                Ok(()) => {
                    tracing::info!("Migration operation completed successfully");
                    return Ok(());
                }
                Err(e) => {
                    tracing::error!("Migration operation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // No command specified, default to serve behavior
            tracing::info!("No command specified, starting server with default behavior");
            // Continue with normal server startup below
        }
    }

    // Server startup logic (for serve command or default behavior)
    match start_server(settings).await {
        Ok(()) => {
            tracing::info!("Server shutdown completed successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Start the web server with the given configuration
async fn start_server(settings: Settings) -> anyhow::Result<()> {
    // Logger is already initialized in main(), so we skip logger initialization here

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
