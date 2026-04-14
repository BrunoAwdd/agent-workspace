use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use aw_api::{routes, state::AppState};
use aw_domain::entities::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

// ── helpers ───────────────────────────────────────────────────────────────────

async fn app() -> axum::Router {
    let pool = aw_storage_sqlite::connect_memory().await.unwrap();
    let storage = Arc::new(aw_storage_sqlite::SqliteStorage::new(pool));
    let state = AppState::new(storage);
    routes::build(state)
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_json(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn delete_json(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

// ── health ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health() {
    let app = app().await;
    let resp = app.oneshot(get("/health")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── agents ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_get_agent() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(post_json(
            "/agents",
            json!({
                "id": "agent-1",
                "name": "Worker One",
                "role": "analyst",
                "capabilities": ["search"],
                "permissions": []
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["id"], "agent-1");

    // GET /agents/:id
    let resp2 = app.oneshot(get("/agents/agent-1")).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2 = json_body(resp2).await;
    assert_eq!(body2["name"], "Worker One");
}

#[tokio::test]
async fn test_get_agent_not_found() {
    let app = app().await;
    let resp = app.oneshot(get("/agents/nobody")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_agents() {
    let app = app().await;

    for i in 1..=3 {
        app.clone()
            .oneshot(post_json(
                "/agents",
                json!({"id": format!("a{}", i), "name": format!("Agent {}", i),
                       "role": "worker", "capabilities": [], "permissions": []}),
            ))
            .await
            .unwrap();
    }

    let resp = app.oneshot(get("/agents")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_agent_create_idempotent() {
    let app = app().await;
    let payload = json!({
        "id": "dup", "name": "First", "role": "worker",
        "capabilities": [], "permissions": []
    });

    app.clone().oneshot(post_json("/agents", payload.clone())).await.unwrap();
    let resp = app.oneshot(post_json("/agents", payload)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "duplicate create should succeed (upsert)");
}

// ── sessions ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_checkin_and_active_sessions() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "alice", "name": "Alice", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "alice"})))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["session"]["agent_id"], "alice");
    assert_eq!(body["session"]["status"], "active");

    let active_resp = app.oneshot(get("/sessions/active")).await.unwrap();
    assert_eq!(active_resp.status(), StatusCode::OK);
    let active = json_body(active_resp).await;
    assert_eq!(active.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_checkout_ends_session() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "bob", "name": "Bob", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    let ci_resp = app
        .clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "bob"})))
        .await
        .unwrap();

    let ci_body = json_body(ci_resp).await;
    let session_id = ci_body["session"]["id"].as_str().unwrap().to_string();

    let co_resp = app
        .clone()
        .oneshot(post_json(
            "/sessions/check-out",
            json!({"session_id": session_id, "create_handoff": false}),
        ))
        .await
        .unwrap();

    assert_eq!(co_resp.status(), StatusCode::NO_CONTENT);

    // Active sessions should now be empty.
    let active_resp = app.oneshot(get("/sessions/active")).await.unwrap();
    let active = json_body(active_resp).await;
    assert!(active.as_array().unwrap().is_empty());
}

// ── messages ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_and_list_messages() {
    let app = app().await;

    for id in ["alice", "bob"] {
        app.clone()
            .oneshot(post_json(
                "/agents",
                json!({"id": id, "name": id, "role": "worker",
                       "capabilities": [], "permissions": []}),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .clone()
        .oneshot(post_json(
            "/messages",
            json!({
                "workspace_id": "main",
                "from_agent_id": "alice",
                "to_agent_id": "bob",
                "kind": "chat_message",
                "payload": {"text": "hello"},
                "deliver_to_inbox": false
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // List by agent.
    let list_resp = app
        .oneshot(get("/messages?agent_id=alice&limit=5"))
        .await
        .unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let msgs = json_body(list_resp).await;
    assert_eq!(msgs.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_send_message_invalid_recipient() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "alice", "name": "Alice", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(post_json(
            "/messages",
            json!({
                "workspace_id": "main",
                "from_agent_id": "alice",
                "to_agent_id": "ghost",
                "kind": "chat_message",
                "payload": {},
                "deliver_to_inbox": false
            }),
        ))
        .await
        .unwrap();

    // Should fail — recipient doesn't exist.
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

// ── tasks ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_task_create_and_list() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(post_json(
            "/tasks",
            json!({
                "title": "analyze market",
                "description": "BTC analysis",
                "kind": "analysis",
                "priority": "high"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let task = json_body(resp).await;
    assert_eq!(task["status"], "open");

    let list_resp = app.oneshot(get("/tasks")).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let tasks = json_body(list_resp).await;
    assert_eq!(tasks.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_task_claim() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "worker", "name": "Worker", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    let ci_resp = app
        .clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "worker"})))
        .await
        .unwrap();
    let ci_body = json_body(ci_resp).await;
    let session_id = ci_body["session"]["id"].as_str().unwrap().to_string();

    let task_resp = app
        .clone()
        .oneshot(post_json(
            "/tasks",
            json!({"title": "do work", "description": "...",
                   "kind": "analysis", "priority": "normal"}),
        ))
        .await
        .unwrap();
    let task_id = json_body(task_resp).await["id"].as_str().unwrap().to_string();

    let claim_resp = app
        .clone()
        .oneshot(post_json(
            &format!("/tasks/{}/claim", task_id),
            json!({"agent_id": "worker", "session_id": session_id}),
        ))
        .await
        .unwrap();

    assert_eq!(claim_resp.status(), StatusCode::OK);
    let claimed = json_body(claim_resp).await;
    assert_eq!(claimed["status"], "claimed");
}

#[tokio::test]
async fn test_task_list_unassigned_filter() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "coord", "name": "Coordinator", "role": "coordinator",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    // Two tasks, one assigned.
    app.clone()
        .oneshot(post_json(
            "/tasks",
            json!({"title": "free task", "description": "...",
                   "kind": "analysis", "priority": "normal"}),
        ))
        .await
        .unwrap();

    let assigned_resp = app
        .clone()
        .oneshot(post_json(
            "/tasks",
            json!({"title": "assigned task", "description": "...",
                   "kind": "analysis", "priority": "normal",
                   "assigned_agent_id": "coord"}),
        ))
        .await
        .unwrap();
    let assigned_task_id = json_body(assigned_resp).await["id"].as_str().unwrap().to_string();

    let list_resp = app
        .oneshot(get("/tasks?unassigned=true"))
        .await
        .unwrap();
    let tasks = json_body(list_resp).await;
    let task_list = tasks.as_array().unwrap();
    assert_eq!(task_list.len(), 1);
    assert_eq!(task_list[0]["title"], "free task");
    // Sanity: the assigned task id is not in unassigned results.
    assert_ne!(task_list[0]["id"].as_str().unwrap(), assigned_task_id);
}

// ── locks ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_acquire_and_release_lock() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "locker", "name": "Locker", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    let ci_resp = app
        .clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "locker"})))
        .await
        .unwrap();
    let session_id = json_body(ci_resp).await["session"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let lock_resp = app
        .clone()
        .oneshot(post_json(
            "/locks",
            json!({
                "scope_type": "document",
                "scope_id": "doc-99",
                "lock_type": "write_lock",
                "owner_agent_id": "locker",
                "owner_session_id": session_id,
                "ttl_secs": 300
            }),
        ))
        .await
        .unwrap();

    assert_eq!(lock_resp.status(), StatusCode::OK);
    let lock = json_body(lock_resp).await;
    let lock_id = lock["id"].as_str().unwrap().to_string();

    let release_resp = app
        .oneshot(delete_json(
            &format!("/locks/{}", lock_id),
            json!({"lock_id": lock_id, "owner_session_id": session_id}),
        ))
        .await
        .unwrap();

    assert_eq!(release_resp.status(), StatusCode::NO_CONTENT);
}

// ── events ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_events_after_checkin() {
    let app = app().await;

    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "eve", "name": "Eve", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    app.clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "eve"})))
        .await
        .unwrap();

    let resp = app
        .oneshot(get("/events?agent_id=eve&limit=10"))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let events = json_body(resp).await;
    assert!(
        !events.as_array().unwrap().is_empty(),
        "check_in should produce at least one event"
    );
}

// ── summary ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_summary() {
    let app = app().await;

    // Register an agent and check in.
    app.clone()
        .oneshot(post_json(
            "/agents",
            json!({"id": "summarized", "name": "Summarized", "role": "worker",
                   "capabilities": [], "permissions": []}),
        ))
        .await
        .unwrap();

    app.clone()
        .oneshot(post_json("/sessions/check-in", json!({"agent_id": "summarized"})))
        .await
        .unwrap();

    app.clone()
        .oneshot(post_json(
            "/tasks",
            json!({"title": "open task", "description": "...",
                   "kind": "analysis", "priority": "normal"}),
        ))
        .await
        .unwrap();

    let resp = app.oneshot(get("/summary")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["active_agents"].as_array().unwrap().len(), 1);
    assert_eq!(body["open_tasks"].as_array().unwrap().len(), 1);
}
