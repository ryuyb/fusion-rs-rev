use thiserror::Error;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Job execution timeout after {0}s")]
    Timeout(u64),

    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    #[error("Job not found: {0}")]
    NotFound(String),

    #[error("Job already exists: {0}")]
    AlreadyExists(String),

    #[error("Concurrency limit reached for job: {0}")]
    ConcurrencyLimitReached(String),

    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type JobResult<T> = Result<T, JobError>;
