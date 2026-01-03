//! Notification-related DTOs for API requests and responses.

use crate::models::{ChannelType, NotificationChannel, NotificationLog, NotificationStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// Channel DTOs
// ============================================================================

/// Request to create a notification channel
#[derive(Debug, Deserialize, ToSchema, Validate)]
#[schema(example = json!({
    "channel_type": "webhook",
    "name": "My Webhook",
    "config": {
        "url": "https://webhook.site/unique-id",
        "method": "POST",
        "headers": {
            "Content-Type": "application/json",
            "X-Api-Key": "your-api-key"
        },
        "timeout_seconds": 30
    },
    "enabled": true,
    "priority": 10
}))]
pub struct CreateChannelRequest {
    /// Type of notification channel (webhook, email, sms, discord, slack)
    pub channel_type: ChannelType,

    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    /// Channel name (1-255 characters)
    pub name: String,

    /// Channel-specific configuration as JSON object.
    /// For webhook: {"url": "https://...", "method": "POST", "headers": {...}, "timeout_seconds": 30}
    #[schema(value_type = Object, example = json!({
        "url": "https://webhook.site/unique-id",
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        },
        "timeout_seconds": 30
    }))]
    pub config: JsonValue,

    #[serde(default = "default_true")]
    /// Whether the channel is enabled
    pub enabled: bool,

    #[serde(default)]
    /// Priority for channel ordering (higher = sent first)
    pub priority: i32,
}

fn default_true() -> bool {
    true
}

/// Request to update a notification channel
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    /// Optional new name for the channel
    pub name: Option<String>,

    /// Optional new configuration (same format as create)
    #[schema(value_type = Option<Object>)]
    pub config: Option<JsonValue>,

    /// Optional enabled status
    pub enabled: Option<bool>,

    /// Optional new priority
    pub priority: Option<i32>,
}

/// Response for notification channel
#[derive(Debug, Serialize, ToSchema)]
pub struct ChannelResponse {
    pub id: i32,
    pub user_id: i32,
    pub channel_type: ChannelType,
    pub name: String,
    pub config: JsonValue,
    pub enabled: bool,
    pub priority: i32,
    #[schema(example = "2024-01-15T10:30:00.000Z")]
    pub created_at: String,
    #[schema(example = "2024-01-20T14:45:30.000Z")]
    pub updated_at: String,
}

impl From<NotificationChannel> for ChannelResponse {
    fn from(channel: NotificationChannel) -> Self {
        Self {
            id: channel.id,
            user_id: channel.user_id,
            channel_type: channel.channel_type,
            name: channel.name,
            config: channel.config,
            enabled: channel.enabled,
            priority: channel.priority,
            created_at: channel
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            updated_at: channel
                .updated_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}

// ============================================================================
// Message DTOs
// ============================================================================

/// Request to send a notification
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SendNotificationRequest {
    #[validate(length(max = 255))]
    #[schema(example = "Alert")]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "Body is required"))]
    #[schema(example = "This is a notification message")]
    pub body: String,

    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Request to send notification to user's channels
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SendToUserRequest {
    #[schema(example = "Webhook")]
    pub channel_type: ChannelType,

    #[validate(length(max = 255))]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "Body is required"))]
    pub body: String,

    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Log DTOs
// ============================================================================

/// Response for notification log
#[derive(Debug, Serialize, ToSchema)]
pub struct LogResponse {
    pub id: i64,
    pub channel_id: i32,
    pub message: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: i32,
    #[schema(example = "2024-01-20T14:45:30.000Z")]
    pub sent_at: String,
}

impl From<NotificationLog> for LogResponse {
    fn from(log: NotificationLog) -> Self {
        Self {
            id: log.id,
            channel_id: log.channel_id,
            message: log.message,
            status: log.status,
            error_message: log.error_message,
            retry_count: log.retry_count,
            sent_at: log.sent_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}
