use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::jobs::models::{NewScheduledJob, ScheduledJob, UpdateScheduledJob};
use crate::jobs::types::JobStatus;
use crate::schema::scheduled_jobs;

#[derive(Clone)]
pub struct JobRepository {
    pool: AsyncDbPool,
}

impl JobRepository {
    pub fn new(pool: AsyncDbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, job: NewScheduledJob) -> AppResult<ScheduledJob> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::insert_into(scheduled_jobs::table)
            .values(&job)
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    pub async fn get_by_id(&self, id: i32) -> AppResult<ScheduledJob> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        scheduled_jobs::table
            .find(id)
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound {
                    entity: "Job".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                },
                _ => AppError::from(e),
            })
    }

    pub async fn get_by_name(&self, name: &str) -> AppResult<ScheduledJob> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        scheduled_jobs::table
            .filter(scheduled_jobs::job_name.eq(name))
            .first(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound {
                    entity: "Job".to_string(),
                    field: "name".to_string(),
                    value: name.to_string(),
                },
                _ => AppError::from(e),
            })
    }

    pub async fn get_enabled_jobs(&self) -> AppResult<Vec<ScheduledJob>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        scheduled_jobs::table
            .filter(scheduled_jobs::enabled.eq(true))
            .load(&mut conn)
            .await
            .map_err(AppError::from)
    }

    pub async fn update(&self, id: i32, update: UpdateScheduledJob) -> AppResult<ScheduledJob> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::update(scheduled_jobs::table.find(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound {
                    entity: "Job".to_string(),
                    field: "id".to_string(),
                    value: id.to_string(),
                },
                _ => AppError::from(e),
            })
    }

    pub async fn delete(&self, id: i32) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        let deleted = diesel::delete(scheduled_jobs::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(AppError::from)?;

        if deleted == 0 {
            Err(AppError::NotFound {
                entity: "Job".to_string(),
                field: "id".to_string(),
                value: id.to_string(),
            })
        } else {
            Ok(())
        }
    }

    pub async fn update_last_run(&self, id: i32, status: JobStatus) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::update(scheduled_jobs::table.find(id))
            .set((
                scheduled_jobs::last_run_at.eq(diesel::dsl::now),
                scheduled_jobs::last_run_status.eq(status),
            ))
            .execute(&mut conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
}
