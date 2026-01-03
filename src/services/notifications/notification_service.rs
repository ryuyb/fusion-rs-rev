//! Notification service for business logic.
//!
//! Provides notification channel management and message sending functionality.

use super::provider::{NotificationMessage, NotificationProvider};
use super::webhook_provider::WebhookProvider;
use crate::error::{AppError, AppResult};
use crate::models::{
    ChannelType, NewNotificationChannel, NewNotificationLog, NotificationChannel, NotificationLog,
    NotificationStatus, UpdateNotificationChannel, WebhookConfig,
};
use crate::repositories::{NotificationChannelRepository, NotificationLogRepository};
use std::sync::Arc;

/// Notification service handling channel management and message sending
#[derive(Clone)]
pub struct NotificationService {
    channel_repo: NotificationChannelRepository,
    log_repo: NotificationLogRepository,
}

impl NotificationService {
    /// Creates a new NotificationService
    ///
    /// # Arguments
    /// * `channel_repo` - Repository for notification channels
    /// * `log_repo` - Repository for notification logs
    pub fn new(
        channel_repo: NotificationChannelRepository,
        log_repo: NotificationLogRepository,
    ) -> Self {
        Self {
            channel_repo,
            log_repo,
        }
    }

    // ========================================================================
    // Channel Management
    // ========================================================================

    /// Creates a new notification channel
    ///
    /// Validates the channel configuration before creating.
    ///
    /// # Arguments
    /// * `new_channel` - The channel data to create
    ///
    /// # Returns
    /// The created channel with generated id and timestamps
    pub async fn create_channel(
        &self,
        new_channel: NewNotificationChannel,
    ) -> AppResult<NotificationChannel> {
        // Validate config based on channel type
        self.validate_channel_config(&new_channel.channel_type, &new_channel.config)?;

        self.channel_repo.create(new_channel).await
    }

