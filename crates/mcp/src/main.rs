mod server;
mod tools;

use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // MCP usa stdout para protocolo — logs vão para stderr
        .init();

    let storage: Arc<dyn aw_domain::storage::WorkspaceStorage> =
        match std::env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "sqlite".to_string())
            .to_lowercase()
            .as_str()
        {
            "postgres" => {
                let url = std::env::var("POSTGRES_URL")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .map_err(|_| anyhow::anyhow!(
                        "STORAGE_BACKEND=postgres requires POSTGRES_URL to be set"
                    ))?;
                tracing::info!("storage: postgres");
                let pool = aw_storage_postgres::connect(&url).await?;
                Arc::new(aw_storage_postgres::PostgresStorage::new(pool))
            }
            _ => {
                let url = std::env::var("SQLITE_URL")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .unwrap_or_else(|_| "sqlite://agent-workspace.db".to_string());
                tracing::info!("storage: sqlite");
                let pool = aw_storage_sqlite::connect(&url).await?;
                Arc::new(aw_storage_sqlite::SqliteStorage::new(pool))
            }
        };

    server::run(storage).await
}
