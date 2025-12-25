//! Configuration types for the advanced logger

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use tracing::Level;

/// Main logger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub console: ConsoleConfig,
    pub file: FileConfig,
    pub level: String, // Will be converted to tracing::Level
}

impl LoggerConfig {
    /// Create a new logger configuration with validation
    pub fn new(console: ConsoleConfig, file: FileConfig, level: String) -> Result<Self> {
        let config = Self {
            console,
            file,
            level,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate log level
        self.parse_level()
            .with_context(|| format!("Invalid log level: {}", self.level))?;
        
        // Validate console config
        self.console.validate()
            .context("Invalid console configuration")?;
        
        // Validate file config
        self.file.validate()
            .context("Invalid file configuration")?;
        
        // Ensure at least one output is enabled
        if !self.console.enabled && !self.file.enabled {
            anyhow::bail!("At least one output (console or file) must be enabled");
        }
        
        Ok(())
    }

    /// Parse the log level string into a tracing::Level
    pub fn parse_level(&self) -> Result<Level> {
        match self.level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => anyhow::bail!("Invalid log level '{}'. Valid levels are: trace, debug, info, warn, error", self.level),
        }
    }

    /// Update configuration at runtime
    pub fn update(&mut self, new_config: LoggerConfig) -> Result<()> {
        new_config.validate()?;
        *self = new_config;
        Ok(())
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            console: ConsoleConfig::default(),
            file: FileConfig::default(),
            level: "info".to_string(),
        }
    }
}

/// Console output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    pub enabled: bool,
    pub colored: bool,
}

impl ConsoleConfig {
    /// Create a new console configuration
    pub fn new(enabled: bool, colored: bool) -> Self {
        Self { enabled, colored }
    }

    /// Validate console configuration
    pub fn validate(&self) -> Result<()> {
        // Console config is always valid - no constraints to check
        Ok(())
    }

    /// Enable console output
    pub fn enable(&mut self) -> &mut Self {
        self.enabled = true;
        self
    }

    /// Disable console output
    pub fn disable(&mut self) -> &mut Self {
        self.enabled = false;
        self
    }

    /// Enable colored output
    pub fn with_colors(&mut self) -> &mut Self {
        self.colored = true;
        self
    }

    /// Disable colored output
    pub fn without_colors(&mut self) -> &mut Self {
        self.colored = false;
        self
    }
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            colored: true,
        }
    }
}

/// File output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub enabled: bool,
    pub path: PathBuf,
    pub append: bool,
    pub format: LogFormat,
    pub rotation: RotationConfig,
}

impl FileConfig {
    /// Create a new file configuration with validation
    pub fn new(
        enabled: bool,
        path: PathBuf,
        append: bool,
        format: LogFormat,
        rotation: RotationConfig,
    ) -> Result<Self> {
        let config = Self {
            enabled,
            path,
            append,
            format,
            rotation,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate file configuration
    /// 
    /// Note: This is a pure validation function that does not create directories.
    /// Directory creation is handled by the writer during initialization.
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            // Validate path is not empty
            if self.path.as_os_str().is_empty() {
                anyhow::bail!("File path cannot be empty when file output is enabled");
            }

            // Validate rotation config
            self.rotation.validate()
                .context("Invalid rotation configuration")?;
        }
        Ok(())
    }

    /// Enable file output
    pub fn enable(&mut self) -> &mut Self {
        self.enabled = true;
        self
    }

    /// Disable file output
    pub fn disable(&mut self) -> &mut Self {
        self.enabled = false;
        self
    }

    /// Set file path
    pub fn with_path<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.path = path.into();
        self
    }

    /// Set append mode
    pub fn with_append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Set log format
    pub fn with_format(&mut self, format: LogFormat) -> &mut Self {
        self.format = format;
        self
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: PathBuf::from("logs/app.log"),
            append: true,
            format: LogFormat::Json,
            rotation: RotationConfig::default(),
        }
    }
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogFormat {
    Full,
    Compact,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Full
    }
}

impl std::str::FromStr for LogFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(LogFormat::Full),
            "compact" => Ok(LogFormat::Compact),
            "json" => Ok(LogFormat::Json),
            _ => anyhow::bail!("Invalid log format '{}'. Valid formats are: full, compact, json", s),
        }
    }
}

impl LogFormat {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogFormat::Full => "full",
            LogFormat::Compact => "compact",
            LogFormat::Json => "json",
        }
    }
}

/// File rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    pub strategy: RotationStrategy,
    pub max_size: u64,      // in bytes
    pub max_files: usize,
    pub compress: bool,
}

