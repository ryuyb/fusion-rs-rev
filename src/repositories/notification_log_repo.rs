//! Notification log repository for async database operations.
//!
//! Provides operations for notification_logs table.

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::models::{NewNotificationLog, NotificationLog, NotificationStatus};

/// Notification log repository
#[derive(Clone)]
pub struct NotificationLogRepository {
    pool: AsyncDbPool,
}

impl NotificationLogRepository {
    /// Creates a new NotificationLogRepository with the given connection pool.
    pub fn new(pool: AsyncDbPool) -> Self {
        Self { pool }
    }

    /// Creates a new notification log entry
    ///
    /// # Arguments
    /// * `new_log` - The log data to insert
    ///
    /// # Returns
    /// The created notification log with generated id and timestamp
    pub async fn create(&self, new_log: NewNotificationLog) -> AppResult<NotificationLog> {
        use crate::schema::notification_logs::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::insert_into(notification_logs)
            .values(&new_log)
            .returning(NotificationLog::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Finds logs for a specific channel with pagination
    ///
    /// # Arguments
    /// * `cid` - The channel ID
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// Tuple of (logs vector, total count)
    pub async fn find_by_channel_id(
        &self,
        cid: i32,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<NotificationLog>, i64)> {
        use crate::schema::notification_logs::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        // Get paginated logs
        let logs = notification_logs
            .filter(channel_id.eq(cid))
            .order(sent_at.desc())
            .offset(offset)
            .limit(limit)
            .select(NotificationLog::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::from)?;

        // Get total count
        let total = notification_logs
            .filter(channel_id.eq(cid))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(AppError::from)?;

        Ok((logs, total))
    }

    /// Finds logs by status with pagination
    ///
    /// # Arguments
    /// * `status_filter` - The status to filter by
    /// * `offset` - Number of records to skip
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// Tuple of (logs vector, total count)
    pub async fn find_by_status(
        &self,
        status_filter: NotificationStatus,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<NotificationLog>, i64)> {
        use crate::schema::notification_logs::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        // Get paginated logs
        let logs = notification_logs
            .filter(status.eq(status_filter))
            .order(sent_at.desc())
            .offset(offset)
            .limit(limit)
            .select(NotificationLog::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::from)?;

        // Get total count
        let total = notification_logs
            .filter(status.eq(status_filter))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(AppError::from)?;

        Ok((logs, total))
    }
}
