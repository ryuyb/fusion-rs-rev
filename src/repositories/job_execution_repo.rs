use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::Value as JsonValue;

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::jobs::models::{JobExecution, NewJobExecution};
use crate::jobs::types::JobStatus;
use crate::schema::job_executions;

#[derive(Clone)]
pub struct JobExecutionRepository {
    pool: AsyncDbPool,
}

impl JobExecutionRepository {
    pub fn new(pool: AsyncDbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, exec: NewJobExecution) -> AppResult<JobExecution> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::insert_into(job_executions::table)
            .values(&exec)
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    pub async fn complete(
        &self,
        id: i64,
        status: JobStatus,
        duration_ms: i64,
        error: Option<String>,
        result: Option<JsonValue>,
    ) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::update(job_executions::table.find(id))
            .set((
                job_executions::completed_at.eq(diesel::dsl::now),
                job_executions::duration_ms.eq(duration_ms),
                job_executions::status.eq(status),
                job_executions::error_message.eq(error),
                job_executions::result.eq(result),
            ))
            .execute(&mut conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn list_by_job(
        &self,
        job_id: i32,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<JobExecution>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        job_executions::table
            .filter(job_executions::job_id.eq(job_id))
            .order(job_executions::started_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .await
            .map_err(AppError::from)
    }

    pub async fn cleanup_old_executions(&self, retention_days: i64) -> AppResult<usize> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        let cutoff = Utc::now().naive_utc() - Duration::days(retention_days);

        diesel::delete(
            job_executions::table.filter(
                job_executions::started_at
                    .lt(cutoff)
                    .and(job_executions::status.ne(JobStatus::Running)),
            ),
        )
        .execute(&mut conn)
        .await
        .map_err(AppError::from)
    }
}
