//! Server module for managing HTTP server lifecycle
//!
//! This module handles server initialization, startup, and graceful shutdown.

use crate::api::routes::create_router;
use crate::config::{Environment, settings::Settings};
use crate::db::establish_async_connection_pool;
use crate::state::AppState;
use tokio::net::TcpListener;
use tokio::signal;

/// HTTP server manager
pub struct Server {
    settings: Settings,
}

impl Server {
    /// Create a new server with the given settings
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Start the server and run until shutdown signal
    ///
    /// This method:
    /// 1. Logs startup information
    /// 2. Initializes database connection pool
    /// 3. Creates application state
    /// 4. Binds to configured address
    /// 5. Starts the HTTP server with graceful shutdown
    ///
    /// # Returns
    /// Returns Ok(()) on successful shutdown, or error on startup failure
    ///
    /// # Errors
    /// - Database connection pool initialization errors
    /// - Address binding errors
    /// - Server runtime errors
    pub async fn run(self) -> anyhow::Result<()> {
        // Log application startup information
        tracing::info!(
            app_name = %self.settings.application.name,
            app_version = %self.settings.application.version,
            environment = %Environment::from_env().as_str(),
            "Application starting"
        );

        // Log server configuration
        tracing::info!(
            host = %self.settings.server.host,
            port = %self.settings.server.port,
            request_timeout = %self.settings.server.request_timeout,
            keep_alive_timeout = %self.settings.server.keep_alive_timeout,
            "Server configuration loaded"
        );

        // Log database configuration (without sensitive URL details)
        tracing::info!(
            max_connections = %self.settings.database.max_connections,
            min_connections = %self.settings.database.min_connections,
            connection_timeout = %self.settings.database.connection_timeout,
            "Database configuration loaded"
        );

        // Log logger configuration
        tracing::info!(
            level = %self.settings.logger.level,
            console_enabled = %self.settings.logger.console.enabled,
            file_enabled = %self.settings.logger.file.enabled,
            "Logger configuration loaded"
        );

        // Log JWT configuration (without sensitive secret)
        tracing::info!(
            access_token_expiration = %self.settings.jwt.access_token_expiration,
            refresh_token_expiration = %self.settings.jwt.refresh_token_expiration,
            secret_configured = %(!self.settings.jwt.secret.is_empty()),
            "JWT configuration loaded"
        );

        // Validate JWT configuration
        self.settings.jwt.validate().map_err(|e| {
            tracing::error!(error = %e, "JWT configuration validation failed");
            anyhow::anyhow!("JWT configuration validation failed: {}", e)
        })?;
        tracing::info!("JWT configuration validated");

        tracing::info!("Configuration loaded successfully");

        // Initialize database connection pool
        tracing::info!("Initializing database connection pool...");
        let pool = establish_async_connection_pool(&self.settings.database).await?;
        tracing::info!("Database connection pool initialized");

        // Create application state with services
        let state = AppState::new(pool, self.settings.jwt.clone());
        tracing::info!("Application state created");

        // Create router with all routes and middleware
        let router = create_router(state);
        tracing::info!("Router configured");

        // Bind to the configured address
        let address = self.settings.server.address();
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
