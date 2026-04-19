//! PostgreSQL storage adapter for Agent Workspace.
//!
//! Full implementation mirrors storage-sqlite but uses Postgres-native
//! features: JSONB, advisory locks, pg_notify for real-time events.

mod adapter;
mod rows;

pub use adapter::PostgresStorage;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgPool};

pub async fn connect(url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .context("failed to connect to postgres")?;

    migrate(&pool).await?;
    Ok(pool)
}

/// Create an isolated schema for a single test run.
///
/// Returns `(pool, schema_name)`. The pool is configured with
/// `search_path = <schema>` so all queries are scoped to that schema.
/// Migrations are run automatically against the new schema.
///
/// Intended for integration tests only — not for production use.
pub async fn connect_test(url: &str) -> anyhow::Result<(PgPool, String)> {
    let schema = format!("test_{}", uuid::Uuid::new_v4().simple());

    // Create schema using a one-off connection.
    let mut admin = sqlx::PgConnection::connect(url)
        .await
        .context("admin connect failed")?;
    admin
        .execute(format!("CREATE SCHEMA IF NOT EXISTS {schema}").as_str())
        .await
        .context("create schema failed")?;

    // Build a pool that always sets search_path to the new schema.
    let schema_clone = schema.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _| {
            let s = schema_clone.clone();
            Box::pin(async move {
                conn.execute(format!("SET search_path = {s}").as_str())
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await
        .context("pool connect failed")?;

    migrate(&pool).await?;
    Ok((pool, schema))
}

async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(pool)
        .await
        .context("migration failed")?;
    Ok(())
}
