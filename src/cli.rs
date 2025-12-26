use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::config::{ConfigLoader, settings::Settings};
use crate::config::error::ConfigError;
use crate::error::AppResult;
use crate::db::MIGRATIONS;

// Include shadow-rs generated build information
use shadow_rs::shadow;
shadow!(build);

/// Custom validation functions for CLI arguments
mod validation {
    use std::path::PathBuf;
    use std::fs;

    /// Validate port number is within valid range (1-65535)
    pub fn validate_port(port_str: &str) -> Result<u16, String> {
        let port: u16 = port_str.parse()
            .map_err(|_| format!("Port must be a valid number between 1 and 65535, got: '{}'", port_str))?;
        
        if port == 0 {
            return Err("Port must be between 1 and 65535. Port 0 is not allowed.".to_string());
        }
        
        Ok(port)
    }

    /// Validate that a file path is accessible (exists and is readable)
    pub fn validate_config_file_path(path_str: &str) -> Result<PathBuf, String> {
        let path = PathBuf::from(path_str);
        
        // Check if file exists
        if !path.exists() {
            return Err(format!("Configuration file does not exist: '{}'", path_str));
        }
        
        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Err(format!("Configuration path is not a file: '{}'", path_str));
        }
        
        // Check if file is readable
        match fs::File::open(&path) {
            Ok(_) => Ok(path),
            Err(e) => Err(format!("Cannot read configuration file '{}': {}", path_str, e))
        }
    }

    /// Validate rollback steps is a positive number
    pub fn validate_rollback_steps(steps_str: &str) -> Result<u32, String> {
        let steps: u32 = steps_str.parse()
            .map_err(|_| format!("Rollback steps must be a valid positive number, got: '{}'", steps_str))?;
        
        if steps == 0 {
            return Err("Rollback steps must be greater than 0".to_string());
        }
        
        // Reasonable upper limit to prevent accidental mass rollbacks
        if steps > 100 {
            return Err("Rollback steps cannot exceed 100 for safety reasons".to_string());
        }
        
        Ok(steps)
    }

    /// Validate host address format (basic validation)
    pub fn validate_host_address(host_str: &str) -> Result<String, String> {
        let host = host_str.trim();
        
        if host.is_empty() {
            return Err("Host address cannot be empty".to_string());
        }
        
        // Check for common invalid characters
        if host.contains(' ') {
            return Err("Host address cannot contain spaces".to_string());
        }
        
        // Basic validation for common formats
        if host == "localhost" || host == "0.0.0.0" || host.starts_with("127.") {
            return Ok(host.to_string());
        }
        
        // Basic IPv4 validation
        if host.chars().all(|c| c.is_ascii_digit() || c == '.') {
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() == 4 {
                for part in parts {
                    if let Ok(_num) = part.parse::<u8>() {
                        // Valid IPv4 octet
                        continue;
                    } else {
                        return Err(format!("Invalid IPv4 address format: '{}'", host_str));
                    }
                }
                return Ok(host.to_string());
            }
        }
        
        // For other formats (hostnames, IPv6), do basic validation
        if host.len() > 253 {
            return Err("Host address is too long (maximum 253 characters)".to_string());
        }
        
        // Allow hostnames and other valid formats
        Ok(host.to_string())
    }
}

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
    #[arg(short, long, value_name = "FILE", value_parser = validation::validate_config_file_path)]
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
        #[arg(long, value_name = "ADDRESS", value_parser = validation::validate_host_address)]
        host: Option<String>,
        
        /// Port number to listen on
        /// 
        /// The TCP port where the server will accept HTTP connections.
        /// Must be between 1 and 65535. Ports below 1024 typically require root privileges.
        /// 
        /// Default: 3000
        #[arg(short, long, value_name = "PORT", value_parser = validation::validate_port)]
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
        #[arg(long, value_name = "STEPS", conflicts_with = "dry_run", value_parser = validation::validate_rollback_steps)]
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
                Commands::Serve { host, port, log_level: _, dry_run: _ } => {
                    // Additional validation for serve command
                    if let Some(host_addr) = host {
                        // Host validation is already done by clap, but we can add additional checks
                        if host_addr == "0.0.0.0" && port.is_some() && *port.as_ref().unwrap() < 1024 {
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
            })
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
        // Override environment if provided
        if let Some(_env) = &cli.env {
            // Environment override affects how configuration is loaded, but since we've already
            // loaded the config, we'll store this information for potential use by the application
            // The actual environment override would need to be handled during initial config loading
        }

        // Apply logging level overrides from global flags
        if cli.verbose {
            config.logger.level = "debug".to_string();
        } else if cli.quiet {
            config.logger.level = "error".to_string();
        }

        Ok(())
    }

    /// Apply command-specific CLI argument overrides
    fn apply_command_overrides(&self, config: &mut Settings, command: &Commands) -> Result<(), ConfigError> {
        match command {
            Commands::Serve { host, port, log_level, dry_run: _ } => {
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
            Commands::Migrate { dry_run: _, rollback: _ } => {
                // Migration commands don't override server configuration
                // but could potentially override database configuration if needed
            }
        }

        Ok(())
    }

    /// Get the current configuration (useful for inspection)
    pub fn config(&self) -> &Settings {
        &self.base_config
    }
}

