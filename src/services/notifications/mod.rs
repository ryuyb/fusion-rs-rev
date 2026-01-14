//! Notification system with pluggable providers.
//!
//! This module provides the notification system abstraction and implementations.
//! The core trait `NotificationProvider` allows for easy extension to support
//! different notification channels (webhook, email, SMS, etc.).

mod bark_provider;
mod provider;
mod webhook_provider;

pub mod notification_service;

pub use bark_provider::BarkProvider;
pub use notification_service::NotificationService;
pub use provider::{NotificationMessage, NotificationProvider, NotificationResult};
pub use webhook_provider::WebhookProvider;
