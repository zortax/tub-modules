// Core modules - always available
pub mod error;
pub mod models;

// Database connectivity - only available with "database" feature
#[cfg(feature = "database")]
pub mod connection;

// Re-exports for convenience
pub use error::{DbError, DbResult};
pub use models::*;

#[cfg(feature = "database")]
pub use connection::{create_pool, create_pool_from_env, run_migrations, DbConfig};

#[cfg(feature = "database")]
pub use sqlx::PgPool;
