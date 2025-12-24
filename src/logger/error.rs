//! Error types for the advanced logger

use thiserror::Error;

/// Errors that can occur in the logger system
#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Rotation error: {message}")]
    Rotation { message: String },
    
    #[error("Compression error: {message}")]
    Compression { message: String },
    
    #[error("Format error: {message}")]
    Format { message: String },
}

impl LoggerError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config { message: message.into() }
    }
    
    pub fn rotation(message: impl Into<String>) -> Self {
        Self::Rotation { message: message.into() }
    }
    
    pub fn compression(message: impl Into<String>) -> Self {
        Self::Compression { message: message.into() }
    }
    
    pub fn format(message: impl Into<String>) -> Self {
        Self::Format { message: message.into() }
    }
}