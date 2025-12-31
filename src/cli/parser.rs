//! CLI argument parsing with clap
//!
//! This module defines the command-line interface structure using clap,
//! including all commands, arguments, and their documentation.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

// Include shadow-rs generated build information
use shadow_rs::shadow;
shadow!(build);

/// A Rust web application with database integration
#[derive(Parser, Debug)]
#[command(name = "fusion-rs")]
#[command(about = "A Rust web application with database integration")]
#[command(long_about = "
Fusion-rs is a modern Rust web application with integrated database support.
It provides a RESTful API server with comprehensive configuration management,
database migrations, and flexible deployment options.

EXAMPLES:
    # Start the server with default configuration
    fusion-rs serve

    # Start server on custom host and port
    fusion-rs serve --host 0.0.0.0 --port 8080

    # Use custom configuration file
    fusion-rs --config /path/to/config.toml serve

    # Run in development mode with verbose logging
    fusion-rs --env development --verbose serve

    # Check configuration without starting server
    fusion-rs serve --dry-run

    # Run database migrations
    fusion-rs migrate

    # Preview pending migrations
    fusion-rs migrate --dry-run

    # Rollback last 2 migrations
    fusion-rs migrate --rollback 2

For more information about configuration options, see the documentation.
")]
#[command(version = build::CLAP_LONG_VERSION)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Configuration file path
    ///
    /// Specify a custom configuration file to use instead of the default.
    /// The file should be in TOML format and contain valid configuration sections.
    /// The file must exist and be readable.
    ///
    /// Example: --config /etc/fusion-rs/production.toml
    #[arg(short, long, value_name = "FILE", value_parser = super::validation::validate_config_file_path)]
    pub config: Option<PathBuf>,

    /// Override environment detection
    ///
    /// Force the application to use a specific environment configuration.
    /// This affects which configuration files are loaded and default settings.
    ///
    /// Available values: development (dev), production (prod), test
    #[arg(short, long, value_enum)]
    pub env: Option<Environment>,

    /// Enable verbose logging
    ///
    /// Increases log output to debug level, showing detailed information
    /// about application operations. Useful for troubleshooting.
    /// Cannot be used with --quiet.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-error output
    ///
    /// Reduces log output to error level only, hiding informational messages.
    /// Useful for production deployments or automated scripts.
    /// Cannot be used with --verbose.
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the web server (default)
    ///
    /// Launches the HTTP server with the configured settings. The server will
    /// bind to the specified host and port, load the database connection pool,
    /// and begin accepting requests.
    ///
    /// Examples:
    ///   fusion-rs serve                           # Start with defaults
    ///   fusion-rs serve --host 0.0.0.0 --port 80 # Bind to all interfaces on port 80
    ///   fusion-rs serve --dry-run                 # Validate config without starting
    Serve {
        /// Host address to bind to
        ///
        /// The network interface address where the server will listen for connections.
        /// Use 127.0.0.1 for localhost only, or 0.0.0.0 to accept connections from any interface.
        /// Must be a valid IPv4 address, hostname, or 'localhost'.
        ///
        /// Default: 127.0.0.1
        #[arg(long, value_name = "ADDRESS", value_parser = super::validation::validate_host_address)]
        host: Option<String>,

        /// Port number to listen on
        ///
        /// The TCP port where the server will accept HTTP connections.
        /// Must be between 1 and 65535. Ports below 1024 typically require root privileges.
        ///
        /// Default: 3000
        #[arg(short, long, value_name = "PORT", value_parser = super::validation::validate_port)]
        port: Option<u16>,

        /// Log level override
        ///
        /// Set the logging verbosity for this server instance.
        /// This overrides both configuration file settings and global --verbose/--quiet flags.
        ///
        /// Available levels: error, warn, info, debug, trace
        #[arg(long, value_enum)]
        log_level: Option<LogLevel>,

        /// Validate configuration and exit
        ///
        /// Performs a complete configuration validation check without starting the server.
        /// Useful for testing configuration changes or deployment validation.
        /// Returns exit code 0 if valid, non-zero if invalid.
        #[arg(long)]
        dry_run: bool,
    },
    /// Database migration operations
    ///
    /// Manage database schema migrations. This command connects to the configured
    /// database and applies or rolls back schema changes.
    ///
    /// Examples:
    ///   fusion-rs migrate                    # Apply all pending migrations
    ///   fusion-rs migrate --dry-run          # Show pending migrations without applying
    ///   fusion-rs migrate --rollback 3       # Rollback the last 3 migrations
    Migrate {
        /// Show pending migrations without applying
        ///
        /// Lists all migrations that would be applied without actually running them.
        /// Useful for reviewing changes before deployment.
        /// Cannot be used with --rollback.
        #[arg(long, conflicts_with = "rollback")]
        dry_run: bool,

        /// Number of migrations to rollback
        ///
        /// Reverts the specified number of most recent migrations.
        /// Use with caution as this can result in data loss.
        /// Must be between 1 and 100 for safety reasons.
        /// Cannot be used with --dry-run.
        ///
        /// Example: --rollback 2 (reverts last 2 migrations)
        #[arg(long, value_name = "STEPS", conflicts_with = "dry_run", value_parser = super::validation::validate_rollback_steps)]
        rollback: Option<u32>,
    },
}

