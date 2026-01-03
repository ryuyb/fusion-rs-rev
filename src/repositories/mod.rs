//! Repository layer for data access operations.
//!
//! Provides async CRUD operations for all domain entities.

mod job_execution_repo;
mod job_repo;
mod notification_channel_repo;
mod notification_log_repo;
mod user_repo;

pub use job_execution_repo::JobExecutionRepository;
pub use job_repo::JobRepository;
pub use notification_channel_repo::NotificationChannelRepository;
pub use notification_log_repo::NotificationLogRepository;
pub use user_repo::UserRepository;

use crate::db::AsyncDbPool;

/// Aggregates all repositories for convenient access.
///
/// This struct is designed to be used as Axum application state.
/// Since `AsyncDbPool` uses `Arc` internally, cloning is cheap.
#[derive(Clone)]
pub struct Repositories {
    pub users: UserRepository,
    pub notification_channels: NotificationChannelRepository,
    pub notification_logs: NotificationLogRepository,
    pub jobs: JobRepository,
    pub executions: JobExecutionRepository,
}

impl Repositories {
    /// Creates a new Repositories instance with all repositories initialized.
    ///
    /// # Arguments
    /// * `pool` - The async database connection pool
    pub fn new(pool: AsyncDbPool) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            notification_channels: NotificationChannelRepository::new(pool.clone()),
            notification_logs: NotificationLogRepository::new(pool.clone()),
            jobs: JobRepository::new(pool.clone()),
            executions: JobExecutionRepository::new(pool),
        }
    }
}
