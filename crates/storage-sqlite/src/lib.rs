mod adapter;
mod rows;

pub use adapter::SqliteStorage;

use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

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
        .context("migration failed")?;
    Ok(())
}
