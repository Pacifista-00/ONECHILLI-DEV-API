// src/config.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub username: String,
    pub password: String,
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
        // Try to load from environment variables first (for Render.com)
        if let Ok(db_host) = env::var("DB_HOST") {
            return Ok(AppConfig {
                database: DatabaseConfig {
                    host: db_host,
                    port: env::var("DB_PORT")?.parse()?,
                    dbname: env::var("DB_NAME")?,
                    username: env::var("DB_USER")?,
                    password: env::var("DB_PASSWORD")?,
                    max_connections: env::var("DB_MAX_CONNECTIONS")
                        .unwrap_or_else(|_| "20".to_string())
                        .parse()?,
                },
                server: ServerConfig {
                    host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                    port: env::var("PORT")
                        .unwrap_or_else(|_| "3000".to_string())
                        .parse()?,
                },
            });
        }

        // Fallback to config file
        let config_content = std::fs::read_to_string("config.yaml")?;
        let config: AppConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}