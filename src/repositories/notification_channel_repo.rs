//! Notification channel repository for async database operations.
//!
//! Provides CRUD operations for notification_channels table.

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::AsyncDbPool;
use crate::error::{AppError, AppResult};
use crate::models::{
    ChannelType, NewNotificationChannel, NotificationChannel, UpdateNotificationChannel,
};

/// Notification channel repository
#[derive(Clone)]
pub struct NotificationChannelRepository {
    pool: AsyncDbPool,
}

impl NotificationChannelRepository {
    /// Creates a new NotificationChannelRepository with the given connection pool.
    pub fn new(pool: AsyncDbPool) -> Self {
        Self { pool }
    }

    /// Creates a new notification channel
    ///
    /// # Arguments
    /// * `new_channel` - The channel data to insert
    ///
    /// # Returns
    /// The created notification channel with generated id and timestamps
    pub async fn create(
        &self,
        new_channel: NewNotificationChannel,
    ) -> AppResult<NotificationChannel> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::insert_into(notification_channels)
            .values(&new_channel)
            .returning(NotificationChannel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Finds a channel by ID
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to find
    ///
    /// # Returns
    /// Some(NotificationChannel) if found, None otherwise
    pub async fn find_by_id(&self, channel_id: i32) -> AppResult<Option<NotificationChannel>> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        notification_channels
            .filter(id.eq(channel_id))
            .select(NotificationChannel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(AppError::from)
    }

    /// Lists all channels for a user ordered by priority
    ///
    /// # Arguments
    /// * `uid` - The user ID
    ///
    /// # Returns
    /// Vector of notification channels ordered by priority (highest first)
    pub async fn find_by_user_id(&self, uid: i32) -> AppResult<Vec<NotificationChannel>> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        notification_channels
            .filter(user_id.eq(uid))
            .order(priority.desc())
            .select(NotificationChannel::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Lists enabled channels for a user by type, ordered by priority
    ///
    /// # Arguments
    /// * `uid` - The user ID
    /// * `ctype` - The channel type to filter by
    ///
    /// # Returns
    /// Vector of enabled notification channels of the specified type, ordered by priority
    pub async fn find_enabled_by_type(
        &self,
        uid: i32,
        ctype: ChannelType,
    ) -> AppResult<Vec<NotificationChannel>> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        notification_channels
            .filter(user_id.eq(uid))
            .filter(channel_type.eq(ctype))
            .filter(enabled.eq(true))
            .order(priority.desc())
            .select(NotificationChannel::as_select())
            .load(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Updates a notification channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to update
    /// * `update_data` - The update data
    ///
    /// # Returns
    /// The updated notification channel
    pub async fn update(
        &self,
        channel_id: i32,
        update_data: UpdateNotificationChannel,
    ) -> AppResult<NotificationChannel> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::update(notification_channels.filter(id.eq(channel_id)))
            .set(&update_data)
            .returning(NotificationChannel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(AppError::from)
    }

    /// Deletes a notification channel
    ///
    /// # Arguments
    /// * `channel_id` - The ID of the channel to delete
    ///
    /// # Returns
    /// Number of rows affected (1 if deleted, 0 if not found)
    pub async fn delete(&self, channel_id: i32) -> AppResult<usize> {
        use crate::schema::notification_channels::dsl::*;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::ConnectionPool {
                source: anyhow::Error::from(e),
            })?;

        diesel::delete(notification_channels.filter(id.eq(channel_id)))
            .execute(&mut conn)
            .await
            .map_err(AppError::from)
    }
}
