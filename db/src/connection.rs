#[cfg(feature = "database")]
use sqlx::postgres::{PgPool, PgPoolOptions};
#[cfg(feature = "database")]
use std::time::Duration;

use crate::error::{DbError, DbResult};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

impl DbConfig {
    /// Create a new database configuration from environment variables
    #[cfg(feature = "database")]
    pub fn from_env() -> DbResult<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: std::env::var("DATABASE_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .map_err(|e| DbError::Config(format!("Invalid port: {}", e)))?,
            username: std::env::var("DATABASE_USER")
                .unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD")
                .unwrap_or_else(|_| "postgres".to_string()),
            database: std::env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "tub_modules".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|e| DbError::Config(format!("Invalid max_connections: {}", e)))?,
        })
    }

    /// Build the database URL from configuration
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// Create a new database connection pool
#[cfg(feature = "database")]
pub async fn create_pool(config: &DbConfig) -> DbResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database_url())
        .await
        .map_err(|e| DbError::PoolCreation(e.to_string()))?;

    Ok(pool)
}

/// Create a database pool from environment variables
#[cfg(feature = "database")]
pub async fn create_pool_from_env() -> DbResult<PgPool> {
    let config = DbConfig::from_env()?;
    create_pool(&config).await
}

/// Run pending database migrations
#[cfg(feature = "database")]
pub async fn run_migrations(pool: &PgPool) -> DbResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::Migration(e.to_string()))?;

    Ok(())
}

#[cfg(all(test, feature = "database"))]
mod tests {
    use super::*;

    #[test]
    fn test_database_url() {
        let config = DbConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            database: "testdb".to_string(),
            max_connections: 5,
        };

        assert_eq!(
            config.database_url(),
            "postgres://testuser:testpass@localhost:5432/testdb"
        );
    }
}
