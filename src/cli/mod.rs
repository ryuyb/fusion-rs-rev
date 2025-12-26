//! CLI module for fusion-rs
//!
//! This module provides command-line interface functionality including:
//! - Argument parsing with clap
//! - Configuration merging (CLI args + config files)
//! - Command execution and validation
//! - Command handlers for serve and migrate operations

pub mod parser;
pub mod validation;
pub mod config_merger;
pub mod handlers;
pub mod executor;

// Re-export public types for convenience
pub use parser::{Cli, Commands, Environment, LogLevel};
pub use config_merger::ConfigurationMerger;
pub use executor::execute_command;

use crate::config::settings::Settings;
use crate::logger::init_logger;

/// Load and merge configuration from CLI arguments
///
/// This function handles the complete configuration loading process:
/// 1. Load base configuration from files
/// 2. Merge CLI argument overrides
/// 3. Validate the final configuration
///
/// # Arguments
/// * `cli` - Parsed CLI arguments
///
/// # Returns
/// Merged and validated Settings
///
/// # Errors
/// Returns error if configuration loading, merging, or validation fails
pub fn load_and_merge_config(cli: &Cli) -> anyhow::Result<Settings> {
    let merger = ConfigurationMerger::from_config_path(cli.config.as_ref())
        .map_err(|e| {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    merger.merge_cli_args(cli).map_err(|e| {
        eprintln!("Configuration merge error: {}", e);
        std::process::exit(1);
    })
}

/// Initialize logger from settings
///
/// # Arguments
/// * `settings` - Application settings containing logger configuration
///
/// # Returns
/// Logger handle on success
///
/// # Errors
/// Returns error if logger initialization fails
pub fn init_logger_from_settings(settings: &Settings) -> anyhow::Result<crate::logger::LogLevelHandle> {
    let logger_config = settings.logger.clone().into_logger_config()
        .map_err(|e| {
            eprintln!("Logger configuration error: {}", e);
            std::process::exit(1);
        })?;
    
    init_logger(logger_config).map_err(|e| {
        eprintln!("Logger initialization error: {}", e);
        std::process::exit(1);
    })
}
