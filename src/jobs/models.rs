use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::jobs::types::JobStatus;
use crate::schema::{job_executions, scheduled_jobs};

// ============================================================================
// ScheduledJob Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = scheduled_jobs)]
pub struct ScheduledJob {
    pub id: i32,
    pub job_name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub allow_concurrent: bool,
    pub max_concurrent: Option<i32>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
    pub retry_backoff_multiplier: bigdecimal::BigDecimal,
    pub timeout_seconds: i32,
    pub payload: Option<JsonValue>,
    pub description: Option<String>,
    pub last_run_at: Option<NaiveDateTime>,
    pub last_run_status: Option<JobStatus>,
    pub next_run_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub created_by: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = scheduled_jobs)]
pub struct NewScheduledJob {
    pub job_name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub allow_concurrent: bool,
    pub max_concurrent: Option<i32>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
    pub retry_backoff_multiplier: bigdecimal::BigDecimal,
    pub timeout_seconds: i32,
    pub payload: Option<JsonValue>,
    pub description: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Default, AsChangeset)]
#[diesel(table_name = scheduled_jobs)]
pub struct UpdateScheduledJob {
    pub cron_expression: Option<String>,
    pub enabled: Option<bool>,
    pub allow_concurrent: Option<bool>,
    pub max_concurrent: Option<Option<i32>>,
    pub max_retries: Option<i32>,
    pub retry_delay_seconds: Option<i32>,
    pub retry_backoff_multiplier: Option<bigdecimal::BigDecimal>,
    pub timeout_seconds: Option<i32>,
    pub payload: Option<JsonValue>,
    pub description: Option<String>,
}

// ============================================================================
// JobExecution Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = job_executions)]
pub struct JobExecution {
    pub id: i64,
    pub job_id: i32,
    pub job_name: String,
    pub execution_id: uuid::Uuid,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub duration_ms: Option<i64>,
    pub status: JobStatus,
    pub retry_attempt: i32,
    pub error_message: Option<String>,
    pub error_details: Option<JsonValue>,
    pub result: Option<JsonValue>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = job_executions)]
pub struct NewJobExecution {
    pub job_id: i32,
    pub job_name: String,
    pub execution_id: uuid::Uuid,
    pub status: JobStatus,
    pub retry_attempt: i32,
}
