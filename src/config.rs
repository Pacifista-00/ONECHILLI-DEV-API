// src/config.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file if it exists
        if let Err(_) = dotenvy::dotenv() {
            // .env file doesn't exist, continue with environment variables
            tracing::info!("No .env file found, using environment variables");
        } else {
            tracing::info!("Loaded environment variables from .env file");
        }

        // Load database config from environment variables
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable is required"))?;
        
        let max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<u32>()?;

        let database_config = DatabaseConfig {
            database_url,
            max_connections,
        };

        // Try to load server config from config.yaml first
        let server_config = if let Ok(config_content) = std::fs::read_to_string("config.yaml") {
            let yaml_config: ServerConfigYaml = serde_yaml::from_str(&config_content)?;
            ServerConfig {
                host: yaml_config.server.host,
                port: yaml_config.server.port,
            }
        } else {
            // Fallback to environment variables for server config
            ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
            }
        };

        Ok(AppConfig {
            database: database_config,
            server: server_config,
        })
    }
}

// Helper struct for parsing YAML server config
#[derive(Debug, Deserialize)]
struct ServerConfigYaml {
    server: ServerConfigInner,
}

#[derive(Debug, Deserialize)]
struct ServerConfigInner {
    host: String,
    port: u16,
}