use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::Instant;
use uuid::Uuid;

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::jobs::models::{NewJobExecution, ScheduledJob};
use crate::jobs::types::{JobContext, JobStatus, JobTask};
use crate::repositories::{JobExecutionRepository, JobRepository};

/// Tracks concurrent job executions in memory
#[derive(Clone)]
pub struct ConcurrencyTracker {
    running: Arc<RwLock<HashMap<String, usize>>>,
}

impl ConcurrencyTracker {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn can_execute(&self, job: &ScheduledJob) -> bool {
        if job.allow_concurrent {
            if let Some(max) = job.max_concurrent {
                let running = self.running.read().await;
                let count = running.get(&job.job_name).copied().unwrap_or(0);
                count < max as usize
            } else {
                true
            }
        } else {
            let running = self.running.read().await;
            !running.contains_key(&job.job_name)
        }
    }

    pub async fn increment(&self, job_name: &str) {
        let mut running = self.running.write().await;
        *running.entry(job_name.to_string()).or_insert(0) += 1;
    }

    pub async fn decrement(&self, job_name: &str) {
        let mut running = self.running.write().await;
        if let Some(count) = running.get_mut(job_name) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                running.remove(job_name);
            }
        }
    }
}

/// Executes jobs with retry, timeout, and concurrency control
pub struct JobExecutor {
    db_pool: AsyncDbPool,
    job_repo: JobRepository,
    execution_repo: JobExecutionRepository,
    concurrency: ConcurrencyTracker,
}

impl JobExecutor {
    pub fn new(db_pool: AsyncDbPool) -> Self {
        Self {
            job_repo: JobRepository::new(db_pool.clone()),
            execution_repo: JobExecutionRepository::new(db_pool.clone()),
            db_pool,
            concurrency: ConcurrencyTracker::new(),
        }
    }

    pub async fn execute_job(
        &self,
        job: ScheduledJob,
        task: Box<dyn JobTask>,
    ) -> AppResult<()> {
        if !self.concurrency.can_execute(&job).await {
            return Err(AppError::UnprocessableContent {
                message: format!("Concurrency limit reached for job: {}", job.job_name),
            });
        }

        self.concurrency.increment(&job.job_name).await;
        let result = self.execute_with_retry(&job, task).await;
        self.concurrency.decrement(&job.job_name).await;

        result
    }

    async fn execute_with_retry(
        &self,
        job: &ScheduledJob,
        task: Box<dyn JobTask>,
    ) -> AppResult<()> {
        let mut last_error = None;

        for attempt in 0..=job.max_retries {
            let execution_id = Uuid::new_v4();
            let start_time = Instant::now();

            let exec = NewJobExecution {
                job_id: job.id,
                job_name: job.job_name.clone(),
                execution_id,
                status: JobStatus::Running,
                retry_attempt: attempt,
            };

            let execution = self.execution_repo.create(exec).await?;

            let ctx = JobContext {
                execution_id,
                job_id: job.id,
                job_name: job.job_name.clone(),
                retry_attempt: attempt as u32,
                db_pool: self.db_pool.clone(),
                cancellation_token: tokio_util::sync::CancellationToken::new(),
            };

            let timeout_duration = Duration::from_secs(job.timeout_seconds as u64);
            let result = tokio::time::timeout(timeout_duration, task.execute(ctx)).await;

            let duration_ms = start_time.elapsed().as_millis() as i64;

            match result {
                Ok(Ok(())) => {
                    self.execution_repo
                        .complete(execution.id, JobStatus::Success, duration_ms, None, None)
                        .await?;
                    self.job_repo
                        .update_last_run(job.id, JobStatus::Success)
                        .await?;
                    return Ok(());
                }
                Ok(Err(e)) => {
                    let error_msg = e.to_string();
                    self.execution_repo
                        .complete(
                            execution.id,
                            JobStatus::Failed,
                            duration_ms,
                            Some(error_msg.clone()),
                            None,
                        )
                        .await?;
                    last_error = Some(error_msg);

                    if attempt < job.max_retries {
                        let delay = self.calculate_retry_delay(job, attempt as u32);
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(_) => {
                    let error_msg = format!("Job timeout after {}s", job.timeout_seconds);
                    self.execution_repo
                        .complete(
                            execution.id,
                            JobStatus::Timeout,
                            duration_ms,
                            Some(error_msg.clone()),
                            None,
                        )
                        .await?;
                    self.job_repo
                        .update_last_run(job.id, JobStatus::Timeout)
                        .await?;
                    return Err(AppError::Internal {
                        source: anyhow::anyhow!("Job timeout after {}s", job.timeout_seconds),
                    });
                }
            }
        }

        self.job_repo
            .update_last_run(job.id, JobStatus::Failed)
            .await?;

        Err(AppError::Internal {
            source: anyhow::anyhow!(
                "Job execution failed: {}",
                last_error.unwrap_or_else(|| "Unknown error".to_string())
            ),
        })
    }

    fn calculate_retry_delay(&self, job: &ScheduledJob, attempt: u32) -> Duration {
        let base_delay = job.retry_delay_seconds as f64;
        let multiplier = job.retry_backoff_multiplier.to_string().parse::<f64>().unwrap_or(2.0);
        let delay_secs = base_delay * multiplier.powi(attempt as i32);
        Duration::from_secs(delay_secs as u64)
    }
}
