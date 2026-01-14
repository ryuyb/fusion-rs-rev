//! Notification models for database operations.
//!
//! This module provides data models for the notification system including
//! notification channels and logs.

use diesel::prelude::*;
use jiff_diesel::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

/// Channel type for notifications
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    diesel_derive_enum::DbEnum,
)]
#[db_enum(existing_type_path = "crate::schema::sql_types::ChannelType")]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Webhook,
    Email,
    Sms,
    Discord,
    Slack,
    Bark,
}

/// Status of a notification log entry
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    diesel_derive_enum::DbEnum,
)]
#[db_enum(existing_type_path = "crate::schema::sql_types::NotificationStatus")]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

// ============================================================================
// NotificationChannel Models (Query/Insert/Update)
// ============================================================================

/// NotificationChannel query model for SELECT operations
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::notification_channels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationChannel {
    pub id: i32,
    pub user_id: i32,
    pub channel_type: ChannelType,
    pub name: String,
    pub config: JsonValue,
    pub enabled: bool,
    pub priority: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// NewNotificationChannel insert model for INSERT operations
#[derive(Debug, Insertable, Deserialize, Clone)]
#[diesel(table_name = crate::schema::notification_channels)]
pub struct NewNotificationChannel {
    pub user_id: i32,
    pub channel_type: ChannelType,
    pub name: String,
    pub config: JsonValue,
    pub enabled: bool,
    pub priority: i32,
}

/// UpdateNotificationChannel model for UPDATE operations
#[derive(Debug, AsChangeset, Deserialize, Clone, Default)]
#[diesel(table_name = crate::schema::notification_channels)]
pub struct UpdateNotificationChannel {
    pub name: Option<String>,
    pub config: Option<JsonValue>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
}

// ============================================================================
// NotificationLog Models (Query/Insert/Update)
// ============================================================================

/// NotificationLog query model for SELECT operations
#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::notification_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationLog {
    pub id: i64,
    pub channel_id: i32,
    pub message: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub sent_at: DateTime,
}

/// NewNotificationLog insert model for INSERT operations
#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::notification_logs)]
pub struct NewNotificationLog {
    pub channel_id: i32,
    pub message: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: i32,
}

// ============================================================================
// Config Type-Safe Helpers
// ============================================================================

/// Webhook-specific configuration
///
/// This struct provides type-safe parsing and serialization of webhook
/// configuration stored as JSONB in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String, // "POST", "PUT", etc.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_timeout() -> u64 {
    30
}

impl WebhookConfig {
    /// Parse JSONB config into typed WebhookConfig
    ///
    /// # Arguments
    /// * `config` - The JSONB value from the database
    ///
    /// # Returns
    /// Result containing the parsed config or deserialization error
    ///
    /// # Example
    /// ```ignore
    /// let config = WebhookConfig::from_json(&channel.config)?;
    /// ```
    pub fn from_json(config: &JsonValue) -> Result<Self, serde_json::Error> {
        serde_json::from_value(config.clone())
    }

    /// Convert to JSONB for database storage
    ///
    /// # Returns
    /// Result containing the JSONB value or serialization error
    ///
    /// # Example
    /// ```ignore
    /// let json_value = webhook_config.to_json()?;
    /// ```
    pub fn to_json(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::to_value(self)
    }
}

// ============================================================================
// Bark Config
// ============================================================================

/// Bark-specific notification configuration
///
/// Bark is an iOS push notification service that supports custom icons, sounds,
/// and interapp navigation. This config stores the Bark server URL and device key.
///
/// # Example JSON Config
/// ```json
/// {
///     "device_key": "YourDeviceKey",
///     "icon": "https://example.com/icon.png",
///     "sound": "notification.wav",
///     "level": "timeSensitive",
///     "url": "https://example.com/deep-link",
///     "group": "app-notifications",
///     "auto_copy": 1,
///     "is_archive": 1
/// }
/// ```
///
/// Note: `server_url` is optional and defaults to `https://api.day.app`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarkConfig {
    /// Bark server base URL (optional, defaults to "https://api.day.app")
    #[serde(default = "default_server_url")]
    pub server_url: String,

    /// Device key for authentication with Bark server
    pub device_key: String,

    /// Custom icon URL (optional, defaults to app icon)
    #[serde(default)]
    pub icon: Option<String>,

    /// Notification sound name (optional, defaults to system default)
    #[serde(default)]
    pub sound: Option<String>,

    /// Notification urgency level (optional)
    /// Values: "passive", "active", "timeSensitive"
    #[serde(default)]
    pub level: Option<String>,

    /// Deep link URL to open when notification is tapped (optional)
    #[serde(default)]
    pub url: Option<String>,

    /// Notification group/bundle identifier (optional)
    #[serde(default)]
    pub group: Option<String>,

    /// Automatically copy notification content to clipboard (optional)
    /// 1 = enabled, 0 = disabled
    #[serde(default = "default_auto_copy")]
    pub auto_copy: u8,

    /// Archive notification in Bark app (optional)
    /// 1 = enabled, 0 = disabled
    #[serde(default = "default_is_archive")]
    pub is_archive: u8,
}

fn default_server_url() -> String {
    "https://api.day.app".to_string()
}

fn default_auto_copy() -> u8 {
    0
}

fn default_is_archive() -> u8 {
    0
}

impl BarkConfig {
    /// Parse JSONB config into typed BarkConfig
    ///
    /// # Arguments
    /// * `config` - The JSONB value from the database
    ///
    /// # Returns
    /// Result containing the parsed config or deserialization error
    pub fn from_json(config: &JsonValue) -> Result<Self, serde_json::Error> {
        serde_json::from_value(config.clone())
    }

    /// Convert to JSONB for database storage
    ///
    /// # Returns
    /// Result containing the JSONB value or serialization error
    pub fn to_json(&self) -> Result<JsonValue, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Builds the full Bark API URL from server URL and device key
    ///
    /// # Returns
    /// Full URL for Bark push notification endpoint
    pub fn build_api_url(&self) -> String {
        format!(
            "{}/{}",
            self.server_url.trim_end_matches('/'),
            self.device_key
        )
    }
}
