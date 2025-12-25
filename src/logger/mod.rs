//! Advanced Logger Module
//!
//! A logging system based on `tracing-subscriber` with support for:
//! - Console output with color control
//! - File output with multiple formats (Full, Compact, JSON)
//! - File rotation (size-based, time-based)
//! - Log file compression
//! - Error recovery strategies (fallback to console, cleanup and retry)
//! - Runtime log level modification

#![allow(dead_code)]

pub mod compression;
pub mod config;
pub mod rotation;
pub(crate) mod writer;

#[cfg(test)]
mod tests;

// Re-export main types
pub use config::*;

use std::io::IsTerminal;
use std::sync::Arc;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    reload,
    util::SubscriberInitExt,
    EnvFilter,
};
use writer::RotatingFileWriter;

/// Handle for modifying the log level at runtime
/// 
/// This handle can be cloned and shared across threads safely.
#[derive(Clone, Debug)]
pub struct LogLevelHandle {
    inner: Arc<reload::Handle<EnvFilter, tracing_subscriber::Registry>>,
}

impl LogLevelHandle {
    /// Modify the log level at runtime
    /// 
    /// # Arguments
    /// * `new_level` - The new log level string (e.g., "info", "debug", "warn,my_crate=trace")
    /// 
    /// # Returns
    /// * `Ok(())` if the level was successfully updated
    /// * `Err` if the level string is invalid
    /// 
    /// # Example
    /// ```ignore
    /// let handle = init_logger(config)?;
    /// // Later, change the log level
    /// handle.set_level("debug")?;
    /// ```
    pub fn set_level(&self, new_level: &str) -> anyhow::Result<()> {
        let new_filter = EnvFilter::try_new(new_level)
            .map_err(|e| anyhow::anyhow!("Invalid log level '{}': {}", new_level, e))?;
        
        self.inner
            .modify(|filter| *filter = new_filter)
            .map_err(|e| anyhow::anyhow!("Failed to update log level: {}", e))?;
        
        Ok(())
    }

    /// Get the current log level configuration
    /// 
    /// Note: This clones the current filter, which may be expensive for complex filters.
    pub fn current_level(&self) -> Option<String> {
        self.inner.with_current(|filter| filter.to_string()).ok()
    }
}

/// Initialize the logger with the given configuration
/// 
/// Returns a `LogLevelHandle` that can be used to modify the log level at runtime.
/// 
/// # Example
/// ```ignore
/// use advanced_logger::{LoggerConfig, init_logger};
/// 
/// let config = LoggerConfig::default();
/// let handle = init_logger(config)?;
/// 
/// // Log at info level
/// tracing::info!("Starting application");
/// 
/// // Later, change to debug level
/// handle.set_level("debug")?;
/// tracing::debug!("This will now be visible");
/// ```
pub fn init_logger(config: LoggerConfig) -> anyhow::Result<LogLevelHandle> {
    config.validate()?;

    // Create filter from level string - validation already passed, so this should not fail
    let filter = EnvFilter::try_new(&config.level)
        .map_err(|e| anyhow::anyhow!("Failed to create log filter: {}", e))?;

    match (config.console.enabled, config.file.enabled) {
        (true, true) => init_both(&config, filter),
        (true, false) => Ok(init_console_only(&config.console, filter)),
        (false, true) => init_file_only(&config.file, filter),
        (false, false) => anyhow::bail!("At least one output (console or file) must be enabled"),
    }
}

fn init_console_only(config: &ConsoleConfig, filter: EnvFilter) -> LogLevelHandle {
    let is_tty = std::io::stdout().is_terminal();
    let use_ansi = config.colored && is_tty;

    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_level(true),
        )
        .init();

    LogLevelHandle {
        inner: Arc::new(reload_handle),
    }
}

fn init_file_only(config: &FileConfig, filter: EnvFilter) -> anyhow::Result<LogLevelHandle> {
    let writer = RotatingFileWriter::new(config)?;
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    match config.format {
        LogFormat::Full => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_target(true)
                        .with_writer(writer),
                )
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_target(true)
                        .compact()
                        .with_writer(writer),
                )
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt::layer().with_ansi(false).json().with_writer(writer))
                .init();
        }
    }

    Ok(LogLevelHandle {
        inner: Arc::new(reload_handle),
    })
}

fn init_both(config: &LoggerConfig, filter: EnvFilter) -> anyhow::Result<LogLevelHandle> {
    let is_tty = std::io::stdout().is_terminal();
    let use_ansi = config.console.colored && is_tty;
    let writer = RotatingFileWriter::new(&config.file)?;

    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    // IMPORTANT: File layer must be added BEFORE console layer to avoid ANSI codes
    // leaking into file output. This is a known tracing-subscriber behavior where
    // span field formatting is affected by the first layer's ANSI setting.
    // See: https://github.com/tokio-rs/tracing/issues/1817
    //
    // Note: Due to tracing-subscriber's type system, we cannot share the console_layer
    // across match arms - each branch needs its own layer instance.
    match config.file.format {
        LogFormat::Full => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt::layer().with_ansi(false).with_target(true).with_writer(writer))
                .with(fmt::layer().with_ansi(use_ansi).with_target(true).with_level(true))
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt::layer().with_ansi(false).with_target(true).compact().with_writer(writer))
                .with(fmt::layer().with_ansi(use_ansi).with_target(true).with_level(true))
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt::layer().with_ansi(false).json().with_writer(writer))
                .with(fmt::layer().with_ansi(use_ansi).with_target(true).with_level(true))
                .init();
        }
    }

    Ok(LogLevelHandle {
        inner: Arc::new(reload_handle),
    })
}