    /// Gets a channel by ID
    ///
    /// # Arguments
    /// * `id` - The channel ID
    ///
    /// # Returns
    /// The channel if found, NotFound error otherwise
    pub async fn get_channel(&self, id: i32) -> AppResult<NotificationChannel> {
        self.channel_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound {
                entity: "notification_channel".to_string(),
                field: "id".to_string(),
                value: id.to_string(),
            })
    }

    /// Lists all channels for a user
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    ///
    /// # Returns
    /// Vector of channels ordered by priority (highest first)
    pub async fn list_user_channels(&self, user_id: i32) -> AppResult<Vec<NotificationChannel>> {
        self.channel_repo.find_by_user_id(user_id).await
    }

    /// Updates a notification channel
    ///
    /// Verifies the channel exists and validates config if being updated.
    ///
    /// # Arguments
    /// * `id` - The channel ID to update
    /// * `update_data` - The update data
    ///
    /// # Returns
    /// The updated channel
    pub async fn update_channel(
        &self,
        id: i32,
        update_data: UpdateNotificationChannel,
    ) -> AppResult<NotificationChannel> {
        // Verify channel exists
        let channel = self.get_channel(id).await?;

        // Validate config if being updated
        if let Some(ref config) = update_data.config {
            self.validate_channel_config(&channel.channel_type, config)?;
        }

        self.channel_repo.update(id, update_data).await
    }

    /// Deletes a notification channel
    ///
    /// # Arguments
    /// * `id` - The channel ID to delete
    ///
    /// # Returns
    /// true if deleted, false if not found
    pub async fn delete_channel(&self, id: i32) -> AppResult<bool> {
        let affected = self.channel_repo.delete(id).await?;
        Ok(affected > 0)
    }

    // ========================================================================
    // Message Sending
    // ========================================================================

    /// Sends a notification to a specific channel
    ///
    /// Logs the send attempt to notification_logs table.
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID to send via
    /// * `message` - The notification message
    ///
    /// # Returns
    /// The log entry for this send attempt
    pub async fn send_to_channel(
        &self,
        channel_id: i32,
        message: NotificationMessage,
    ) -> AppResult<NotificationLog> {
        let channel = self.get_channel(channel_id).await?;

        if !channel.enabled {
            return Err(AppError::BadRequest {
                message: format!("Channel {} is disabled", channel_id),
            });
        }

        // Create provider
        let provider = self.create_provider(&channel)?;

        // Send notification
        let result = provider.send(&message).await?;

        // Log the result
        let log_entry = NewNotificationLog {
            channel_id,
            message: serde_json::to_string(&message).unwrap_or_default(),
            status: if result.success {
                NotificationStatus::Sent
            } else {
                NotificationStatus::Failed
            },
            error_message: if !result.success {
                result.response.clone()
            } else {
                None
            },
            retry_count: 0,
        };

        self.log_repo.create(log_entry).await
    }

    /// Sends a notification to all enabled channels of a specific type for a user
    ///
    /// Sends to channels in priority order (highest first).
    ///
    /// # Arguments
    /// * `user_id` - The user ID
    /// * `channel_type` - The channel type to send to
    /// * `message` - The notification message
    ///
    /// # Returns
    /// List of log entries for each send attempt
    pub async fn send_to_user(
        &self,
        user_id: i32,
        channel_type: ChannelType,
        message: NotificationMessage,
    ) -> AppResult<Vec<NotificationLog>> {
        let channels = self
            .channel_repo
            .find_enabled_by_type(user_id, channel_type)
            .await?;

        if channels.is_empty() {
            return Err(AppError::NotFound {
                entity: "notification_channel".to_string(),
                field: "user_id,channel_type,enabled".to_string(),
                value: format!("{},{:?},true", user_id, channel_type),
            });
        }

        let mut logs = Vec::new();
        for channel in channels {
            let log = self.send_to_channel(channel.id, message.clone()).await?;
            logs.push(log);
        }

        Ok(logs)
    }

    // ========================================================================
    // Log Queries
    // ========================================================================

    /// Gets logs for a specific channel
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `offset` - Number of records to skip (for pagination)
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// Tuple of (logs vector, total count)
    pub async fn get_channel_logs(
        &self,
        channel_id: i32,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<NotificationLog>, i64)> {
        self.log_repo
            .find_by_channel_id(channel_id, offset, limit)
            .await
    }

    /// Gets logs by status
    ///
    /// # Arguments
    /// * `status` - The status to filter by
    /// * `offset` - Number of records to skip (for pagination)
    /// * `limit` - Maximum number of records to return
    ///
    /// # Returns
    /// Tuple of (logs vector, total count)
    pub async fn get_logs_by_status(
        &self,
        status: NotificationStatus,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<NotificationLog>, i64)> {
        self.log_repo.find_by_status(status, offset, limit).await
    }

    // ========================================================================
    // Private Helpers
    // ========================================================================

    /// Creates a provider instance from channel configuration
    ///
    /// Factory method pattern - returns Arc<dyn NotificationProvider> for
    /// dynamic dispatch.
    ///
    /// # Arguments
    /// * `channel` - The notification channel
    ///
    /// # Returns
    /// Arc-wrapped provider implementing NotificationProvider trait
    fn create_provider(
        &self,
        channel: &NotificationChannel,
    ) -> AppResult<Arc<dyn NotificationProvider>> {
        match channel.channel_type {
            ChannelType::Webhook => {
                let config = WebhookConfig::from_json(&channel.config).map_err(|e| {
                    AppError::Validation {
                        field: "config".to_string(),
                        reason: format!("Invalid webhook config: {}", e),
                    }
                })?;
                Ok(Arc::new(WebhookProvider::new(config)))
            }
            // Future providers:
            // ChannelType::Email => { ... }
            // ChannelType::Sms => { ... }
            _ => Err(AppError::BadRequest {
                message: format!("Unsupported channel type: {:?}", channel.channel_type),
            }),
        }
    }

    /// Validates channel configuration based on type
    ///
    /// # Arguments
    /// * `channel_type` - The type of channel
    /// * `config` - The configuration JSONB value
    ///
    /// # Returns
    /// Ok(()) if valid, Err with validation details otherwise
    fn validate_channel_config(
        &self,
        channel_type: &ChannelType,
        config: &serde_json::Value,
    ) -> AppResult<()> {
        match channel_type {
            ChannelType::Webhook => {
                WebhookConfig::from_json(config).map_err(|e| AppError::Validation {
                    field: "config".to_string(),
                    reason: format!("Invalid webhook config: {}", e),
                })?;
            }
            // Future validations:
            // ChannelType::Email => { ... }
            _ => {
                return Err(AppError::BadRequest {
                    message: format!("Unsupported channel type: {:?}", channel_type),
                });
            }
        }
        Ok(())
    }
}
