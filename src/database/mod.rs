pub mod models;
pub mod queries_simple;
pub mod migrations;

pub use queries_simple as queries;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;
use anyhow::Result;

pub async fn create_connection(database_url: &str) -> Result<SqlitePool> {
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    
    let pool = SqlitePool::connect_with(connect_options).await?;
    
    // Run migrations
    migrations::run_migrations(&pool).await?;
    
    Ok(pool)
}