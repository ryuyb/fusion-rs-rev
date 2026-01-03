//! Core notification provider trait and types.
//!
//! This module provides the abstraction for notification providers,
//! allowing easy extension to support different notification channels.

use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message to be sent via notification provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Message title/subject (optional for some providers)
    pub title: Option<String>,
    /// Message body/content (required)
    pub body: String,
    /// Additional metadata for provider-specific data
    pub metadata: HashMap<String, String>,
}

/// Result of a notification send attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    /// Whether send was successful
    pub success: bool,
    /// HTTP status code or provider-specific status
    pub status_code: Option<u16>,
    /// Response body or error message
    pub response: Option<String>,
    /// Time taken for the operation in milliseconds
    pub duration_ms: u64,
}

/// Trait for notification providers (email, webhook, SMS, etc.)
///
/// Uses `async_trait` to support async methods with dynamic dispatch.
/// All providers must be Send + Sync for use in async contexts.
///
/// # Example Implementation
/// ```ignore
/// use async_trait::async_trait;
///
/// pub struct WebhookProvider {
///     config: WebhookConfig,
/// }
///
/// #[async_trait]
/// impl NotificationProvider for WebhookProvider {
///     async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult> {
///         // Implementation here
///     }
///
///     fn name(&self) -> &'static str {
///         "webhook"
///     }
/// }
/// ```
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Sends a notification message
    ///
    /// # Arguments
    /// * `message` - The notification message to send
    ///
    /// # Returns
    /// Result containing send outcome details (success, status, duration, etc.)
    async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult>;

    /// Returns the provider name for logging/debugging
    ///
    /// # Returns
    /// Static string identifying the provider (e.g., "webhook", "email")
    fn name(&self) -> &'static str;

    /// Validates provider configuration (optional, default no-op)
    ///
    /// Override this method to perform config validation before saving.
    ///
    /// # Returns
    /// Ok(()) if configuration is valid, Err otherwise
    async fn validate_config(&self) -> AppResult<()> {
        Ok(())
    }
}
