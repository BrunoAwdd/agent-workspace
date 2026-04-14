use axum::{routing::{delete, get, post}, Router};

use crate::{
    handlers::{agents, dependencies, events, handoffs, inbox, locks, messages, sessions, summary, tasks},
    state::AppState,
};
use tower_http::cors::{Any, CorsLayer};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        // Agents
        .route("/agents",     get(agents::list_agents).post(agents::create_agent))
        .route("/agents/:id", get(agents::get_agent))
        // Sessions
        .route("/sessions/active",   get(sessions::list_active))
        .route("/sessions/check-in",  post(sessions::check_in))
        .route("/sessions/heartbeat", post(sessions::heartbeat))
        .route("/sessions/check-out", post(sessions::check_out))
        // Events
        .route("/events", get(events::list_events))
        // Messages
        .route("/messages", get(messages::list_messages).post(messages::send_message))
        // Inbox
        .route("/inbox/:agent_id",    get(inbox::list_inbox))
        .route("/inbox/:item_id/ack", post(inbox::ack_inbox_item))
        // Tasks
        .route("/tasks",             get(tasks::list_tasks).post(tasks::create_task))
        .route("/tasks/:id/claim",   post(tasks::claim_task))
        .route("/tasks/:id/status",  post(tasks::update_task_status))
        .route("/tasks/:id/assign",  post(tasks::assign_task))
        // Locks
        .route("/locks",     post(locks::acquire_lock))
        .route("/locks/:id", delete(locks::release_lock))
        // Handoffs
        .route("/handoffs",           post(handoffs::create_handoff))
        .route("/handoffs/:agent_id", get(handoffs::list_handoffs))
        // Dependencies
        .route("/dependencies",      post(dependencies::upsert_dependency))
        .route("/dependencies/:key", get(dependencies::get_dependency))
        // Summary
        .route("/summary", get(summary::get_summary))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
