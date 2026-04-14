mod adapter;
mod rows;

pub use adapter::SqliteStorage;

use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Open a single-connection in-memory SQLite database with migrations applied.
/// Intended for integration tests — each call returns a fresh isolated database.
pub async fn connect_memory() -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .context("failed to open in-memory sqlite")?;
    migrate(&pool).await?;
    Ok(pool)
}

pub async fn connect(path: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(path)
        .context("invalid sqlite path")?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(opts)
        .await
        .context("failed to open sqlite database")?;

    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    let sql = include_str!("../migrations/0001_init.sql");
    sqlx::raw_sql(sql)
        .execute(pool)
        .await
        .context("migration 0001 failed")?;

    // 0002: inbox retry columns — idempotent (ignore "duplicate column" errors).
    for stmt in [
        "ALTER TABLE inbox_items ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE inbox_items ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 3",
    ] {
        let _ = sqlx::raw_sql(stmt).execute(pool).await;
    }

    Ok(())
}
