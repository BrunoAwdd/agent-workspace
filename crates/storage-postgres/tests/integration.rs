//! PostgreSQL integration tests.
//!
//! Requires Docker. Starts a Postgres container once per test binary
//! (via `tokio::sync::OnceCell`). Each test creates an isolated schema
//! so tests can run in parallel without interference.
//!
//! Run with:
//!   cargo test -p aw-storage-postgres

use aw_storage_postgres::PostgresStorage;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use tokio::sync::OnceCell;

// Container + base URL shared across all parallel tests.
static BASE_URL: OnceCell<String> = OnceCell::const_new();
static CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();

/// Start the container once and install extensions. Returns the base URL.
async fn base_url() -> &'static str {
    BASE_URL
        .get_or_init(|| async {
            let container = CONTAINER
                .get_or_init(|| async {
                    Postgres::default()
                        .start()
                        .await
                        .expect("failed to start Postgres container")
                })
                .await;

            let port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("failed to get container port");

            let url = format!("postgresql://postgres:postgres@127.0.0.1:{port}/postgres");

            // Install pgcrypto once — avoids concurrent CREATE EXTENSION races
            // when tests run in parallel.
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&url)
                .await
                .expect("admin connect failed");
            sqlx::Executor::execute(
                &pool,
                "CREATE EXTENSION IF NOT EXISTS pgcrypto SCHEMA public",
            )
            .await
            .expect("pgcrypto install failed");

            url
        })
        .await
}

async fn make_storage() -> PostgresStorage {
    let url = base_url().await;
    let (pool, _schema) = aw_storage_postgres::connect_test(url)
        .await
        .expect("failed to create isolated postgres schema");
    PostgresStorage::new(pool)
}

aw_storage_tests::define_storage_tests!(make_storage);
