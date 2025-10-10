use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Failed to create database pool: {0}")]
    PoolCreation(String),

    #[error("Failed to run migrations: {0}")]
    Migration(String),

    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(String),

    #[error("Invalid database configuration: {0}")]
    Config(String),

    #[error("Environment variable error: {0}")]
    EnvVar(String),

    #[cfg(feature = "database")]
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[cfg(feature = "database")]
    #[error("Migration error: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
}

pub type DbResult<T> = Result<T, DbError>;
