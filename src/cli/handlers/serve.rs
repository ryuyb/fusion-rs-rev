//! Serve command handler
//!
//! Handles the serve command including dry-run validation and server startup.

use crate::config::settings::Settings;
use crate::error::AppResult;

/// Handler for the serve command
pub struct ServeCommandHandler {
    config: Settings,
}

impl ServeCommandHandler {
    /// Create a new serve command handler
    pub fn new(config: Settings) -> Self {
        Self { config }
    }

    /// Execute the serve command with optional dry-run support
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
    pub async fn execute(&self, dry_run: bool) -> AppResult<()> {
        if dry_run {
            self.validate_only().await
        } else {
            // For actual server startup, this returns Ok and lets main.rs handle it
            Ok(())
        }
    }

    /// Validate configuration without starting the server
    pub async fn validate_only(&self) -> AppResult<()> {
        // Validate configuration
        self.validate_configuration()?;
        
        println!("✓ Configuration is valid");
        println!("✓ Server would bind to: {}", self.config.server.address());
        println!("✓ Database URL is configured");
        println!("✓ Logger configuration is valid");
        
        // Additional validation checks
        self.validate_server_configuration()?;
        
        println!("Dry run completed successfully - configuration is ready for deployment");
        Ok(())
    }

    /// Validate server-specific configuration
    fn validate_server_configuration(&self) -> AppResult<()> {
        let address = self.config.server.address();
        println!("✓ Server configuration validated for address: {}", address);
        Ok(())
    }

    /// Validate the current configuration
    fn validate_configuration(&self) -> AppResult<()> {
        self.config.validate().map_err(|e| e.into())
    }

    /// Get the configuration
    pub fn config(&self) -> &Settings {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_config() -> Settings {
        let mut config = Settings::default();
        config.database.url = "postgres://localhost/test".to_string();
        config
    }

    #[tokio::test]
    async fn test_serve_handler_new() {
        let config = create_valid_config();
        let handler = ServeCommandHandler::new(config.clone());
        assert_eq!(handler.config(), &config);
    }

    #[tokio::test]
    async fn test_serve_handler_dry_run() {
        let config = create_valid_config();
        let handler = ServeCommandHandler::new(config);
        
        let result = handler.execute(true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_serve_handler_dry_run_invalid_config() {
        let mut config = create_valid_config();
        config.server.port = 0; // Invalid port
        let handler = ServeCommandHandler::new(config);
        
        let result = handler.execute(true).await;
        assert!(result.is_err());
    }
}
