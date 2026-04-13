mod routes;

use anyhow::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://agent-workspace.db".to_string());

    let pool = aw_storage_sqlite::connect(&db_path).await?;
    let storage: Arc<dyn aw_domain::storage::WorkspaceStorage> =
        Arc::new(aw_storage_sqlite::SqliteStorage::new(pool));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4000);

    let app = routes::build(storage);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("agent-workspace API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
