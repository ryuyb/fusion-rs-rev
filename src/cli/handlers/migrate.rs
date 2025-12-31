//! Migrate command handler
//!
//! Handles database migration operations including dry-run and rollback.

use crate::config::settings::Settings;
use crate::db::MIGRATIONS;
use crate::error::AppResult;

/// Handler for the migrate command
pub struct MigrateCommandHandler {
    config: Settings,
}

impl MigrateCommandHandler {
    /// Create a new migrate command handler
    pub fn new(config: Settings) -> Self {
        Self { config }
    }

    /// Execute the migrate command with dry-run and rollback support
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
    pub async fn execute(&self, dry_run: bool, rollback: Option<u32>) -> AppResult<()> {
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

        let database_url = self.config.database.url.clone();
        let pending_count: usize = tokio::task::spawn_blocking(move || {
            use diesel::Connection;
            use diesel::pg::PgConnection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url).map_err(|e| {
                crate::error::AppError::Database {
                    operation: "establish connection for migration check".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                }
            })?;

            let pending = conn.pending_migrations(MIGRATIONS).map_err(|e| {
                crate::error::AppError::Database {
                    operation: "check pending migrations".to_string(),
                    source: anyhow::anyhow!("Migration error: {}", e),
                }
            })?;

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
            use diesel::Connection;
            use diesel::pg::PgConnection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url).map_err(|e| {
                crate::error::AppError::Database {
                    operation: "establish connection for migrations".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                }
            })?;

            let applied = conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
                crate::error::AppError::Database {
                    operation: "run pending migrations".to_string(),
                    source: anyhow::anyhow!("Migration error: {}", e),
                }
            })?;

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
            use diesel::Connection;
            use diesel::pg::PgConnection;
            use diesel_migrations::MigrationHarness;

            let mut conn = PgConnection::establish(&database_url).map_err(|e| {
                crate::error::AppError::Database {
                    operation: "establish connection for rollback".to_string(),
                    source: anyhow::anyhow!("Connection error: {}", e),
                }
            })?;

            let applied =
                conn.applied_migrations()
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

            let mut reverted_count = 0;
            for _ in 0..steps {
                conn.revert_last_migration(MIGRATIONS).map_err(|e| {
                    crate::error::AppError::Database {
                        operation: "revert migration".to_string(),
                        source: anyhow::anyhow!("Migration rollback error: {}", e),
                    }
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

    #[test]
    fn test_migrate_handler_new() {
        let config = create_valid_config();
        let handler = MigrateCommandHandler::new(config.clone());
        assert_eq!(handler.config(), &config);
    }

    #[tokio::test]
    async fn test_migrate_handler_zero_rollback_steps() {
        let config = create_valid_config();
        let handler = MigrateCommandHandler::new(config);

        let result = handler.execute(false, Some(0)).await;
        assert!(result.is_err());

        if let Err(crate::error::AppError::Validation { field, reason }) = result {
            assert_eq!(field, "rollback_steps");
            assert!(reason.contains("must be greater than 0"));
        } else {
            panic!("Expected validation error for zero rollback steps");
        }
    }
}
