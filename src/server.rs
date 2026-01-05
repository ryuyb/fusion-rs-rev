//! Server module for managing HTTP server lifecycle
//!
//! This module handles server initialization, startup, and graceful shutdown.

use crate::api::routes::create_router;
use crate::cache::{CacheManager, init_cache};
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

    /// Log all configuration settings at startup
    fn log_startup_config(&self) {
        tracing::info!(
            app_name = %self.settings.application.name,
            app_version = %self.settings.application.version,
            environment = %Environment::from_env().as_str(),
            "Application starting"
        );

        tracing::info!(
            host = %self.settings.server.host,
            port = %self.settings.server.port,
            log_level = %self.settings.logger.level,
            "Configuration loaded"
        );
    }

    /// Validate configuration settings
    fn validate_config(&self) -> anyhow::Result<()> {
        self.settings.jwt.validate().map_err(|e| {
            tracing::error!(error = %e, "JWT configuration validation failed");
            anyhow::anyhow!("JWT configuration validation failed: {}", e)
        })?;
        tracing::info!("JWT configuration validated");
        Ok(())
    }

    /// Initialize database connection pool
    async fn initialize_database(&self) -> anyhow::Result<crate::db::AsyncDbPool> {
        tracing::info!("Initializing database connection pool...");
        let pool = establish_async_connection_pool(&self.settings.database).await?;
        tracing::info!("Database connection pool initialized");
        Ok(pool)
    }

    /// Initialize job scheduler if enabled
    async fn initialize_scheduler(
        &self,
        pool: crate::db::AsyncDbPool,
    ) -> anyhow::Result<Option<crate::jobs::JobScheduler>> {
        if self.settings.jobs.enabled {
            tracing::info!("Initializing job scheduler");

            let mut registry = crate::jobs::JobRegistry::new();
            registry.register::<crate::jobs::tasks::DataCleanupTask>();

            let job_scheduler = crate::jobs::JobScheduler::new(pool, registry).await?;
            job_scheduler.start().await?;

            tracing::info!("Job scheduler started");
            Ok(Some(job_scheduler))
        } else {
            Ok(None)
        }
    }

    /// Initialize cache manager if enabled
    async fn initialize_cache(&self) -> anyhow::Result<Option<CacheManager>> {
        if self.settings.cache.enabled {
            tracing::info!(
                backend = ?self.settings.cache.backend,
                "Initializing cache manager"
            );

            let cache = init_cache(self.settings.cache.clone(), "app").await?;
            tracing::info!(
                "Cache manager initialized: {backend:?}",
                backend = self.settings.cache.backend
            );
            Ok(Some(cache.clone()))
        } else {
            tracing::info!("Cache disabled");
            Ok(None)
        }
    }

    /// Bind TCP listener to configured address
    async fn bind_listener(&self) -> anyhow::Result<TcpListener> {
        let address = self.settings.server.address();
        let listener = TcpListener::bind(&address).await.map_err(|e| {
            tracing::error!(error = %e, address = %address, "Failed to bind to address");
            anyhow::anyhow!("Failed to bind to {}: {}", address, e)
        })?;

        tracing::info!(address = %format!("http://{}", address), "Server listening");
        Ok(listener)
    }

    /// Gracefully shutdown the job scheduler
    async fn shutdown_scheduler(state: &AppState) -> anyhow::Result<()> {
        if let Some(scheduler) = &state.scheduler {
            tracing::info!("Stopping job scheduler");
            scheduler.stop().await?;
            tracing::info!("Job scheduler stopped");
        }
        Ok(())
    }

    /// Start the server and run until shutdown signal
    ///
    /// This method:
    /// 1. Logs startup information
    /// 2. Validates configuration
    /// 3. Initializes database connection pool
    /// 4. Initializes job scheduler (if enabled)
    /// 5. Initializes cache manager (if enabled)
    /// 6. Creates application state
    /// 7. Binds to configured address
    /// 8. Starts the HTTP server with graceful shutdown
    ///
    /// # Returns
    /// Returns Ok(()) on successful shutdown, or error on startup failure
    ///
    /// # Errors
    /// - Configuration validation errors
    /// - Database connection pool initialization errors
    /// - Job scheduler initialization errors
    /// - Cache manager initialization errors
    /// - Address binding errors
    /// - Server runtime errors
    pub async fn run(self) -> anyhow::Result<()> {
        self.log_startup_config();
        self.validate_config()?;

        let pool = self.initialize_database().await?;
        let scheduler = self.initialize_scheduler(pool.clone()).await?;
        let cache = self.initialize_cache().await?;

        let state = AppState::new(pool, self.settings.jwt.clone(), scheduler, cache);
        tracing::info!("Application state created");

        let router = create_router(state.clone());
        tracing::info!("Router configured");

        let listener = self.bind_listener().await?;

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Self::shutdown_scheduler(&state).await?;
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
