mod tools;
mod server;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://agent-workspace.db".to_string());

    let pool = aw_storage_sqlite::connect(&db_path).await?;
    let storage = std::sync::Arc::new(aw_storage_sqlite::SqliteStorage::new(pool));

    server::run(storage).await
}
