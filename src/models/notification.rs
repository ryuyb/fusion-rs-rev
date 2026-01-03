//! Notification models for database operations.
//!
//! This module provides data models for the notification system including
//! notification channels and logs.

use chrono::NaiveDateTime;
use diesel::AsExpression;
use diesel::FromSqlRow;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::Write;

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
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Webhook,
    Email,
    Sms,
    Discord,
    Slack,
}

impl diesel::query_builder::QueryId for ChannelType {
    type QueryId = ChannelType;
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl ToSql<Text, Pg> for ChannelType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match self {
            ChannelType::Webhook => "webhook",
            ChannelType::Email => "email",
            ChannelType::Sms => "sms",
            ChannelType::Discord => "discord",
            ChannelType::Slack => "slack",
        };
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for ChannelType {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match s.as_str() {
            "webhook" => Ok(ChannelType::Webhook),
            "email" => Ok(ChannelType::Email),
            "sms" => Ok(ChannelType::Sms),
            "discord" => Ok(ChannelType::Discord),
            "slack" => Ok(ChannelType::Slack),
            _ => Err(format!("Unrecognized channel_type: {}", s).into()),
        }
    }
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
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

impl diesel::query_builder::QueryId for NotificationStatus {
    type QueryId = NotificationStatus;
    const HAS_STATIC_QUERY_ID: bool = false;
}

impl ToSql<Text, Pg> for NotificationStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match self {
            NotificationStatus::Pending => "pending",
            NotificationStatus::Sent => "sent",
            NotificationStatus::Failed => "failed",
            NotificationStatus::Retrying => "retrying",
        };
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for NotificationStatus {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match s.as_str() {
            "pending" => Ok(NotificationStatus::Pending),
            "sent" => Ok(NotificationStatus::Sent),
            "failed" => Ok(NotificationStatus::Failed),
            "retrying" => Ok(NotificationStatus::Retrying),
            _ => Err(format!("Unrecognized notification_status: {}", s).into()),
        }
    }
}

// ============================================================================
// NotificationChannel Models (Query/Insert/Update)
// ============================================================================

/// NotificationChannel query model for SELECT operations
#[derive(Debug, Queryable, Selectable, Serialize, Clone)]
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
#[derive(Debug, Queryable, Selectable, Serialize, Clone)]
#[diesel(table_name = crate::schema::notification_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationLog {
    pub id: i64,
    pub channel_id: i32,
    pub message: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub sent_at: NaiveDateTime,
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
