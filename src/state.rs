//! Application state for Axum web framework.
//!
//! Contains shared services and resources that are accessible
//! across all request handlers.

use std::sync::Arc;

use crate::config::JwtConfig;
use crate::db::AsyncDbPool;
use crate::jobs::JobScheduler;
use crate::repositories::Repositories;
use crate::services::Services;

/// Application state containing all shared services and resources.
///
/// This struct is designed to be used with Axum's State extractor.
/// Cloning is cheap since both Services and AsyncDbPool use Arc internally.
#[derive(Clone)]
pub struct AppState {
    /// All business logic services
    pub services: Services,
    /// Direct access to the database connection pool
    pub db_pool: AsyncDbPool,
    /// JWT configuration for token generation and validation
    pub jwt_config: JwtConfig,
    /// Optional job scheduler (only present when jobs are enabled)
    pub scheduler: Option<Arc<JobScheduler>>,
}

impl AppState {
    /// Creates a new AppState from a database connection pool and JWT config.
    ///
    /// Initializes all repositories and services from the provided pool.
    ///
    /// # Arguments
    /// * `pool` - The async database connection pool
    /// * `jwt_config` - JWT configuration for authentication
    /// * `scheduler` - Optional job scheduler
    ///
    /// # Example
    /// ```ignore
    /// let pool = establish_async_connection_pool().await?;
    /// let state = AppState::new(pool, jwt_config, None);
    /// ```
    pub fn new(pool: AsyncDbPool, jwt_config: JwtConfig, scheduler: Option<JobScheduler>) -> Self {
        let repos = Repositories::new(pool.clone());
        let services = Services::new(repos);
        Self {
            services,
            db_pool: pool,
            jwt_config,
            scheduler: scheduler.map(Arc::new),
        }
    }
}
