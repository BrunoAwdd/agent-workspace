use axum::{middleware, routing::{delete, get, post}, Router};

use crate::{
    auth::{self, check_scope},
    handlers::{agents, dependencies, events, handoffs, inbox, locks, messages, reputation, sessions, summary, tasks},
    state::AppState,
};
use tower_http::cors::{Any, CorsLayer};

// Scope constants — what each role/operation requires.
//
// agents:write   → register or update agents
// tasks:read     → list / inspect tasks
// tasks:write    → create, claim, update status
// tasks:admin    → assign tasks (coordinator privilege)
// workspace:read → summary, events, active sessions

pub fn build(state: AppState) -> Router {
    let protected = Router::new()
        // Agents
        .route("/agents",     get(agents::list_agents)
            .post(agents::create_agent)
                .route_layer(middleware::from_fn(|req, next| check_scope("agents:write", req, next))))
        .route("/agents/:id", get(agents::get_agent))
        .route("/agents/:id/eligibility", get(agents::get_agent_eligibility))
        // Sessions
        .route("/sessions/active",    get(sessions::list_active)
            .route_layer(middleware::from_fn(|req, next| check_scope("workspace:read", req, next))))
        .route("/sessions/check-in",  post(sessions::check_in))
        .route("/sessions/heartbeat", post(sessions::heartbeat))
        .route("/sessions/check-out", post(sessions::check_out))
        // Events
        .route("/events", get(events::list_events)
            .route_layer(middleware::from_fn(|req, next| check_scope("workspace:read", req, next))))
        // Messages
        .route("/messages", get(messages::list_messages).post(messages::send_message))
        // Inbox
        .route("/inbox/:agent_id",    get(inbox::list_inbox))
        .route("/inbox/:item_id/ack", post(inbox::ack_inbox_item))
        // Tasks
        .route("/tasks",           get(tasks::list_tasks)
            .route_layer(middleware::from_fn(|req, next| check_scope("tasks:read", req, next))))
        .route("/tasks",           post(tasks::create_task)
            .route_layer(middleware::from_fn(|req, next| check_scope("tasks:write", req, next))))
        .route("/tasks/:id/claim",  post(tasks::claim_task)
            .route_layer(middleware::from_fn(|req, next| check_scope("tasks:write", req, next))))
        .route("/tasks/:id/status", post(tasks::update_task_status)
            .route_layer(middleware::from_fn(|req, next| check_scope("tasks:write", req, next))))
        .route("/tasks/:id/assign", post(tasks::assign_task)
            .route_layer(middleware::from_fn(|req, next| check_scope("tasks:admin", req, next))))
        // Locks
        .route("/locks",     post(locks::acquire_lock))
        .route("/locks/:id", delete(locks::release_lock))
        // Handoffs
        .route("/handoffs",           post(handoffs::create_handoff))
        .route("/handoffs/:agent_id", get(handoffs::list_handoffs))
        // Reputation — legacy
        .route("/agents/:id/reviews",   post(reputation::upsert_review))
        .route("/agents/:id/endorse",   post(reputation::create_endorsement))
        .route("/agents/:id/reputation", get(reputation::get_reputation))
        // Reputation — Phase 1: dual-channel + capabilities
        .route("/agents/:id/human-reviews",     post(reputation::upsert_human_review))
        .route("/agents/:id/agent-peer-reviews", post(reputation::upsert_agent_peer_review))
        .route("/agents/:id/capabilities",       get(reputation::list_capabilities))
        .route("/agents/:id/capabilities/:domain", axum::routing::put(reputation::upsert_capability))
        .route("/agents/:id/full-reputation",   get(reputation::get_full_reputation))
        // Dependencies
        .route("/dependencies",      post(dependencies::upsert_dependency))
        .route("/dependencies/:key", get(dependencies::get_dependency))
        // Summary
        .route("/summary", get(summary::get_summary)
            .route_layer(middleware::from_fn(|req, next| check_scope("workspace:read", req, next))))
        // JWT validation for all protected routes
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_auth));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
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
