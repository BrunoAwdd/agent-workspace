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

    let db_path = std::env::var("SQLITE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "sqlite://agent-workspace.db".to_string());

    let pool = aw_storage_sqlite::connect(&db_path).await?;
    let storage: Arc<dyn aw_domain::storage::WorkspaceStorage> =
        Arc::new(aw_storage_sqlite::SqliteStorage::new(pool));

    server::run(storage).await
}