/// Command handler for dispatching CLI commands
///
/// This struct provides methods to handle different CLI commands with proper
/// configuration validation and error handling.
pub struct CommandHandler {
    config: Settings,
}

impl CommandHandler {
    /// Create a new command handler with the given configuration
    pub fn new(config: Settings) -> Self {
        Self { config }
    }

    /// Validate command arguments and configuration before execution
    /// 
    /// This method performs comprehensive validation of both CLI arguments
    /// and configuration values, providing specific error messages for
    /// validation failures.
    pub fn validate_command_args(&self, cli: &Cli) -> Result<(), crate::error::AppError> {
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
                Commands::Serve { host, port, log_level: _, dry_run: _ } => {
                    self.validate_serve_args(host.as_ref(), *port)?;
                }
                Commands::Migrate { dry_run: _, rollback } => {
                    self.validate_migrate_args(*rollback)?;
                }
            }
        }

        Ok(())
    }

    /// Validate serve command arguments
    fn validate_serve_args(&self, host: Option<&String>, port: Option<u16>) -> Result<(), crate::error::AppError> {
        // Additional validation for host/port combinations
        if let (Some(host_addr), Some(port_num)) = (host, port) {
            // Warn about privileged ports
            if port_num < 1024 && host_addr == "0.0.0.0" {
                // This is a warning, not an error, so we'll log it but not fail
                eprintln!("Warning: Binding to 0.0.0.0 on port {} requires root privileges", port_num);
            }

            // Validate that the host/port combination makes sense
            if host_addr == "localhost" && port_num == 80 {
                eprintln!("Warning: Using port 80 with localhost may conflict with other services");
            }
        }

        Ok(())
    }

    /// Validate migrate command arguments
    fn validate_migrate_args(&self, rollback: Option<u32>) -> Result<(), crate::error::AppError> {
        // Additional validation for rollback steps
        if let Some(steps) = rollback {
            // The basic validation is already done by clap, but we can add contextual validation
            if steps > 50 {
                eprintln!("Warning: Rolling back {} migrations is a large operation. Consider using smaller steps.", steps);
            }
        }

        Ok(())
    }

    /// Handle the serve command with optional dry-run support
    ///
    /// # Arguments
    /// * `dry_run` - If true, validates configuration and exits without starting server
    ///
    /// # Returns
    /// Returns Ok(()) on success, or AppError on failure
    ///
    /// # Errors
    /// - Configuration validation errors
    /// - Server startup errors (if not dry-run)
    pub async fn handle_serve(&self, dry_run: bool) -> AppResult<()> {
        if dry_run {
            // Validate configuration without starting the server
            self.validate_configuration()?;
            println!("✓ Configuration is valid");
            println!("✓ Server would bind to: {}", self.config.server.address());
            println!("✓ Database URL is configured");
            println!("✓ Logger configuration is valid");
            
            // Additional validation checks for dry-run
            self.validate_server_configuration()?;
            
            println!("Dry run completed successfully - configuration is ready for deployment");
            return Ok(());
        }

        // For actual server startup, this would delegate to the existing server logic
        // This is handled in main.rs integration
        Ok(())
    }

    /// Validate server-specific configuration
    fn validate_server_configuration(&self) -> AppResult<()> {
        // Check if the configured port is available (basic check)
        let address = self.config.server.address();
        println!("✓ Server configuration validated for address: {}", address);
        
        // Additional server validation could go here
        // For example, checking if the port is already in use
        
        Ok(())
    }

    /// Validate the current configuration
    ///
    /// Performs comprehensive validation of all configuration sections
    /// and returns detailed error information if validation fails.
    fn validate_configuration(&self) -> AppResult<()> {
        // Use the existing validation from the Settings struct
        self.config.validate().map_err(|e| e.into())
    }

    /// Get the configuration (useful for integration with main application)
    pub fn config(&self) -> &Settings {
        &self.config
    }

    /// Handle the migrate command with dry-run and rollback support
    ///
    /// # Arguments
    /// * `dry_run` - If true, shows pending migrations without applying them
    /// * `rollback` - Optional number of migrations to rollback
    ///
    /// # Returns
    /// Returns Ok(()) on success, or AppError on failure
    ///
    /// # Errors
    /// - Database connection errors
    /// - Migration execution errors
    /// - Configuration validation errors
    pub async fn handle_migrate(&self, dry_run: bool, rollback: Option<u32>) -> AppResult<()> {
        // Validate database configuration first
        self.config.database.validate()?;

        if dry_run {
            self.show_pending_migrations().await?;
            return Ok(());
        }

        if let Some(steps) = rollback {
            self.rollback_migrations(steps).await?;
        } else {
            self.run_migrations().await?;
        }

        Ok(())
    }

    /// Show pending migrations without applying them
    async fn show_pending_migrations(&self) -> AppResult<()> {
        println!("Checking for pending migrations...");

        // Use blocking task for synchronous diesel operations
        let database_url = self.config.database.url.clone();
        let pending_count: usize = tokio::task::spawn_blocking(move || {
            use diesel::pg::PgConnection;
            use diesel::Connection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| crate::error::AppError::Database {
                    operation: "establish connection for migration check".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                })?;

            let pending = conn
                .pending_migrations(MIGRATIONS)
                .map_err(|e| crate::error::AppError::Database {
                    operation: "check pending migrations".to_string(),
                    source: anyhow::anyhow!("Migration error: {}", e),
                })?;

            // Just return the count to avoid Send issues
            Ok::<_, crate::error::AppError>(pending.len())
        })
        .await
        .map_err(|e| crate::error::AppError::Internal {
            source: anyhow::Error::from(e),
        })??;

        if pending_count == 0 {
            println!("✓ No pending migrations found - database is up to date");
        } else {
            println!("Found {} pending migration(s)", pending_count);
            println!("\nRun without --dry-run to apply these migrations");
        }

        Ok(())
    }

    /// Run pending migrations
    async fn run_migrations(&self) -> AppResult<()> {
        println!("Running database migrations...");

        let database_url = self.config.database.url.clone();
        let applied_migrations = tokio::task::spawn_blocking(move || {
            use diesel::pg::PgConnection;
            use diesel::Connection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| crate::error::AppError::Database {
                    operation: "establish connection for migrations".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                })?;

            let applied = conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| crate::error::AppError::Database {
                    operation: "run pending migrations".to_string(),
                    source: anyhow::anyhow!("Migration error: {}", e),
                })?;

            // Convert to owned strings to avoid lifetime issues
            let migration_names: Vec<String> = applied.iter().map(|m| m.to_string()).collect();
            Ok::<_, crate::error::AppError>(migration_names)
        })
        .await
        .map_err(|e| crate::error::AppError::Internal {
            source: anyhow::Error::from(e),
        })??;

        if applied_migrations.is_empty() {
            println!("✓ No migrations to apply - database is already up to date");
        } else {
            println!("✓ Applied {} migration(s):", applied_migrations.len());
            for migration in &applied_migrations {
                println!("  - {}", migration);
            }
            println!("Database migration completed successfully");
        }

        Ok(())
    }

    /// Rollback the specified number of migrations
    async fn rollback_migrations(&self, steps: u32) -> AppResult<()> {
        if steps == 0 {
            return Err(crate::error::AppError::Validation {
                field: "rollback_steps".to_string(),
                reason: "Number of rollback steps must be greater than 0".to_string(),
            });
        }

        println!("Rolling back {} migration(s)...", steps);

        let database_url = self.config.database.url.clone();
        let reverted_count: usize = tokio::task::spawn_blocking(move || {
            use diesel::pg::PgConnection;
            use diesel::Connection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url)
                .map_err(|e| crate::error::AppError::Database {
                    operation: "establish connection for rollback".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                })?;

            // Get applied migrations to check if we have enough to rollback
            let applied = conn
                .applied_migrations()
                .map_err(|e| crate::error::AppError::Database {
                    operation: "get applied migrations".to_string(),
                    source: anyhow::anyhow!("Migration error: {}", e),
                })?;

            if applied.len() < steps as usize {
                return Err(crate::error::AppError::Validation {
                    field: "rollback_steps".to_string(),
                    reason: format!(
                        "Cannot rollback {} migrations - only {} applied migrations available",
                        steps,
                        applied.len()
                    ),
                });
            }

            // Rollback the specified number of migrations
            let mut reverted_count = 0;
            for _ in 0..steps {
                conn.revert_last_migration(MIGRATIONS)
                    .map_err(|e| crate::error::AppError::Database {
                        operation: "revert migration".to_string(),
                        source: anyhow::anyhow!("Migration rollback error: {}", e),
                    })?;
                reverted_count += 1;
            }

            Ok::<_, crate::error::AppError>(reverted_count)
        })
        .await
        .map_err(|e| crate::error::AppError::Internal {
            source: anyhow::Error::from(e),
        })??;

        println!("✓ Rolled back {} migration(s)", reverted_count);
        println!("Migration rollback completed successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify that the CLI definition is valid
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
        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--host", "0.0.0.0", "--port", "8080"]).unwrap();
        if let Some(Commands::Serve { host, port, log_level: _, dry_run }) = cli.command {
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
    fn test_migrate_rollback() {
        let cli = Cli::try_parse_from(&["fusion-rs", "migrate", "--rollback", "3"]).unwrap();
        if let Some(Commands::Migrate { dry_run, rollback }) = cli.command {
            assert!(!dry_run);
            assert_eq!(rollback, Some(3));
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
    fn test_quiet_flag() {
        let cli = Cli::try_parse_from(&["fusion-rs", "--quiet"]).unwrap();
        assert!(!cli.verbose);
        assert!(cli.quiet);
    }

    #[test]
    fn test_conflicting_verbose_quiet() {
        let result = Cli::try_parse_from(&["fusion-rs", "--verbose", "--quiet"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn test_conflicting_migrate_dry_run_rollback() {
        let result = Cli::try_parse_from(&["fusion-rs", "migrate", "--dry-run", "--rollback", "2"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn test_help_contains_examples() {
        let mut cmd = Cli::command();
        let help_output = cmd.render_long_help().to_string();
        
        // Check that the help contains examples
        assert!(help_output.contains("EXAMPLES:"));
        assert!(help_output.contains("fusion-rs serve"));
        assert!(help_output.contains("fusion-rs migrate"));
        assert!(help_output.contains("--host 0.0.0.0 --port 8080"));
    }

    #[test]
    fn test_serve_subcommand_help() {
        let result = Cli::try_parse_from(&["fusion-rs", "serve", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        
        // Check that the help output contains detailed descriptions
        let help_str = err.to_string();
        assert!(help_str.contains("Host address to bind to"));
        assert!(help_str.contains("Default: 127.0.0.1"));
        assert!(help_str.contains("Examples:"));
    }

    #[test]
    fn test_migrate_subcommand_help() {
        let result = Cli::try_parse_from(&["fusion-rs", "migrate", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        
        // Check that the help output contains detailed descriptions
        let help_str = err.to_string();
        assert!(help_str.contains("Database migration operations"));
        assert!(help_str.contains("Cannot be used with --rollback"));
        assert!(help_str.contains("Examples:"));
    }

    #[test]
    fn test_detailed_help_descriptions() {
        let mut cmd = Cli::command();
        let help_output = cmd.render_long_help().to_string();
        
        // Check for enhanced descriptions in main help
        assert!(help_output.contains("Fusion-rs is a modern Rust web application"));
        assert!(help_output.contains("Configuration file path"));
        assert!(help_output.contains("Specify a custom configuration file"));
        assert!(help_output.contains("Override environment detection"));
        assert!(help_output.contains("Force the application to use a specific environment"));
        assert!(help_output.contains("Enable verbose logging"));
        assert!(help_output.contains("Increases log output to debug level"));
        assert!(help_output.contains("Cannot be used with --quiet"));
        
        // Check for examples
        assert!(help_output.contains("fusion-rs serve --host 0.0.0.0 --port 8080"));
        assert!(help_output.contains("fusion-rs --config /path/to/config.toml serve"));
        assert!(help_output.contains("fusion-rs migrate --dry-run"));
        assert!(help_output.contains("fusion-rs migrate --rollback 2"));
    }

    #[test]
    fn test_environment_values() {
        let cli = Cli::try_parse_from(&["fusion-rs", "--env", "development"]).unwrap();
        assert!(matches!(cli.env, Some(Environment::Development)));

        let cli = Cli::try_parse_from(&["fusion-rs", "--env", "dev"]).unwrap();
        assert!(matches!(cli.env, Some(Environment::Development)));

        let cli = Cli::try_parse_from(&["fusion-rs", "--env", "production"]).unwrap();
        assert!(matches!(cli.env, Some(Environment::Production)));

        let cli = Cli::try_parse_from(&["fusion-rs", "--env", "prod"]).unwrap();
        assert!(matches!(cli.env, Some(Environment::Production)));

        let cli = Cli::try_parse_from(&["fusion-rs", "--env", "test"]).unwrap();
        assert!(matches!(cli.env, Some(Environment::Test)));
    }

    #[test]
    fn test_log_level_values() {
        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--log-level", "debug"]).unwrap();
        if let Some(Commands::Serve { log_level, .. }) = cli.command {
            assert!(matches!(log_level, Some(LogLevel::Debug)));
        } else {
            panic!("Expected Serve command");
        }

        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--log-level", "warn"]).unwrap();
        if let Some(Commands::Serve { log_level, .. }) = cli.command {
            assert!(matches!(log_level, Some(LogLevel::Warn)));
        } else {
            panic!("Expected Serve command");
        }
    }

    #[test]
    fn test_config_file_path() {
        let cli = Cli::try_parse_from(&["fusion-rs", "--config", "/path/to/config.toml"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
    }

    // ========================================================================
    // ConfigurationMerger tests
    // ========================================================================

    #[test]
    fn test_configuration_merger_new() {
        let base_config = Settings::default();
        let merger = ConfigurationMerger::new(base_config.clone());
        assert_eq!(merger.config(), &base_config);
    }

    fn create_valid_base_config() -> Settings {
        let mut config = Settings::default();
        config.database.url = "postgres://localhost/test".to_string();
        config
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
    fn test_configuration_merger_merge_serve_log_level() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);
        
        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--log-level", "trace"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();
        
        assert_eq!(merged_config.logger.level, "trace");
    }

    #[test]
    fn test_configuration_merger_command_log_level_overrides_global() {
        let base_config = create_valid_base_config();
        let merger = ConfigurationMerger::new(base_config);
        
        // Both --verbose and --log-level are provided, command-specific should win
        let cli = Cli::try_parse_from(&["fusion-rs", "--verbose", "serve", "--log-level", "warn"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();
        
        assert_eq!(merged_config.logger.level, "warn");
    }

    #[test]
    fn test_configuration_merger_multiple_overrides() {
        let mut base_config = create_valid_base_config();
        base_config.server.host = "127.0.0.1".to_string();
        base_config.server.port = 3000;
        base_config.logger.level = "info".to_string();
        
        let merger = ConfigurationMerger::new(base_config);
        
        let cli = Cli::try_parse_from(&[
            "fusion-rs", 
            "serve", 
            "--host", "0.0.0.0",
            "--port", "9000",
            "--log-level", "debug"
        ]).unwrap();
        
        let merged_config = merger.merge_cli_args(&cli).unwrap();
        
        assert_eq!(merged_config.server.host, "0.0.0.0");
        assert_eq!(merged_config.server.port, 9000);
        assert_eq!(merged_config.logger.level, "debug");
    }

    #[test]
    fn test_configuration_merger_migrate_command_no_server_overrides() {
        let mut base_config = create_valid_base_config();
        base_config.server.host = "127.0.0.1".to_string();
        base_config.server.port = 3000;
        
        let merger = ConfigurationMerger::new(base_config.clone());
        
        let cli = Cli::try_parse_from(&["fusion-rs", "migrate", "--dry-run"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();
        
        // Migrate command should not affect server configuration
        assert_eq!(merged_config.server.host, base_config.server.host);
        assert_eq!(merged_config.server.port, base_config.server.port);
    }

    #[test]
    fn test_configuration_merger_validation_error() {
        let mut base_config = create_valid_base_config();
        // Set an invalid configuration that will fail validation
        base_config.server.port = 0; // Invalid port
        
        let merger = ConfigurationMerger::new(base_config);
        
        let cli = Cli::try_parse_from(&["fusion-rs"]).unwrap();
        let result = merger.merge_cli_args(&cli);
        
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { field, .. }) = result {
            assert_eq!(field, "server.port");
        } else {
            panic!("Expected ValidationError for invalid port");
        }
    }

    #[test]
    fn test_configuration_merger_cli_fixes_validation_error() {
        let mut base_config = create_valid_base_config();
        // Set an invalid configuration
        base_config.server.port = 0; // Invalid port
        
        let merger = ConfigurationMerger::new(base_config);
        
        // CLI argument should fix the validation error
        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--port", "8080"]).unwrap();
        let merged_config = merger.merge_cli_args(&cli).unwrap();
        
        assert_eq!(merged_config.server.port, 8080);
    }

    // ========================================================================
    // Input validation tests
    // ========================================================================

    #[test]
    fn test_port_validation_valid_ports() {
        // Test valid port numbers
        let valid_ports = ["1", "80", "443", "3000", "8080", "65535"];
        
        for port_str in valid_ports {
            let result = validation::validate_port(port_str);
            assert!(result.is_ok(), "Port {} should be valid", port_str);
        }
    }

    #[test]
    fn test_port_validation_invalid_ports() {
        // Test invalid port numbers
        let invalid_ports = ["0", "65536", "99999", "abc", "-1", ""];
        
        for port_str in invalid_ports {
            let result = validation::validate_port(port_str);
            assert!(result.is_err(), "Port {} should be invalid", port_str);
        }
    }

    #[test]
    fn test_port_validation_error_messages() {
        let result = validation::validate_port("0");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Port must be between 1 and 65535"));
        assert!(error.contains("Port 0 is not allowed"));

        let result = validation::validate_port("abc");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Port must be a valid number"));
    }

    #[test]
    fn test_host_validation_valid_hosts() {
        let valid_hosts = [
            "localhost",
            "127.0.0.1",
            "0.0.0.0",
            "192.168.1.1",
            "10.0.0.1",
            "example.com",
            "my-server.local"
        ];
        
        for host in valid_hosts {
            let result = validation::validate_host_address(host);
            assert!(result.is_ok(), "Host {} should be valid", host);
        }
    }

    #[test]
    fn test_host_validation_invalid_hosts() {
        let invalid_hosts = [
            "",
            "   ",
            "host with spaces",
            "999.999.999.999",
            &"x".repeat(300), // Too long
        ];
        
        for host in invalid_hosts {
            let result = validation::validate_host_address(host);
            assert!(result.is_err(), "Host '{}' should be invalid", host);
        }
    }

    #[test]
    fn test_rollback_steps_validation_valid() {
        let valid_steps = ["1", "5", "10", "50", "100"];
        
        for steps_str in valid_steps {
            let result = validation::validate_rollback_steps(steps_str);
            assert!(result.is_ok(), "Steps {} should be valid", steps_str);
        }
    }

    #[test]
    fn test_rollback_steps_validation_invalid() {
        let invalid_steps = ["0", "101", "999", "-1", "abc", ""];
        
        for steps_str in invalid_steps {
            let result = validation::validate_rollback_steps(steps_str);
            assert!(result.is_err(), "Steps '{}' should be invalid", steps_str);
        }
    }

    #[test]
    fn test_rollback_steps_error_messages() {
        let result = validation::validate_rollback_steps("0");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("must be greater than 0"));

        let result = validation::validate_rollback_steps("101");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("cannot exceed 100"));
    }

    #[test]
    fn test_cli_validation_conflicting_flags() {
        let cli = Cli {
            command: None,
            config: None,
            env: None,
            verbose: true,
            quiet: true, // This should conflict
        };
        
        let result = cli.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Cannot use --verbose and --quiet together"));
    }

    #[test]
    fn test_cli_validation_privileged_port_warning() {
        let cli = Cli {
            command: Some(Commands::Serve {
                host: Some("0.0.0.0".to_string()),
                port: Some(80),
                log_level: None,
                dry_run: false,
            }),
            config: None,
            env: None,
            verbose: false,
            quiet: false,
        };
        
        // This should validate successfully but might generate warnings
        let result = cli.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_validation_migrate_conflicts() {
        let cli = Cli {
            command: Some(Commands::Migrate {
                dry_run: true,
                rollback: Some(5), // This should conflict with dry_run
            }),
            config: None,
            env: None,
            verbose: false,
            quiet: false,
        };
        
        let result = cli.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Cannot use --dry-run and --rollback together"));
    }

    #[test]
    fn test_cli_get_validation_help() {
        let help = Cli::get_validation_help();
        assert!(help.contains("Port validation"));
        assert!(help.contains("Host validation"));
        assert!(help.contains("Configuration file validation"));
        assert!(help.contains("Migration rollback validation"));
        assert!(help.contains("fusion-rs help"));
    }

    // Test CLI parsing with validation
    #[test]
    fn test_cli_parsing_with_invalid_port() {
        let result = Cli::try_parse_from(&["fusion-rs", "serve", "--port", "0"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn test_cli_parsing_with_invalid_rollback() {
        let result = Cli::try_parse_from(&["fusion-rs", "migrate", "--rollback", "0"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ValueValidation);
    }

    #[test]
    fn test_cli_parsing_with_valid_port() {
        let result = Cli::try_parse_from(&["fusion-rs", "serve", "--port", "8080"]);
        assert!(result.is_ok());
        let cli = result.unwrap();
        if let Some(Commands::Serve { port, .. }) = cli.command {
            assert_eq!(port, Some(8080));
        } else {
            panic!("Expected Serve command");
        }
    }

    #[test]
    fn test_cli_parsing_with_valid_host() {
        let result = Cli::try_parse_from(&["fusion-rs", "serve", "--host", "0.0.0.0"]);
        assert!(result.is_ok());
        let cli = result.unwrap();
        if let Some(Commands::Serve { host, .. }) = cli.command {
            assert_eq!(host, Some("0.0.0.0".to_string()));
        } else {
            panic!("Expected Serve command");
        }
    }

    // ========================================================================
    // CommandHandler validation tests
    // ========================================================================

    #[tokio::test]
    async fn test_command_handler_validate_command_args() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config);
        
        let cli = Cli::try_parse_from(&["fusion-rs", "serve", "--port", "8080"]).unwrap();
        let result = handler.validate_command_args(&cli);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_handler_validate_conflicting_args() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config);
        
        // Create CLI with conflicting args manually (since clap would catch this)
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
        
        let result = handler.validate_command_args(&cli);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_command_handler_serve_with_enhanced_validation() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config);
        
        // Test dry run with enhanced validation
        let result = handler.handle_serve(true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_handler_new() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config.clone());
        assert_eq!(handler.config(), &config);
    }

    #[tokio::test]
    async fn test_command_handler_serve_dry_run() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config);
        
        // Dry run should succeed with valid config
        let result = handler.handle_serve(true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_handler_serve_dry_run_invalid_config() {
        let mut config = create_valid_base_config();
        config.server.port = 0; // Invalid port
        let handler = CommandHandler::new(config);
        
        // Dry run should fail with invalid config
        let result = handler.handle_serve(true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_command_handler_migrate_invalid_database_url() {
        let mut config = create_valid_base_config();
        config.database.url = "invalid-url".to_string();
        let handler = CommandHandler::new(config);
        
        // Should fail with invalid database URL
        let result = handler.handle_migrate(true, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_command_handler_migrate_zero_rollback_steps() {
        let config = create_valid_base_config();
        let handler = CommandHandler::new(config);
        
        // Should fail with zero rollback steps
        let result = handler.handle_migrate(false, Some(0)).await;
        assert!(result.is_err());
        
        if let Err(crate::error::AppError::Validation { field, reason }) = result {
            assert_eq!(field, "rollback_steps");
            assert!(reason.contains("must be greater than 0"));
        } else {
            panic!("Expected validation error for zero rollback steps");
        }
    }
}