//! Service layer for business logic operations.
//!
//! Services encapsulate business logic and coordinate between
//! repositories and handlers.

mod job_service;
mod live_service;
pub mod notifications;
mod user_service;

pub use job_service::JobService;
pub use live_service::LiveService;
pub use notifications::NotificationService;
pub use user_service::UserService;

use crate::repositories::Repositories;

/// Aggregates all services for convenient access.
///
/// This struct is designed to be used as Axum application state.
/// Cloning is cheap since underlying pools use `Arc` internally.
#[derive(Clone)]
pub struct Services {
    pub users: UserService,
    pub notifications: NotificationService,
    pub jobs: JobService,
    pub live: LiveService,
}

impl Services {
    /// Creates a new Services instance from Repositories.
    pub fn new(repos: Repositories) -> Self {
        Self {
            users: UserService::new(repos.users),
            notifications: NotificationService::new(
                repos.notification_channels,
                repos.notification_logs,
            ),
            jobs: JobService::new(repos.jobs, repos.executions),
            live: LiveService::new(),
        }
    }
}
