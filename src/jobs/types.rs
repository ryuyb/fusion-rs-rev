use async_trait::async_trait;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::db::AsyncDbPool;
use crate::error::AppResult;

/// Job execution context passed to tasks
#[derive(Clone)]
pub struct JobContext {
    pub execution_id: Uuid,
    pub job_id: i32,
    pub job_name: String,
    pub retry_attempt: u32,
    pub db_pool: AsyncDbPool,
    pub cancellation_token: CancellationToken,
}

/// Job execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[db_enum(existing_type_path = "crate::schema::sql_types::JobStatus")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed,
    Timeout,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Success => write!(f, "success"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Timeout => write!(f, "timeout"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Trait that all job tasks must implement
#[async_trait]
pub trait JobTask: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this task type
    fn task_type() -> &'static str
    where
        Self: Sized;

    /// Execute the task
    async fn execute(&self, ctx: JobContext) -> AppResult<()>;

    /// Optional description
    fn description(&self) -> Option<String> {
        None
    }
}
