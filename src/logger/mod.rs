//! Advanced Logger Module
//!
//! A logging system based on `tracing-subscriber` with support for:
//! - Console output with color control
//! - File output with multiple formats (Full, Compact, JSON)
//! - File rotation (size-based, time-based)
//! - Log file compression
//! - Error recovery strategies (fallback to console, cleanup and retry)

pub mod compression;
pub mod config;
pub mod error;
pub mod rotation;
pub(crate) mod writer;

#[cfg(test)]
mod tests;

// Re-export main types
pub use config::*;
pub use error::LoggerError;
pub use writer::{RecoveryStrategy, ErrorCallback};

use std::io::IsTerminal;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use writer::RotatingFileWriter;

/// Initialize the logger with the given configuration
pub fn init_logger(config: LoggerConfig) -> anyhow::Result<()> {
    config.validate()?;

    // Create filter from level string
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    match (config.console.enabled, config.file.enabled) {
        (true, true) => init_both(&config, filter)?,
        (true, false) => init_console_only(&config.console, filter),
        (false, true) => init_file_only(&config.file, filter)?,
        (false, false) => anyhow::bail!("At least one output (console or file) must be enabled"),
    }

    Ok(())
}

fn init_console_only(config: &ConsoleConfig, filter: EnvFilter) {
    let is_tty = std::io::stdout().is_terminal();
    let use_ansi = config.colored && is_tty;

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_level(true),
        )
        .init();
}

fn init_file_only(config: &FileConfig, filter: EnvFilter) -> anyhow::Result<()> {
    let writer = RotatingFileWriter::new(config)?;

    match config.format {
        LogFormat::Full => {
            tracing_subscriber::registry()
                .with(filter)
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
                .with(filter)
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
                .with(filter)
                .with(fmt::layer().with_ansi(false).json().with_writer(writer))
                .init();
        }
    }

    Ok(())
}

fn init_both(config: &LoggerConfig, filter: EnvFilter) -> anyhow::Result<()> {
    let is_tty = std::io::stdout().is_terminal();
    let use_ansi = config.console.colored && is_tty;
    let writer = RotatingFileWriter::new(&config.file)?;

    // IMPORTANT: File layer must be added BEFORE console layer to avoid ANSI codes
    // leaking into file output. This is a known tracing-subscriber behavior where
    // span field formatting is affected by the first layer's ANSI setting.
    // See: https://github.com/tokio-rs/tracing/issues/1817
    match config.file.format {
        LogFormat::Full => {
            let file_layer = fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_writer(writer);

            let console_layer = fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_level(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(console_layer)
                .init();
        }
        LogFormat::Compact => {
            let file_layer = fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .compact()
                .with_writer(writer);

            let console_layer = fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_level(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(console_layer)
                .init();
        }
        LogFormat::Json => {
            let file_layer = fmt::layer()
                .with_ansi(false)
                .json()
                .with_writer(writer);

            let console_layer = fmt::layer()
                .with_ansi(use_ansi)
                .with_target(true)
                .with_level(true);

            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(console_layer)
                .init();
        }
    }

    Ok(())
}
