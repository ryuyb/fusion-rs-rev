//! Command handlers for CLI operations
//!
//! This module contains handlers for different CLI commands,
//! separating command execution logic from parsing and validation.

pub mod migrate;
pub mod serve;

pub use migrate::MigrateCommandHandler;
pub use serve::ServeCommandHandler;