impl RotationConfig {
    /// Create a new rotation configuration with validation
    pub fn new(
        strategy: RotationStrategy,
        max_size: u64,
        max_files: usize,
        compress: bool,
    ) -> Result<Self> {
        let config = Self {
            strategy,
            max_size,
            max_files,
            compress,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate rotation configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_size == 0 {
            anyhow::bail!("Maximum file size must be greater than 0");
        }
        
        if self.max_files == 0 {
            anyhow::bail!("Maximum number of files must be greater than 0");
        }

        // Validate strategy-specific constraints
        self.strategy.validate()?;
        
        Ok(())
    }

    /// Set maximum file size
    pub fn with_max_size(&mut self, max_size: u64) -> Result<&mut Self> {
        if max_size == 0 {
            anyhow::bail!("Maximum file size must be greater than 0");
        }
        self.max_size = max_size;
        Ok(self)
    }

    /// Set maximum number of files
    pub fn with_max_files(&mut self, max_files: usize) -> Result<&mut Self> {
        if max_files == 0 {
            anyhow::bail!("Maximum number of files must be greater than 0");
        }
        self.max_files = max_files;
        Ok(self)
    }

    /// Enable compression
    pub fn with_compression(&mut self) -> &mut Self {
        self.compress = true;
        self
    }

    /// Disable compression
    pub fn without_compression(&mut self) -> &mut Self {
        self.compress = false;
        self
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            strategy: RotationStrategy::Size,
            max_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            compress: false,
        }
    }
}

/// Rotation strategy options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RotationStrategy {
    Size,
    Time(TimeUnit),
    Count,
    Combined,
}

impl RotationStrategy {
    /// Validate rotation strategy
    pub fn validate(&self) -> Result<()> {
        match self {
            RotationStrategy::Time(time_unit) => time_unit.validate(),
            _ => Ok(()),
        }
    }
}

impl Default for RotationStrategy {
    fn default() -> Self {
        RotationStrategy::Size
    }
}

/// Time units for time-based rotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeUnit {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl TimeUnit {
    /// Validate time unit
    pub fn validate(&self) -> Result<()> {
        // All time units are valid
        Ok(())
    }

    /// Get duration in seconds for the time unit
    /// 
    /// Note: Monthly uses calendar-aware calculation via `duration_from_now()` for accuracy.
    /// This method returns a fixed approximation (30 days) for backward compatibility.
    pub fn duration_seconds(&self) -> u64 {
        match self {
            TimeUnit::Hourly => 3600,
            TimeUnit::Daily => 86400,
            TimeUnit::Weekly => 604800,
            TimeUnit::Monthly => 30 * 86400, // 30 days approximation
        }
    }

    /// Get the actual duration from a given timestamp, accounting for calendar variations
    /// 
    /// This is more accurate for Monthly rotation as it considers actual month lengths.
    pub fn duration_from(&self, from: chrono::DateTime<chrono::Utc>) -> chrono::Duration {
        use chrono::{Duration, Months};
        
        match self {
            TimeUnit::Hourly => Duration::hours(1),
            TimeUnit::Daily => Duration::days(1),
            TimeUnit::Weekly => Duration::weeks(1),
            TimeUnit::Monthly => {
                // Calculate actual duration to next month
                if let Some(next_month) = from.checked_add_months(Months::new(1)) {
                    next_month.signed_duration_since(from)
                } else {
                    // Fallback to 30 days if overflow
                    Duration::days(30)
                }
            }
        }
    }
}

/// Builder for LoggerConfig
pub struct LoggerConfigBuilder {
    console: ConsoleConfig,
    file: FileConfig,
    level: String,
}

impl LoggerConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            console: ConsoleConfig::default(),
            file: FileConfig::default(),
            level: "info".to_string(),
        }
    }

    /// Set console configuration
    pub fn console(mut self, config: ConsoleConfig) -> Self {
        self.console = config;
        self
    }

    /// Set file configuration
    pub fn file(mut self, config: FileConfig) -> Self {
        self.file = config;
        self
    }

    /// Set log level
    pub fn level<S: Into<String>>(mut self, level: S) -> Self {
        self.level = level.into();
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<LoggerConfig> {
        LoggerConfig::new(self.console, self.file, self.level)
    }
}

impl Default for LoggerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = LoggerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = LoggerConfig::default();
        config.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_both_outputs_disabled() {
        let config = LoggerConfig {
            console: ConsoleConfig { enabled: false, colored: false },
            file: FileConfig { enabled: false, ..Default::default() },
            level: "info".to_string(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_log_levels() {
        for level in &["trace", "debug", "info", "warn", "error"] {
            let config = LoggerConfig {
                level: level.to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_ok(), "Level {} should be valid", level);
        }
    }

    #[test]
    fn test_rotation_config_validation() {
        // Valid config
        let config = RotationConfig::new(
            RotationStrategy::Size,
            1024,
            5,
            false,
        );
        assert!(config.is_ok());

        // Invalid max_size
        let config = RotationConfig::new(
            RotationStrategy::Size,
            0,
            5,
            false,
        );
        assert!(config.is_err());

        // Invalid max_files
        let config = RotationConfig::new(
            RotationStrategy::Size,
            1024,
            0,
            false,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_log_format_parsing() {
        use std::str::FromStr;
        assert_eq!(LogFormat::from_str("full").unwrap(), LogFormat::Full);
        assert_eq!(LogFormat::from_str("compact").unwrap(), LogFormat::Compact);
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_builder_pattern() {
        let config = LoggerConfigBuilder::new()
            .level("debug")
            .console(ConsoleConfig::new(true, false))
            .build();
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.level, "debug");
        assert!(config.console.enabled);
        assert!(!config.console.colored);
    }
}