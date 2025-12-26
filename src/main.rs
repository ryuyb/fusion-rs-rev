mod api;
pub mod cli;
mod config;
mod db;
mod error;
mod logger;
mod models;
mod repositories;
mod schema;
mod server;
mod services;
mod state;
mod utils;

pub use state::AppState;

use cli::Cli;
use clap::Parser;
use server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Load and merge configuration
    let settings = cli::load_and_merge_config(&cli)?;

    // Initialize logger
    let _handle = cli::init_logger_from_settings(&settings)?;

    // Execute command
    match cli::execute_command(&cli, settings.clone()).await {
        Ok(()) => {
            // Command completed successfully, or it's a serve command
            // Check if we should start the server
            if matches!(cli.command, Some(cli::Commands::Serve { dry_run: false, .. }) | None) {
                // Start the server
                let server = Server::new(settings);
                match server.run().await {
                    Ok(()) => {
                        tracing::info!("Server shutdown completed successfully");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Server error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Command completed successfully (dry-run or migrate)
                Ok(())
            }
        }
        Err(e) => {
            tracing::error!("Command execution failed: {}", e);
            std::process::exit(1);
        }
    }
}


