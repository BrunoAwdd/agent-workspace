use anyhow::Result;
use std::sync::Arc;

/// Background maintenance loop: sweeps dead sessions and expires stale locks
/// every 60 seconds. Runs independently of any agent activity.
async fn maintenance_loop(storage: Arc<dyn aw_domain::storage::WorkspaceStorage>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        match storage.sweep_dead_sessions(300).await {
            Ok(swept) if swept > 0 => tracing::info!("maintenance: swept {} dead session(s)", swept),
            Err(e) => tracing::warn!("maintenance: sweep_dead_sessions failed: {}", e),
            _ => {}
        }
        match storage.expire_stale_locks().await {
            Ok(expired) if expired > 0 => tracing::info!("maintenance: expired {} stale lock(s)", expired),
            Err(e) => tracing::warn!("maintenance: expire_stale_locks failed: {}", e),
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

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
                tracing::info!("storage: postgres ({})", url);
                let pool = aw_storage_postgres::connect(&url).await?;
                Arc::new(aw_storage_postgres::PostgresStorage::new(pool))
            }
            "sqlite" | _ => {
                let url = std::env::var("SQLITE_URL")
                    .or_else(|_| std::env::var("DATABASE_URL"))
                    .unwrap_or_else(|_| "sqlite://agent-workspace.db".to_string());
                tracing::info!("storage: sqlite ({})", url);
                let pool = aw_storage_sqlite::connect(&url).await?;
                Arc::new(aw_storage_sqlite::SqliteStorage::new(pool))
            }
        };

    // Spawn background maintenance — runs every 60s regardless of agent activity.
    tokio::spawn(maintenance_loop(Arc::clone(&storage)));

    let state = aw_api::state::AppState::new(storage);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4000);

    let app = aw_api::routes::build(state);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("agent-workspace API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
