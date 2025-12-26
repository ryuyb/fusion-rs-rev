//! Command handlers for CLI operations
//!
//! This module contains handlers for different CLI commands,
//! separating command execution logic from parsing and validation.

pub mod serve;
pub mod migrate;

pub use serve::ServeCommandHandler;
pub use migrate::MigrateCommandHandler;
