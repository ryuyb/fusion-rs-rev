//! Configuration management module for fusion-rs
//!
//! This module provides layered configuration loading with support for:
//! - TOML configuration files
//! - Environment variable overrides
//! - Multiple environment configurations (development, test, staging, production)
//!
//! # Configuration Priority (lowest to highest)
//! 1. `default.toml` - Base default configuration
//! 2. `{environment}.toml` - Environment-specific configuration
//! 3. `local.toml` - Local development overrides (not committed to version control)
//! 4. `FUSION_*` environment variables

pub mod environment;
pub mod error;
pub mod loader;
pub mod settings;
pub mod validation;

// Re-export public types
pub use environment::Environment;
pub use loader::ConfigLoader;
pub use settings::{
    DatabaseConfig,
    JwtConfig,
};
