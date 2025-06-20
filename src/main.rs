// src/main.rs
mod config;
mod database;
mod request;
mod response;
mod server;
mod tables;
mod utils;

use anyhow::Result;
use config::AppConfig;
use database::Database;
use server::Server;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting OneChill Dev API server...");

    // Load configuration
    let config = AppConfig::load()?;
    info!("Configuration loaded successfully");

    // Initialize database
    let database = Database::new(config.database.clone()).await?;
    info!("Database connection established");

    // Create and run server
    let server = Server::new(config, database);
    server.run().await?;

    Ok(())
}