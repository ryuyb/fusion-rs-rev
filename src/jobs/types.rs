use async_trait::async_trait;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use utoipa::ToSchema;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum, ToSchema)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Running.to_string(), "running");
        assert_eq!(JobStatus::Success.to_string(), "success");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
        assert_eq!(JobStatus::Timeout.to_string(), "timeout");
        assert_eq!(JobStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_job_status_serialization() {
        let status = JobStatus::Success;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"success\"");

        let deserialized: JobStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, JobStatus::Success);
    }

    #[test]
    fn test_job_status_equality() {
        assert_eq!(JobStatus::Pending, JobStatus::Pending);
        assert_ne!(JobStatus::Pending, JobStatus::Running);
    }
}
