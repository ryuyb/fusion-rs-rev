//! Job-related DTOs for API requests and responses.

use crate::jobs::models::{NewScheduledJob, ScheduledJob, JobExecution, UpdateScheduledJob};
use crate::jobs::types::JobStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request body for creating a new scheduled job.
#[derive(Debug, Deserialize, ToSchema, Validate)]
#[schema(example = json!({
    "job_name": "daily_cleanup",
    "job_type": "data_cleanup",
    "cron_expression": "0 0 2 * * * *",
    "enabled": true,
    "allow_concurrent": false,
    "max_concurrent": 1,
    "max_retries": 3,
    "timeout_seconds": 300,
    "payload": {
        "retention_days": 30
    },
    "description": "Clean up old job execution records"
}))]
pub struct CreateJobRequest {
    #[validate(length(min = 1, max = 255, message = "Job name must be between 1 and 255 characters"))]
    #[schema(example = "daily_cleanup")]
    pub job_name: String,

    #[validate(length(min = 1, max = 100, message = "Job type must be between 1 and 100 characters"))]
    #[schema(example = "data_cleanup")]
    pub job_type: String,

    #[validate(length(min = 1, max = 255, message = "Cron expression must be between 1 and 255 characters"))]
    #[schema(example = "0 0 2 * * * *")]
    pub cron_expression: String,

    #[serde(default = "default_true")]
    #[schema(example = true)]
    pub enabled: bool,

    #[serde(default)]
    #[schema(example = false)]
    pub allow_concurrent: bool,

    #[schema(example = 1)]
    pub max_concurrent: Option<i32>,

    #[serde(default = "default_max_retries")]
    #[schema(example = 3)]
    pub max_retries: i32,

    #[serde(default = "default_timeout")]
    #[schema(example = 300)]
    pub timeout_seconds: i32,

    #[schema(value_type = Option<Object>, example = json!({"retention_days": 30}))]
    pub payload: Option<JsonValue>,

    #[schema(example = "Clean up old job execution records")]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_max_retries() -> i32 {
    3
}

fn default_timeout() -> i32 {
    300
}

impl CreateJobRequest {
    pub fn into_new_job(self) -> NewScheduledJob {
        NewScheduledJob {
            job_name: self.job_name,
            job_type: self.job_type,
            cron_expression: self.cron_expression,
            enabled: self.enabled,
            allow_concurrent: self.allow_concurrent,
            max_concurrent: self.max_concurrent,
            max_retries: self.max_retries,
            retry_delay_seconds: 60,
            retry_backoff_multiplier: bigdecimal::BigDecimal::from(2),
            timeout_seconds: self.timeout_seconds,
            payload: self.payload,
            description: self.description,
            created_by: None,
        }
    }
}

/// Request body for updating a scheduled job.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateJobRequest {
    #[validate(length(min = 1, max = 255))]
    pub cron_expression: Option<String>,

    pub enabled: Option<bool>,
    pub allow_concurrent: Option<bool>,
    pub max_concurrent: Option<i32>,
    pub max_retries: Option<i32>,
    pub retry_delay_seconds: Option<i32>,
    pub retry_backoff_multiplier: Option<f64>,
    pub timeout_seconds: Option<i32>,
    pub payload: Option<JsonValue>,
    pub description: Option<String>,
}

impl UpdateJobRequest {
    pub fn into_update_job(self) -> UpdateScheduledJob {
        UpdateScheduledJob {
            cron_expression: self.cron_expression,
            enabled: self.enabled,
            allow_concurrent: self.allow_concurrent,
            max_concurrent: self.max_concurrent.map(Some),
            max_retries: self.max_retries,
            retry_delay_seconds: self.retry_delay_seconds,
            retry_backoff_multiplier: self.retry_backoff_multiplier.map(|v| {
                bigdecimal::BigDecimal::try_from(v).unwrap_or_else(|_| bigdecimal::BigDecimal::from(2))
            }),
            timeout_seconds: self.timeout_seconds,
            payload: self.payload,
            description: self.description,
        }
    }
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response body for scheduled job data.
#[derive(Debug, Serialize, ToSchema)]
pub struct JobResponse {
    pub id: i32,
    pub job_name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub allow_concurrent: bool,
    pub max_concurrent: Option<i32>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
    pub retry_backoff_multiplier: String,
    pub timeout_seconds: i32,
    pub payload: Option<JsonValue>,
    pub description: Option<String>,
    pub last_run_at: Option<String>,
    pub last_run_status: Option<JobStatus>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

impl From<ScheduledJob> for JobResponse {
    fn from(job: ScheduledJob) -> Self {
        Self {
            id: job.id,
            job_name: job.job_name,
            job_type: job.job_type,
            cron_expression: job.cron_expression,
            enabled: job.enabled,
            allow_concurrent: job.allow_concurrent,
            max_concurrent: job.max_concurrent,
            max_retries: job.max_retries,
            retry_delay_seconds: job.retry_delay_seconds,
            retry_backoff_multiplier: job.retry_backoff_multiplier.to_string(),
            timeout_seconds: job.timeout_seconds,
            payload: job.payload,
            description: job.description,
            last_run_at: job.last_run_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            last_run_status: job.last_run_status,
            next_run_at: job.next_run_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            created_at: job.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            updated_at: job.updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            created_by: job.created_by,
        }
    }
}

/// Response body for job execution data.
#[derive(Debug, Serialize, ToSchema)]
pub struct JobExecutionResponse {
    pub id: i64,
    pub job_id: i32,
    pub job_name: String,
    pub execution_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub status: JobStatus,
    pub retry_attempt: i32,
    pub error_message: Option<String>,
    pub error_details: Option<JsonValue>,
    pub result: Option<JsonValue>,
}

impl From<JobExecution> for JobExecutionResponse {
    fn from(exec: JobExecution) -> Self {
        Self {
            id: exec.id,
            job_id: exec.job_id,
            job_name: exec.job_name,
            execution_id: exec.execution_id.to_string(),
            started_at: exec.started_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            completed_at: exec.completed_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            duration_ms: exec.duration_ms,
            status: exec.status,
            retry_attempt: exec.retry_attempt,
            error_message: exec.error_message,
            error_details: exec.error_details,
            result: exec.result,
        }
    }
}
