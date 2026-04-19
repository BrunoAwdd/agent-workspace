//! SQLite integration tests — delegates to the shared aw-storage-tests suite.

use aw_storage_sqlite::SqliteStorage;

async fn make_storage() -> SqliteStorage {
    let pool = aw_storage_sqlite::connect_memory().await.unwrap();
    SqliteStorage::new(pool)
}

aw_storage_tests::define_storage_tests!(make_storage);

// ── SQLite-specific tests (inbox retry logic) ──────────────────────────────

use aw_domain::entities::*;
use aw_domain::storage::WorkspaceStorage;

#[allow(dead_code)]
fn mk_agent(id: &str) -> CreateAgentInput {
    aw_storage_tests::mk_agent(id)
}

#[allow(dead_code)]
fn mk_session(agent_id: &str) -> CheckInInput {
    aw_storage_tests::mk_session(agent_id)
}

#[tokio::test]
async fn test_inbox_ack_failure_retries() {
    let s = make_storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::ReviewRequest,
        payload: serde_json::json!({}),
        deliver_to_inbox: true,
    })
    .await
    .unwrap();

    let item_id = s.list_inbox("bob").await.unwrap()[0].id;

    // First failure — item stays pending (retry_count = 1, under max_retries = 3).
    s.ack_inbox_item(AckInboxItemInput {
        item_id,
        agent_id: "bob".to_string(),
        status: InboxStatus::Failed,
    })
    .await
    .unwrap();

    let inbox = s.list_inbox("bob").await.unwrap();
    assert_eq!(inbox.len(), 1, "item should be re-queued after first failure");
    assert_eq!(inbox[0].status, InboxStatus::Pending);
}

#[tokio::test]
async fn test_inbox_permanent_failure_after_max_retries() {
    let s = make_storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::ReviewRequest,
        payload: serde_json::json!({}),
        deliver_to_inbox: true,
    })
    .await
    .unwrap();

    let item_id = s.list_inbox("bob").await.unwrap()[0].id;

    // Fail 3 times — the default max_retries.
    for _ in 0..3 {
        s.ack_inbox_item(AckInboxItemInput {
            item_id,
            agent_id: "bob".to_string(),
            status: InboxStatus::Failed,
        })
        .await
        .unwrap();
    }

    let inbox = s.list_inbox("bob").await.unwrap();
    assert!(
        inbox.is_empty(),
        "item should be permanently failed after max retries"
    );
}
