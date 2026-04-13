//! PostgreSQL storage adapter for Agent Workspace.
//!
//! Full implementation mirrors storage-sqlite but uses Postgres-native
//! features: JSONB, advisory locks, pg_notify for real-time events.
//!
//! Status: schema + connection ready; query implementation pending.

mod adapter;

pub use adapter::PostgresStorage;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .context("failed to connect to postgres")?;

    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(pool)
        .await
        .context("migration failed")?;
    Ok(())
}
