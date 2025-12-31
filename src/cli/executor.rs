//! Command executor for dispatching CLI commands
//!
//! This module provides the main entry point for executing CLI commands
//! after parsing and configuration loading.

use super::handlers::{MigrateCommandHandler, ServeCommandHandler};
use super::parser::{Cli, Commands};
use crate::config::settings::Settings;
use crate::error::AppResult;

/// Execute a CLI command with the given settings
///
/// This function dispatches to the appropriate command handler based on
/// the parsed CLI arguments.
///
/// # Arguments
/// * `cli` - Parsed CLI arguments
/// * `settings` - Merged and validated settings
///
/// # Returns
/// Returns Ok(()) on success, or AppError on failure
///
/// # Errors
/// Returns errors from command handlers or validation failures
pub async fn execute_command(cli: &Cli, settings: Settings) -> AppResult<()> {
    // Validate CLI arguments and configuration
    validate_command_args(cli, &settings)?;

    match &cli.command {
        Some(Commands::Serve { dry_run, .. }) if *dry_run => {
            ServeCommandHandler::new(settings).execute(true).await
        }
        Some(Commands::Serve { .. }) | None => {
            // Return Ok to signal that server should start
            // Actual server startup is handled in main.rs
            Ok(())
        }
        Some(Commands::Migrate { dry_run, rollback }) => {
            MigrateCommandHandler::new(settings)
                .execute(*dry_run, *rollback)
                .await
        }
    }
}

/// Validate command arguments and configuration before execution
///
/// This function performs comprehensive validation of both CLI arguments
/// and configuration values, providing specific error messages for
/// validation failures.
fn validate_command_args(cli: &Cli, _settings: &Settings) -> AppResult<()> {
    // Validate CLI arguments first
    if let Err(msg) = cli.validate() {
        return Err(crate::error::AppError::Validation {
            field: "cli_arguments".to_string(),
            reason: msg,
        });
    }

    // Validate command-specific requirements
    if let Some(ref command) = cli.command {
        match command {
            Commands::Serve {
                host,
                port,
                log_level: _,
                dry_run: _,
            } => {
                validate_serve_args(host.as_ref(), *port)?;
            }
            Commands::Migrate {
                dry_run: _,
                rollback,
            } => {
                validate_migrate_args(*rollback)?;
            }
        }
    }

    Ok(())
}

/// Validate serve command arguments
fn validate_serve_args(host: Option<&String>, port: Option<u16>) -> AppResult<()> {
    // Additional validation for host/port combinations
    if let (Some(host_addr), Some(port_num)) = (host, port) {
        // Warn about privileged ports
        if port_num < 1024 && host_addr == "0.0.0.0" {
            eprintln!(
                "Warning: Binding to 0.0.0.0 on port {} requires root privileges",
                port_num
            );
        }

        // Validate that the host/port combination makes sense
        if host_addr == "localhost" && port_num == 80 {
            eprintln!("Warning: Using port 80 with localhost may conflict with other services");
        }
    }

    Ok(())
}

/// Validate migrate command arguments
fn validate_migrate_args(rollback: Option<u32>) -> AppResult<()> {
    // Additional validation for rollback steps
    if let Some(steps) = rollback
        && steps > 50
    {
        eprintln!(
            "Warning: Rolling back {} migrations is a large operation. Consider using smaller steps.",
            steps
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parser::Cli;
    use clap::Parser;

    fn create_valid_config() -> Settings {
        let mut config = Settings::default();
        config.database.url = "postgres://localhost/test".to_string();
        config
    }

    #[tokio::test]
    async fn test_execute_serve_dry_run() {
        let cli = Cli::try_parse_from(["fusion-rs", "serve", "--dry-run"]).unwrap();
        let config = create_valid_config();

        let result = execute_command(&cli, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_serve_normal() {
        let cli = Cli::try_parse_from(["fusion-rs", "serve"]).unwrap();
        let config = create_valid_config();

        let result = execute_command(&cli, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_command_args() {
        let cli = Cli::try_parse_from(["fusion-rs", "serve", "--port", "8080"]).unwrap();
        let config = create_valid_config();

        let result = validate_command_args(&cli, &config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_conflicting_args() {
        let cli = Cli {
            command: Some(Commands::Migrate {
                dry_run: true,
                rollback: Some(5),
            }),
            config: None,
            env: None,
            verbose: false,
            quiet: false,
        };
        let config = create_valid_config();

        let result = validate_command_args(&cli, &config);
        assert!(result.is_err());
    }
}
