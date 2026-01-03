//! Job service for business logic operations.

use crate::error::AppResult;
use crate::jobs::models::{JobExecution, NewScheduledJob, ScheduledJob, UpdateScheduledJob};
use crate::repositories::{JobExecutionRepository, JobRepository};

/// Job service for handling job-related business logic.
#[derive(Clone)]
pub struct JobService {
    job_repo: JobRepository,
    execution_repo: JobExecutionRepository,
}

impl JobService {
    /// Creates a new JobService with the given repositories.
    pub fn new(job_repo: JobRepository, execution_repo: JobExecutionRepository) -> Self {
        Self {
            job_repo,
            execution_repo,
        }
    }

    /// Creates a new scheduled job.
    pub async fn create_job(&self, new_job: NewScheduledJob) -> AppResult<ScheduledJob> {
        self.job_repo.create(new_job).await
    }

    /// Gets a job by ID.
    pub async fn get_job(&self, id: i32) -> AppResult<ScheduledJob> {
        self.job_repo.get_by_id(id).await
    }

    /// Gets all enabled jobs.
    pub async fn get_enabled_jobs(&self) -> AppResult<Vec<ScheduledJob>> {
        self.job_repo.get_enabled_jobs().await
    }

    /// Updates a job.
    pub async fn update_job(&self, id: i32, update: UpdateScheduledJob) -> AppResult<ScheduledJob> {
        self.job_repo.update(id, update).await
    }

    /// Deletes a job.
    pub async fn delete_job(&self, id: i32) -> AppResult<()> {
        self.job_repo.delete(id).await
    }

    /// Pauses a job (disables it).
    pub async fn pause_job(&self, id: i32) -> AppResult<ScheduledJob> {
        let update = UpdateScheduledJob {
            enabled: Some(false),
            ..Default::default()
        };
        self.job_repo.update(id, update).await
    }

    /// Resumes a job (enables it).
    pub async fn resume_job(&self, id: i32) -> AppResult<ScheduledJob> {
        let update = UpdateScheduledJob {
            enabled: Some(true),
            ..Default::default()
        };
        self.job_repo.update(id, update).await
    }

    /// Lists execution history for a job.
    pub async fn list_job_executions(
        &self,
        job_id: i32,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<JobExecution>> {
        self.execution_repo.list_by_job(job_id, limit, offset).await
    }
}
