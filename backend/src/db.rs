use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::config::DATABASE_URL;

pub async fn init_db() -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await?;

    // Run migrations
    sqlx::query(include_str!("../migrations/001_init.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}
