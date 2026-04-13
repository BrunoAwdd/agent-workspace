use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use aw_domain::storage::WorkspaceStorage;

pub fn build(storage: Arc<dyn WorkspaceStorage>) -> Router {
    Router::new()
        .route("/health", get(health))
        // Agents
        .route("/agents", get(|| async { "list agents" }))
        .route("/agents", post(|| async { "create agent" }))
        // Sessions
        .route("/sessions/check-in",  post(|| async { "check-in" }))
        .route("/sessions/heartbeat", post(|| async { "heartbeat" }))
        .route("/sessions/check-out", post(|| async { "check-out" }))
        // Messages
        .route("/messages", post(|| async { "send message" }))
        // Inbox
        .route("/inbox/:agent_id",     get(|| async { "list inbox" }))
        .route("/inbox/:item_id/ack",  post(|| async { "ack inbox item" }))
        // Tasks
        .route("/tasks",              post(|| async { "create task" }))
        .route("/tasks/:id/claim",    post(|| async { "claim task" }))
        .route("/tasks/:id/status",   post(|| async { "update task status" }))
        // Locks
        .route("/locks",          post(|| async { "acquire lock" }))
        .route("/locks/:id",      axum::routing::delete(|| async { "release lock" }))
        // Handoffs
        .route("/handoffs",           post(|| async { "create handoff" }))
        .route("/handoffs/:agent_id", get(|| async { "list handoffs" }))
        // Dependencies
        .route("/dependencies",      post(|| async { "upsert dependency" }))
        .route("/dependencies/:key", get(|| async { "get dependency" }))
        .with_state(storage)
}

async fn health() -> &'static str {
    "ok"
}
