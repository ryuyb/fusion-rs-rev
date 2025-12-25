//! Rotating file writer for the advanced logger

use crate::logger::config::FileConfig;
use crate::logger::rotation::RotationManager;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

/// Error recovery strategy for handling write failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Fallback to console output when file write fails
    FallbackToConsole,
    /// Try to clean up old files and retry
    CleanupAndRetry,
    /// Silently drop the log message
    SilentDrop,
}

/// Callback type for error notifications
pub type ErrorCallback = Arc<dyn Fn(&io::Error) + Send + Sync>;

/// File writer with rotation support
pub struct RotatingFileWriter {
    state: Arc<Mutex<WriterState>>,
    config: FileConfig,
    recovery_strategy: RecoveryStrategy,
    error_callback: Option<ErrorCallback>,
}

struct WriterState {
    file: BufWriter<File>,
    current_size: u64,
    rotation_manager: RotationManager,
    /// Flag indicating if we're in fallback mode (writing to console)
    fallback_mode: bool,
    /// Count of consecutive write failures
    failure_count: u32,
}

impl RotatingFileWriter {
    pub fn new(config: &FileConfig) -> anyhow::Result<Self> {
        Self::with_recovery(config, RecoveryStrategy::FallbackToConsole, None)
    }

    /// Create a new RotatingFileWriter with a specific recovery strategy
    pub fn with_recovery(
        config: &FileConfig,
        recovery_strategy: RecoveryStrategy,
        error_callback: Option<ErrorCallback>,
    ) -> anyhow::Result<Self> {
        // Create directory if it doesn't exist
        if let Some(parent) = config.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let file = open_log_file(&config.path, config.append)?;
        let current_size = if config.append {
            std::fs::metadata(&config.path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let rotation_manager = RotationManager::new(config.rotation.clone());

        Ok(Self {
            state: Arc::new(Mutex::new(WriterState {
                file,
                current_size,
                rotation_manager,
                fallback_mode: false,
                failure_count: 0,
            })),
            config: config.clone(),
            recovery_strategy,
            error_callback,
        })
    }

    /// Set the error callback for this writer
    pub fn set_error_callback(&mut self, callback: ErrorCallback) {
        self.error_callback = Some(callback);
    }

    /// Check if the writer is currently in fallback mode
    pub fn is_in_fallback_mode(&self) -> bool {
        self.state
            .lock()
            .map(|s| s.fallback_mode)
            .unwrap_or(false)
    }

    /// Attempt to recover from fallback mode by reopening the file
    pub fn try_recover(&self) -> bool {
        if let Ok(mut state) = self.state.lock() {
            if state.fallback_mode {
                if let Ok(file) = open_log_file(&self.config.path, true) {
                    state.file = file;
                    state.current_size = std::fs::metadata(&self.config.path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    state.fallback_mode = false;
                    state.failure_count = 0;
                    return true;
                }
            }
        }
        false
    }
}

impl<'a> MakeWriter<'a> for RotatingFileWriter {
    type Writer = RotatingWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        RotatingWriterGuard {
            state: self.state.clone(),
            path: self.config.path.clone(),
            recovery_strategy: self.recovery_strategy,
            error_callback: self.error_callback.clone(),
        }
    }
}

/// Guard for file writer access with rotation check
pub struct RotatingWriterGuard {
    state: Arc<Mutex<WriterState>>,
    path: PathBuf,
    recovery_strategy: RecoveryStrategy,
    error_callback: Option<ErrorCallback>,
}

impl Write for RotatingWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self.state.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to acquire writer lock")
        })?;

        // If in fallback mode, write to stderr instead
        if state.fallback_mode {
            return io::stderr().write(buf);
        }

        // Check if rotation is needed
        if state.rotation_manager.should_rotate(state.current_size) {
            // Flush current file
            if let Err(e) = state.file.flush() {
                return self.handle_write_error(&mut state, buf, e);
            }

            // Perform rotation - this may also clean up old files to free disk space
            if let Err(e) = state.rotation_manager.rotate(&self.path) {
                let io_err = io::Error::new(io::ErrorKind::Other, e.to_string());
                return self.handle_write_error(&mut state, buf, io_err);
            }

            // Reopen file (always truncate after rotation)
            match open_log_file(&self.path, false) {
                Ok(file) => {
                    state.file = file;
                    state.current_size = 0;
                    state.failure_count = 0;
                }
                Err(e) => {
                    return self.handle_write_error(&mut state, buf, e);
                }
            }
        }

        // Attempt to write to file
        match state.file.write(buf) {
            Ok(written) => {
                state.current_size += written as u64;
                state.failure_count = 0;
                Ok(written)
            }
            Err(e) => self.handle_write_error(&mut state, buf, e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self.state.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to acquire writer lock")
        })?;

        if state.fallback_mode {
            return io::stderr().flush();
        }

        state.file.flush()
    }
}

