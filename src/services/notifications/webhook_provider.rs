//! Webhook notification provider implementation.
//!
//! Sends HTTP requests to configured webhook URLs using the global HTTP_CLIENT.

use super::provider::{NotificationMessage, NotificationProvider, NotificationResult};
use crate::error::{AppError, AppResult};
use crate::external::client::HTTP_CLIENT;
use crate::models::WebhookConfig;
use async_trait::async_trait;
use reqwest::{Method, Url};
use serde_json::json;
use std::time::Instant;

/// Webhook notification provider
///
/// Sends HTTP requests to configured webhook URLs.
/// Uses the global `HTTP_CLIENT` for connection pooling and efficiency.
///
/// # Example
/// ```ignore
/// let config = WebhookConfig {
///     url: "https://example.com/webhook".to_string(),
///     method: "POST".to_string(),
///     headers: HashMap::new(),
///     timeout_seconds: 30,
/// };
/// let provider = WebhookProvider::new(config);
/// let result = provider.send(&message).await?;
/// ```
pub struct WebhookProvider {
    config: WebhookConfig,
}

impl WebhookProvider {
    /// Creates a new webhook provider with configuration
    ///
    /// # Arguments
    /// * `config` - Webhook configuration (URL, method, headers, timeout)
    pub fn new(config: WebhookConfig) -> Self {
        Self { config }
    }

    /// Parses HTTP method string into reqwest Method
    ///
    /// # Returns
    /// Result containing the parsed Method or validation error
    fn parse_method(&self) -> Result<Method, AppError> {
        self.config
            .method
            .parse()
            .map_err(|_| AppError::Validation {
                field: "method".to_string(),
                reason: format!("Invalid HTTP method: {}", self.config.method),
            })
    }
}

#[async_trait]
impl NotificationProvider for WebhookProvider {
    /// Sends a notification via webhook
    ///
    /// Constructs an HTTP request with the configured method, headers, and timeout,
    /// then sends the notification message as JSON in the request body.
    ///
    /// # Arguments
    /// * `message` - The notification message to send
    ///
    /// # Returns
    /// NotificationResult with success status, HTTP status code, response body, and duration
    async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult> {
        let start = Instant::now();

        // Build request
        let method = self.parse_method()?;
        let mut request = HTTP_CLIENT
            .request(method, &self.config.url)
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .json(&json!({
                "title": message.title,
                "body": message.body,
                "metadata": message.metadata,
            }));

        // Add custom headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Send request
        let response = request.send().await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let success = resp.status().is_success();
                let response_text = resp.text().await.ok();

                Ok(NotificationResult {
                    success,
                    status_code: Some(status_code),
                    response: response_text,
                    duration_ms,
                })
            }
            Err(e) => {
                // Log error but return NotificationResult (don't fail the operation)
                // This allows the log to be saved with error details
                Ok(NotificationResult {
                    success: false,
                    status_code: None,
                    response: Some(e.to_string()),
                    duration_ms,
                })
            }
        }
    }

    fn name(&self) -> &'static str {
        "webhook"
    }

    /// Validates webhook configuration
    ///
    /// Checks that:
    /// - URL starts with http:// or https://
    /// - HTTP method is valid
    ///
    /// # Returns
    /// Ok(()) if valid, Err with validation details otherwise
    async fn validate_config(&self) -> AppResult<()> {
        let url = Url::parse(&self.config.url).map_err(|_| AppError::Validation {
            field: "url".to_string(),
            reason: "Invalid URL format".to_string(),
        })?;

        if url.scheme() != "https" {
            return Err(AppError::Validation {
                field: "url".to_string(),
                reason: "Only HTTPS URLs are allowed".to_string(),
            });
        }

        // Validate method
        self.parse_method()?;

        Ok(())
    }
}
