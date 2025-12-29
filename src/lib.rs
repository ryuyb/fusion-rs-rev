//! Fusion-RS Library
//!
//! Core library modules for the Fusion-RS web application.

pub mod api;
pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod logger;
pub mod models;
pub mod repositories;
pub mod schema;
pub mod server;
pub mod services;
pub mod state;
pub mod utils;
pub mod external;
pub mod jobs;

pub use state::AppState;