impl RotatingWriterGuard {
    /// Handle write errors with recovery strategies
    fn handle_write_error(
        &self,
        state: &mut WriterState,
        buf: &[u8],
        error: io::Error,
    ) -> io::Result<usize> {
        state.failure_count += 1;

        // Notify via callback if set
        if let Some(ref callback) = self.error_callback {
            callback(&error);
        }

        // Check if this is a disk space error using error kind and OS-specific codes
        let is_disk_space_error = Self::is_disk_space_error(&error);

        match self.recovery_strategy {
            RecoveryStrategy::CleanupAndRetry => {
                // Try to clean up old files first (especially for disk space issues)
                if is_disk_space_error || state.failure_count <= 3 {
                    if let Ok(()) = state.rotation_manager.force_cleanup(&self.path) {
                        // Try to reopen and write again
                        if let Ok(file) = open_log_file(&self.path, true) {
                            state.file = file;
                            state.current_size = std::fs::metadata(&self.path)
                                .map(|m| m.len())
                                .unwrap_or(0);

                            // Retry the write
                            match state.file.write(buf) {
                                Ok(written) => {
                                    state.current_size += written as u64;
                                    state.failure_count = 0;
                                    return Ok(written);
                                }
                                Err(_) => {
                                    // Fall through to fallback
                                }
                            }
                        }
                    }
                }

                // If cleanup didn't help, fallback to console
                state.fallback_mode = true;
                eprintln!(
                    "[Logger] File write failed after cleanup attempt, falling back to stderr: {}",
                    error
                );
                io::stderr().write(buf)
            }

            RecoveryStrategy::FallbackToConsole => {
                // Immediately fallback to console output
                state.fallback_mode = true;
                eprintln!(
                    "[Logger] File write failed, falling back to stderr: {}",
                    error
                );
                io::stderr().write(buf)
            }

            RecoveryStrategy::SilentDrop => {
                // Silently drop the message but report success to avoid breaking the logger
                Ok(buf.len())
            }
        }
    }

    /// Check if an error indicates disk space exhaustion
    /// 
    /// Uses both error kind matching and OS-specific error codes for reliability.
    #[cfg(unix)]
    fn is_disk_space_error(error: &io::Error) -> bool {
        // ENOSPC = 28 on most Unix systems
        // EDQUOT = 122 on Linux (disk quota exceeded)
        matches!(error.raw_os_error(), Some(28) | Some(122))
            || error.kind() == io::ErrorKind::StorageFull
    }

    #[cfg(windows)]
    fn is_disk_space_error(error: &io::Error) -> bool {
        // ERROR_DISK_FULL = 112
        // ERROR_HANDLE_DISK_FULL = 39
        matches!(error.raw_os_error(), Some(112) | Some(39))
            || error.kind() == io::ErrorKind::StorageFull
    }

    #[cfg(not(any(unix, windows)))]
    fn is_disk_space_error(error: &io::Error) -> bool {
        error.kind() == io::ErrorKind::StorageFull
    }
}

impl Drop for RotatingWriterGuard {
    fn drop(&mut self) {
        // Ensure buffer is flushed when guard is dropped
        if let Ok(mut state) = self.state.lock() {
            let _ = state.file.flush();
        }
    }
}

fn open_log_file(path: &PathBuf, append: bool) -> io::Result<BufWriter<File>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)?;

    Ok(BufWriter::new(file))
}