/// Environment options
#[derive(ValueEnum, Clone, Debug)]
pub enum Environment {
    #[value(name = "development", alias = "dev")]
    Development,
    #[value(name = "production", alias = "prod")]
    Production,
    #[value(name = "test")]
    Test,
}

/// Log level options
#[derive(ValueEnum, Clone, Debug)]
pub enum LogLevel {
    #[value(name = "error")]
    Error,
    #[value(name = "warn", alias = "warning")]
    Warn,
    #[value(name = "info")]
    Info,
    #[value(name = "debug")]
    Debug,
    #[value(name = "trace")]
    Trace,
}

impl Cli {
    /// Validate CLI arguments and provide detailed error messages
    ///
    /// This method performs additional validation beyond what clap provides,
    /// ensuring that all argument combinations are valid and providing
    /// specific error messages for validation failures.
    pub fn validate(&self) -> Result<(), String> {
        // Validate command-specific arguments
        if let Some(ref command) = self.command {
            match command {
                Commands::Serve {
                    host,
                    port,
                    log_level: _,
                    dry_run: _,
                } => {
                    // Additional validation for serve command
                    if let Some(host_addr) = host {
                        // Host validation is already done by clap, but we can add additional checks
                        if host_addr == "0.0.0.0"
                            && port.is_some()
                            && *port.as_ref().unwrap() < 1024
                        {
                            return Err("Warning: Binding to 0.0.0.0 on a privileged port (< 1024) typically requires root privileges".to_string());
                        }
                    }
                }
                Commands::Migrate { dry_run, rollback } => {
                    // Validate migrate command arguments
                    if *dry_run && rollback.is_some() {
                        return Err("Cannot use --dry-run and --rollback together".to_string());
                    }
                }
            }
        }

        // Validate global argument combinations
        if self.verbose && self.quiet {
            return Err("Cannot use --verbose and --quiet together".to_string());
        }

        Ok(())
    }

    /// Get detailed help for validation errors
    pub fn get_validation_help() -> &'static str {
        r#"
Common validation errors and solutions:

Port validation:
  - Port must be between 1 and 65535
  - Ports below 1024 require root privileges on most systems
  - Example: --port 8080

Host validation:
  - Use 'localhost' or '127.0.0.1' for local access only
  - Use '0.0.0.0' to accept connections from any interface
  - IPv4 addresses must be in valid format (e.g., 192.168.1.100)
  - Example: --host 0.0.0.0

Configuration file validation:
  - File must exist and be readable
  - File must be in TOML format
  - Example: --config /path/to/config.toml

Migration rollback validation:
  - Steps must be between 1 and 100
  - Cannot be used with --dry-run
  - Example: --rollback 3

For more help, use: fusion-rs help <subcommand>
"#
    }
}

impl From<LogLevel> for String {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => "error".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Info => "info".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Trace => "trace".to_string(),
        }
    }
}

impl From<Environment> for crate::config::Environment {
    fn from(env: Environment) -> Self {
        match env {
            Environment::Development => crate::config::Environment::Development,
            Environment::Production => crate::config::Environment::Production,
            Environment::Test => crate::config::Environment::Test,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_help_flag() {
        let result = Cli::try_parse_from(&["fusion-rs", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_version_flag() {
        let result = Cli::try_parse_from(&["fusion-rs", "--version"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_default_behavior() {
        let cli = Cli::try_parse_from(&["fusion-rs"]).unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.verbose);
        assert!(!cli.quiet);
        assert!(cli.config.is_none());
        assert!(cli.env.is_none());
    }

    #[test]
    fn test_serve_command() {
        let cli =
            Cli::try_parse_from(&["fusion-rs", "serve", "--host", "0.0.0.0", "--port", "8080"])
                .unwrap();
        if let Some(Commands::Serve {
            host,
            port,
            log_level: _,
            dry_run,
        }) = cli.command
        {
            assert_eq!(host, Some("0.0.0.0".to_string()));
            assert_eq!(port, Some(8080));
            assert!(!dry_run);
        } else {
            panic!("Expected Serve command");
        }
    }

    #[test]
    fn test_migrate_command() {
        let cli = Cli::try_parse_from(&["fusion-rs", "migrate", "--dry-run"]).unwrap();
        if let Some(Commands::Migrate { dry_run, rollback }) = cli.command {
            assert!(dry_run);
            assert!(rollback.is_none());
        } else {
            panic!("Expected Migrate command");
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::try_parse_from(&["fusion-rs", "--verbose"]).unwrap();
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_conflicting_verbose_quiet() {
        let result = Cli::try_parse_from(&["fusion-rs", "--verbose", "--quiet"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
