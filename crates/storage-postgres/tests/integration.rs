//! PostgreSQL integration tests.
//!
//! Requires Docker. Starts a Postgres container once per test binary
//! (via `tokio::sync::OnceCell`). Each test creates an isolated schema
//! so tests can run in parallel without interference.
//!
//! Run with:
//!   cargo test -p aw-storage-postgres

use aw_storage_postgres::PostgresStorage;
use tokio::sync::OnceCell;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

static CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();

async fn pg_port() -> u16 {
    let container = CONTAINER
        .get_or_init(|| async {
            Postgres::default()
                .start()
                .await
                .expect("failed to start Postgres container")
        })
        .await;
    container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get container port")
}

async fn make_storage() -> PostgresStorage {
    let port = pg_port().await;
    let base_url = format!("postgresql://postgres:postgres@127.0.0.1:{}/postgres", port);
    let (pool, _schema) = aw_storage_postgres::connect_test(&base_url)
        .await
        .expect("failed to create isolated postgres schema");
    PostgresStorage::new(pool)
}

aw_storage_tests::define_storage_tests!(make_storage);
