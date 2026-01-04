//! Fusion-RS Library
//!
//! Core library modules for the Fusion-RS web application.

use shadow_rs::shadow;
shadow!(build);

pub mod api;
pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod external;
pub mod jobs;
pub mod logger;
pub mod models;
pub mod repositories;
pub mod schema;
pub mod server;
pub mod services;
pub mod state;
pub mod utils;

pub use state::AppState;

pub fn pkg_version() -> &'static str {
    build::PKG_VERSION
}

pub fn clap_long_version() -> &'static str {
    build::CLAP_LONG_VERSION
}
