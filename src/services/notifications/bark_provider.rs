//! Bark notification provider implementation.
//!
//! Sends push notifications to iOS devices via Bark server API.
//! Uses the global `HTTP_CLIENT` for connection pooling and efficiency.
//!
//! Bark API Reference: https://github.com/Finb/Bark

use super::provider::{NotificationMessage, NotificationProvider, NotificationResult};
use crate::error::{AppError, AppResult};
use crate::external::client::HTTP_CLIENT;
use crate::models::BarkConfig;
use async_trait::async_trait;
use reqwest::Url;
use serde_json::json;
use std::time::Instant;

/// Bark notification provider
///
/// Sends push notifications to iOS devices through a Bark server.
/// Bark is an open-source iOS push notification service that supports
/// custom icons, sounds, deep links, and more.
///
/// # Example
/// ```ignore
/// let config = BarkConfig {
///     server_url: "https://bark.example.com".to_string(),
///     device_key: "YourDeviceKey".to_string(),
///     icon: Some("https://example.com/icon.png".to_string()),
///     sound: Some("notification.wav".to_string()),
///     level: Some("active".to_string()),
///     url: None,
///     group: None,
///     auto_copy: 0,
///     is_archive: 0,
/// };
/// let provider = BarkProvider::new(config);
/// let result = provider.send(&message).await?;
/// ```
#[derive(Clone)]
pub struct BarkProvider {
    config: BarkConfig,
}

impl BarkProvider {
    /// Creates a new bark provider with configuration
    ///
    /// # Arguments
    /// * `config` - Bark configuration (server_url, device_key, optional settings)
    pub fn new(config: BarkConfig) -> Self {
        Self { config }
    }

    /// Validates the Bark server URL
    ///
    /// # Returns
    /// Ok(()) if URL is valid HTTPS URL
    fn validate_server_url(&self) -> Result<(), AppError> {
        let url = Url::parse(&self.config.server_url).map_err(|_| AppError::Validation {
            field: "server_url".to_string(),
            reason: "Invalid URL format".to_string(),
        })?;

        if url.scheme() != "https" && url.scheme() != "http" {
            return Err(AppError::Validation {
                field: "server_url".to_string(),
                reason: "URL must use http or https protocol".to_string(),
            });
        }

        Ok(())
    }

    /// Builds the request body for Bark API
    ///
    /// # Arguments
    /// * `message` - The notification message to send
    ///
    /// # Returns
    /// JSON object for the Bark push API request body
    fn build_request_body(&self, message: &NotificationMessage) -> serde_json::Value {
        let mut body = json!({
            "title": message.title.clone().unwrap_or_else(|| "Notification".to_string()),
            "body": message.body,
        });

        // Add optional fields
        if let Some(icon) = &self.config.icon {
            body["icon"] = json!(icon);
        }

        if let Some(sound) = &self.config.sound {
            body["sound"] = json!(sound);
        }

        if let Some(level) = &self.config.level {
            body["level"] = json!(level);
        }

        if let Some(url) = &self.config.url {
            body["url"] = json!(url);
        }

        if let Some(group) = &self.config.group {
            body["group"] = json!(group);
        }

        if self.config.auto_copy > 0 {
            body["autoCopy"] = json!(self.config.auto_copy);
        }

        if self.config.is_archive > 0 {
            body["isArchive"] = json!(self.config.is_archive);
        }

        // Add metadata as custom parameters if present
        if !message.metadata.is_empty() {
            for (key, value) in &message.metadata {
                body[key] = json!(value);
            }
        }

        body
    }
}

#[async_trait]
impl NotificationProvider for BarkProvider {
    /// Sends a notification via Bark
    ///
    /// Constructs an HTTP POST request to the Bark push API endpoint
    /// with the notification title, body, and optional parameters.
    ///
    /// # Arguments
    /// * `message` - The notification message to send
    ///
    /// # Returns
    /// NotificationResult with success status, HTTP status code, response body, and duration
    async fn send(&self, message: &NotificationMessage) -> AppResult<NotificationResult> {
        let start = Instant::now();

        // Build request body
        let request_body = self.build_request_body(message);

        // Build URL
        let api_url = self.config.build_api_url();

        // Send request
        let response = HTTP_CLIENT
            .post(&api_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await;

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
            Err(e) => Ok(NotificationResult {
                success: false,
                status_code: None,
                response: Some(e.to_string()),
                duration_ms,
            }),
        }
    }

