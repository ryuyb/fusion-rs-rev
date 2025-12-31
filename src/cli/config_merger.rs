//! Configuration merger for CLI arguments and config files
//!
//! This module handles merging CLI argument overrides with file-based configuration,
//! implementing the configuration precedence logic.

use super::parser::{Cli, Commands};
use crate::config::error::ConfigError;
use crate::config::{ConfigLoader, settings::Settings};
use std::path::PathBuf;

/// Configuration merger that handles CLI argument integration with file-based configuration
///
/// This struct implements the configuration precedence logic where CLI arguments
/// override configuration file values.
pub struct ConfigurationMerger {
    base_config: Settings,
}

impl ConfigurationMerger {
    /// Create a new configuration merger with base configuration
    pub fn new(base_config: Settings) -> Self {
        Self { base_config }
    }

    /// Create a configuration merger by loading configuration from the specified path or default loader
    ///
    /// # Arguments
    /// * `config_path` - Optional path to configuration file. If None, uses default loader behavior
    ///
    /// # Errors
    /// Returns ConfigError if configuration loading or validation fails
    pub fn from_config_path(config_path: Option<&PathBuf>) -> Result<Self, ConfigError> {
        let config = if let Some(path) = config_path {
            // Validate file path accessibility (additional validation beyond clap)
            Self::validate_config_file_access(path)?;
            // Load configuration from specific file
            Self::load_config_from_file(path)?
        } else {
            // Use default configuration loader
            ConfigLoader::new()?.load()?
        };

        Ok(Self::new(config))
    }

    /// Validate that the configuration file is accessible and readable
    fn validate_config_file_access(path: &PathBuf) -> Result<(), ConfigError> {
        // Check if file exists
        if !path.exists() {
            return Err(ConfigError::ValidationError {
                field: "config_file".to_string(),
                message: format!("Configuration file does not exist: '{}'", path.display()),
            });
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Err(ConfigError::ValidationError {
                field: "config_file".to_string(),
                message: format!("Configuration path is not a file: '{}'", path.display()),
            });
        }

        // Check if file is readable
        match std::fs::File::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(ConfigError::ValidationError {
                field: "config_file".to_string(),
                message: format!("Cannot read configuration file '{}': {}", path.display(), e),
            }),
        }
    }

    /// Load configuration from a specific file path
    fn load_config_from_file(path: &PathBuf) -> Result<Settings, ConfigError> {
        // Set environment variable to use specific config file
        unsafe {
            std::env::set_var("FUSION_CONFIG_FILE", path);
        }

        // Create loader and load configuration
        let loader = ConfigLoader::new()?;
        let config = loader.load()?;

        // Clean up environment variable
        unsafe {
            std::env::remove_var("FUSION_CONFIG_FILE");
        }

        Ok(config)
    }

    /// Merge CLI arguments with the base configuration
    ///
    /// This method applies CLI argument overrides according to the precedence rules:
    /// 1. CLI arguments have highest priority
    /// 2. Configuration file values are used as base
    ///
    /// # Arguments
    /// * `cli` - Parsed CLI arguments
    ///
    /// # Returns
    /// A new Settings instance with CLI overrides applied
    pub fn merge_cli_args(&self, cli: &Cli) -> Result<Settings, ConfigError> {
        let mut config = self.base_config.clone();

        // Apply global CLI overrides
        self.apply_global_overrides(&mut config, cli)?;

        // Apply command-specific overrides
        if let Some(ref command) = cli.command {
            self.apply_command_overrides(&mut config, command)?;
        }

        // Validate the merged configuration
        config.validate()?;

        Ok(config)
    }

    /// Apply global CLI argument overrides
    fn apply_global_overrides(&self, config: &mut Settings, cli: &Cli) -> Result<(), ConfigError> {
        // Apply logging level overrides from global flags
        if cli.verbose {
            config.logger.level = "debug".to_string();
        } else if cli.quiet {
            config.logger.level = "error".to_string();
        }

        Ok(())
    }

    /// Apply command-specific CLI argument overrides
    fn apply_command_overrides(
        &self,
        config: &mut Settings,
        command: &Commands,
    ) -> Result<(), ConfigError> {
        match command {
            Commands::Serve {
                host,
                port,
                log_level,
                dry_run: _,
            } => {
                // Override server host if provided
                if let Some(host_addr) = host {
                    config.server.host = host_addr.clone();
                }

                // Override server port if provided
                if let Some(port_num) = port {
                    config.server.port = *port_num;
                }

                // Override log level if provided (command-specific override takes precedence over global)
                if let Some(level) = log_level {
                    config.logger.level = level.clone().into();
                }
            }
            Commands::Migrate {
                dry_run: _,
                rollback: _,
            } => {
                // Migration commands don't override server configuration
            }
        }

        Ok(())
    }

    /// Get the current configuration (useful for inspection)
    pub fn config(&self) -> &Settings {
        &self.base_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parser::Cli;
    use clap::Parser;

    fn create_valid_base_config() -> Settings {
        let mut config = Settings::default();
        config.database.url = "postgres://localhost/test".to_string();
        config
    }

    #[test]
    fn test_configuration_merger_new() {
        let base_config = Settings::default();
        let merger = ConfigurationMerger::new(base_config.clone());
        assert_eq!(merger.config(), &base_config);
    }

    #[test]
    fn test_configuration_merger_merge_verbose_flag() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);

        let cli = Cli::try_parse_from(&["fusion-rs", "--verbose"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();

        assert_eq!(merged_config.logger.level, "debug");
    }

    #[test]
    fn test_configuration_merger_merge_quiet_flag() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);

        let cli = Cli::try_parse_from(&["fusion-rs", "--quiet"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();

        assert_eq!(merged_config.logger.level, "error");
    }

    #[test]
    fn test_configuration_merger_merge_serve_host() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);

        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--host", "0.0.0.0"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();

        assert_eq!(merged_config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_configuration_merger_merge_serve_port() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);

        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--port", "8080"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();

        assert_eq!(merged_config.server.port, 8080);
    }

    #[test]
    fn test_configuration_merger_command_log_level_overrides_global() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);

        let cli = Cli::try_parse_from(&["fusion-rs", "--verbose", "serve", "--log-level", "warn"])
            .unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();

        assert_eq!(merged_config.logger.level, "warn");
    }
}
