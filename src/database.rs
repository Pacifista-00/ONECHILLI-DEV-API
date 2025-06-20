// src/database.rs
use crate::config::DatabaseConfig;
use crate::tables::{GoodsTable, InventoryTable};
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::{error, info};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
    pub goods_table: GoodsTable,
    pub inventory_table: InventoryTable,
}

impl Database {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Connecting to database...");
        
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.dbname
        );

        // Create connection pool with proper configuration
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&database_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to database: {}", e);
                e
            })?;

        info!("Database connection pool created with {} max connections", config.max_connections);

        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| {
                error!("Database health check failed: {}", e);
                e
            })?;

        info!("Database connection verified");

        // Initialize tables
        let goods_table = GoodsTable::new(pool.clone());
        let inventory_table = InventoryTable::new(pool.clone());
        
        // Verify table access instead of trying to create tables
        crate::utils::database::verify_table_access(&pool, "goods").await?;
        info!("Goods table access verified");

        crate::utils::database::verify_table_access(&pool, "inventory").await?;
        info!("Inventory table access verified");

        Ok(Self {
            pool,
            goods_table,
            inventory_table,
        })
    }

    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