    fn name(&self) -> &'static str {
        "bark"
    }

    /// Validates bark configuration
    ///
    /// Checks that:
    /// - server_url is a valid URL
    /// - device_key is not empty
    ///
    /// # Returns
    /// Ok(()) if valid, Err with validation details otherwise
    async fn validate_config(&self) -> AppResult<()> {
        // Validate server URL
        self.validate_server_url()?;

        // Validate device key
        if self.config.device_key.is_empty() {
            return Err(AppError::Validation {
                field: "device_key".to_string(),
                reason: "Device key cannot be empty".to_string(),
            });
        }

        // Validate level if provided
        if let Some(level) = &self.config.level {
            match level.as_str() {
                "passive" | "active" | "timeSensitive" => {}
                _ => {
                    return Err(AppError::Validation {
                        field: "level".to_string(),
                        reason: "Level must be one of: passive, active, timeSensitive".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_api_url() {
        let config = BarkConfig {
            server_url: "https://bark.example.com".to_string(),
            device_key: "test_key_123".to_string(),
            icon: None,
            sound: None,
            level: None,
            url: None,
            group: None,
            auto_copy: 0,
            is_archive: 0,
        };

        let provider = BarkProvider::new(config);
        assert_eq!(
            provider.config.build_api_url(),
            "https://bark.example.com/push/test_key_123"
        );
    }

    #[test]
    fn test_build_api_url_trailing_slash() {
        let config = BarkConfig {
            server_url: "https://bark.example.com/".to_string(),
            device_key: "test_key_123".to_string(),
            icon: None,
            sound: None,
            level: None,
            url: None,
            group: None,
            auto_copy: 0,
            is_archive: 0,
        };

        let provider = BarkProvider::new(config);
        assert_eq!(
            provider.config.build_api_url(),
            "https://bark.example.com/push/test_key_123"
        );
    }

    #[test]
    fn test_validate_server_url_valid() {
        let config = BarkConfig {
            server_url: "https://bark.example.com".to_string(),
            device_key: "test_key".to_string(),
            icon: None,
            sound: None,
            level: None,
            url: None,
            group: None,
            auto_copy: 0,
            is_archive: 0,
        };

        let provider = BarkProvider::new(config);
        assert!(provider.validate_server_url().is_ok());
    }

    #[test]
    fn test_validate_server_url_invalid() {
        let config = BarkConfig {
            server_url: "not-a-url".to_string(),
            device_key: "test_key".to_string(),
            icon: None,
            sound: None,
            level: None,
            url: None,
            group: None,
            auto_copy: 0,
            is_archive: 0,
        };

        let provider = BarkProvider::new(config);
        assert!(provider.validate_server_url().is_err());
    }

    #[tokio::test]
    async fn test_build_request_body_minimal() {
        let config = BarkConfig {
            server_url: "https://bark.example.com".to_string(),
            device_key: "test_key".to_string(),
            icon: None,
            sound: None,
            level: None,
            url: None,
            group: None,
            auto_copy: 0,
            is_archive: 0,
        };

        let provider = BarkProvider::new(config);

        let message = NotificationMessage {
            title: Some("Test Title".to_string()),
            body: "Test Body".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let body = provider.build_request_body(&message);
        assert_eq!(body["title"], "Test Title");
        assert_eq!(body["body"], "Test Body");
        assert!(body.get("icon").is_none());
        assert!(body.get("sound").is_none());
    }

    #[tokio::test]
    async fn test_build_request_body_full() {
        let config = BarkConfig {
            server_url: "https://bark.example.com".to_string(),
            device_key: "test_key".to_string(),
            icon: Some("https://example.com/icon.png".to_string()),
            sound: Some("notification.wav".to_string()),
            level: Some("active".to_string()),
            url: Some("https://example.com/deep-link".to_string()),
            group: Some("app-notifications".to_string()),
            auto_copy: 1,
            is_archive: 1,
        };

        let provider = BarkProvider::new(config);

        let message = NotificationMessage {
            title: Some("Test Title".to_string()),
            body: "Test Body".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let body = provider.build_request_body(&message);
        assert_eq!(body["title"], "Test Title");
        assert_eq!(body["body"], "Test Body");
        assert_eq!(body["icon"], "https://example.com/icon.png");
        assert_eq!(body["sound"], "notification.wav");
        assert_eq!(body["level"], "active");
        assert_eq!(body["url"], "https://example.com/deep-link");
        assert_eq!(body["group"], "app-notifications");
        assert_eq!(body["autoCopy"], 1);
        assert_eq!(body["isArchive"], 1);
    }
}
