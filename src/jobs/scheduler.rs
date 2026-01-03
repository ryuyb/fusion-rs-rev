use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler as TokioCronScheduler};

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::jobs::executor::JobExecutor;
use crate::jobs::models::ScheduledJob;
use crate::jobs::registry::JobRegistry;
use crate::repositories::JobRepository;

/// Wrapper around tokio-cron-scheduler with dynamic job management
pub struct JobScheduler {
    scheduler: Arc<Mutex<TokioCronScheduler>>,
    executor: Arc<JobExecutor>,
    registry: Arc<JobRegistry>,
    job_repo: JobRepository,
}

impl JobScheduler {
    pub async fn new(db_pool: AsyncDbPool, registry: JobRegistry) -> AppResult<Self> {
        let scheduler = TokioCronScheduler::new()
            .await
            .map_err(|e| AppError::Internal {
                source: anyhow::Error::from(e),
            })?;

        Ok(Self {
            scheduler: Arc::new(Mutex::new(scheduler)),
            executor: Arc::new(JobExecutor::new(db_pool.clone())),
            registry: Arc::new(registry),
            job_repo: JobRepository::new(db_pool),
        })
    }

    /// Start the scheduler and load jobs from database
    pub async fn start(&self) -> AppResult<()> {
        self.reload_jobs().await?;
        self.scheduler
            .lock()
            .await
            .start()
            .await
            .map_err(|e| AppError::Internal {
                source: anyhow::Error::from(e),
            })?;
        Ok(())
    }

    /// Stop the scheduler gracefully
    pub async fn stop(&self) -> AppResult<()> {
        self.scheduler
            .lock()
            .await
            .shutdown()
            .await
            .map_err(|e| AppError::Internal {
                source: anyhow::Error::from(e),
            })?;
        Ok(())
    }

    /// Reload all enabled jobs from database
    pub async fn reload_jobs(&self) -> AppResult<()> {
        let jobs = self.job_repo.get_enabled_jobs().await?;

        for job in jobs {
            self.schedule_job(job).await?;
        }

        Ok(())
    }

    /// Schedule a single job
    async fn schedule_job(&self, job: ScheduledJob) -> AppResult<()> {
        let executor = Arc::clone(&self.executor);
        let registry = Arc::clone(&self.registry);
        let job_clone = job.clone();

        let cron_job = Job::new_async(job.cron_expression.as_str(), move |_uuid, _lock| {
            let executor = Arc::clone(&executor);
            let registry = Arc::clone(&registry);
            let job = job_clone.clone();

            Box::pin(async move {
                let payload = job.payload.clone().unwrap_or(serde_json::json!({}));

                match registry.create_task(&job.job_type, payload) {
                    Ok(task) => {
                        if let Err(e) = executor.execute_job(job, task).await {
                            tracing::error!(error = %e, "Job execution failed");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to create task");
                    }
                }
            })
        })
        .map_err(|e| AppError::BadRequest {
            message: format!("Invalid cron expression: {}", e),
        })?;

        self.scheduler
            .lock()
            .await
            .add(cron_job)
            .await
            .map_err(|e| AppError::Internal {
                source: anyhow::Error::from(e),
            })?;

        Ok(())
    }
}
