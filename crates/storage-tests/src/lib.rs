//! Shared integration test suite for all WorkspaceStorage implementations.
//!
//! ## Usage
//!
//! In your crate's `tests/integration.rs`:
//!
//! ```rust,ignore
//! async fn make_storage() -> impl aw_storage_tests::WorkspaceStorage + Send + Sync {
//!     // create your storage here
//! }
//!
//! aw_storage_tests::define_storage_tests!(make_storage);
//! ```
//!
//! The macro expands to one `#[tokio::test]` per test function, each calling
//! `make_storage().await` to get a fresh, isolated storage instance.

pub use aw_domain::entities::*;
pub use aw_domain::error::WorkspaceError;
pub use aw_domain::storage::WorkspaceStorage;
pub use uuid::Uuid;

// ── Test input builders ───────────────────────────────────────────────────────

pub fn mk_agent(id: &str) -> CreateAgentInput {
    CreateAgentInput {
        id: id.to_string(),
        name: format!("{} name", id),
        role: "worker".to_string(),
        capabilities: vec!["test".to_string()],
        permissions: vec![],
        metadata: None,
    }
}

pub fn mk_session(agent_id: &str) -> CheckInInput {
    CheckInInput {
        agent_id: agent_id.to_string(),
        metadata: None,
    }
}

pub fn mk_task(title: &str) -> CreateTaskInput {
    CreateTaskInput {
        title: title.to_string(),
        description: "desc".to_string(),
        kind: TaskKind::Analysis,
        priority: TaskPriority::Normal,
        assigned_agent_id: None,
        source_ref: None,
        metadata: None,
    }
}

pub fn mk_lock(agent_id: &str, session_id: Uuid) -> AcquireLockInput {
    AcquireLockInput {
        scope_type: "document".to_string(),
        scope_id: "doc-42".to_string(),
        lock_type: LockType::WriteLock,
        owner_agent_id: agent_id.to_string(),
        owner_session_id: session_id,
        ttl_secs: 300,
        metadata: None,
    }
}

// ── Agents ────────────────────────────────────────────────────────────────────

pub async fn test_agent_create_and_get(s: &impl WorkspaceStorage) {
    let agent = s.create_agent(mk_agent("alice")).await.unwrap();
    assert_eq!(agent.id, "alice");
    assert_eq!(agent.role, "worker");

    let fetched = s.get_agent("alice").await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, "alice");
}

pub async fn test_agent_get_missing_returns_none(s: &impl WorkspaceStorage) {
    let result = s.get_agent("nobody").await.unwrap();
    assert!(result.is_none());
}

pub async fn test_agent_upsert_is_idempotent(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("bob")).await.unwrap();

    let updated = s
        .create_agent(CreateAgentInput {
            id: "bob".to_string(),
            name: "Bob Updated".to_string(),
            role: "coordinator".to_string(),
            capabilities: vec![],
            permissions: vec![],
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(updated.name, "Bob Updated");
    assert_eq!(updated.role, "coordinator");

    let all = s.list_agents().await.unwrap();
    let bobs: Vec<_> = all.iter().filter(|a| a.id == "bob").collect();
    assert_eq!(bobs.len(), 1);
}

pub async fn test_agent_list(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("a1")).await.unwrap();
    s.create_agent(mk_agent("a2")).await.unwrap();
    s.create_agent(mk_agent("a3")).await.unwrap();

    let all = s.list_agents().await.unwrap();
    assert_eq!(all.len(), 3);
}

// ── Sessions ──────────────────────────────────────────────────────────────────

pub async fn test_session_checkin_creates_session(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();

    let result = s.check_in(mk_session("alice")).await.unwrap();
    assert_eq!(result.session.agent_id, "alice");
    assert_eq!(result.session.status, SessionStatus::Active);
    assert!(result.inbox.is_empty());
    assert!(result.pending_tasks.is_empty());
    assert!(result.pending_handoffs.is_empty());
}

pub async fn test_session_checkin_returns_pending_tasks(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();

    s.create_task(CreateTaskInput {
        title: "do something".to_string(),
        description: "details".to_string(),
        kind: TaskKind::Analysis,
        priority: TaskPriority::Normal,
        assigned_agent_id: Some("alice".to_string()),
        source_ref: None,
        metadata: None,
    })
    .await
    .unwrap();

    let result = s.check_in(mk_session("alice")).await.unwrap();
    assert_eq!(result.pending_tasks.len(), 1);
    assert_eq!(result.pending_tasks[0].title, "do something");
}

pub async fn test_session_heartbeat(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();
    let session_id = ci.session.id;

    let updated = s
        .heartbeat(HeartbeatInput {
            session_id,
            health: Some(SessionHealth::Degraded),
            current_task_id: None,
        })
        .await
        .unwrap();

    assert_eq!(updated.health, SessionHealth::Degraded);
    assert!(updated.last_seen_at >= ci.session.last_seen_at);
}

pub async fn test_session_checkout_ends_session(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();
    let session_id = ci.session.id;

    s.check_out(CheckOutInput {
        session_id,
        create_handoff: false,
        handoff_summary: None,
        handoff_payload: None,
    })
    .await
    .unwrap();

    let session = s.get_session(session_id).await.unwrap().unwrap();
    assert_eq!(session.status, SessionStatus::CheckedOut);
    assert!(session.ended_at.is_some());
}

pub async fn test_list_active_sessions(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("a")).await.unwrap();
    s.create_agent(mk_agent("b")).await.unwrap();
    s.create_agent(mk_agent("c")).await.unwrap();

    s.check_in(mk_session("a")).await.unwrap();
    let ci_b = s.check_in(mk_session("b")).await.unwrap();
    s.check_in(mk_session("c")).await.unwrap();

    s.check_out(CheckOutInput {
        session_id: ci_b.session.id,
        create_handoff: false,
        handoff_summary: None,
        handoff_payload: None,
    })
    .await
    .unwrap();

    let active = s.list_active_sessions().await.unwrap();
    let ids: Vec<_> = active.iter().map(|s| &s.agent_id).collect();
    assert_eq!(active.len(), 2);
    assert!(!ids.contains(&&"b".to_string()));
}

// ── Messages ──────────────────────────────────────────────────────────────────

pub async fn test_send_message_requires_existing_recipient(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();

    let err = s
        .send_message(SendMessageInput {
            workspace_id: "main".to_string(),
            from_agent_id: "alice".to_string(),
            to_agent_id: Some("nobody".to_string()),
            channel_id: None,
            thread_id: None,
            kind: MessageKind::ChatMessage,
            payload: serde_json::json!({"text": "hello"}),
            deliver_to_inbox: false,
        })
        .await;

    assert!(err.is_err());
}

pub async fn test_send_message_and_list_by_channel(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: Some("general".to_string()),
        thread_id: None,
        kind: MessageKind::ChatMessage,
        payload: serde_json::json!({"text": "hi bob"}),
        deliver_to_inbox: false,
    })
    .await
    .unwrap();

    let msgs = s.list_messages("general", 10).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].from_agent_id, "alice");
}

pub async fn test_list_messages_for_agent_includes_sent_and_received(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::ChatMessage,
        payload: serde_json::json!({}),
        deliver_to_inbox: false,
    })
    .await
    .unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "bob".to_string(),
        to_agent_id: Some("alice".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::StatusUpdate,
        payload: serde_json::json!({}),
        deliver_to_inbox: false,
    })
    .await
    .unwrap();

    let alice_msgs = s.list_messages_for_agent("alice", 10).await.unwrap();
    assert_eq!(alice_msgs.len(), 2);
    let bob_msgs = s.list_messages_for_agent("bob", 10).await.unwrap();
    assert_eq!(bob_msgs.len(), 2);
}

// ── Inbox ─────────────────────────────────────────────────────────────────────

pub async fn test_inbox_ack_done(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::Alert,
        payload: serde_json::json!({"text": "urgent"}),
        deliver_to_inbox: true,
    })
    .await
    .unwrap();

    let inbox = s.list_inbox("bob").await.unwrap();
    assert_eq!(inbox.len(), 1);
    let item_id = inbox[0].id;

    s.ack_inbox_item(AckInboxItemInput {
        item_id,
        agent_id: "bob".to_string(),
        status: InboxStatus::Done,
    })
    .await
    .unwrap();

    let inbox_after = s.list_inbox("bob").await.unwrap();
    assert!(inbox_after.is_empty());
}

// ── Tasks ─────────────────────────────────────────────────────────────────────

pub async fn test_task_create_and_list(s: &impl WorkspaceStorage) {
    s.create_task(mk_task("task-1")).await.unwrap();
    s.create_task(mk_task("task-2")).await.unwrap();

    let tasks = s.list_tasks(ListTasksFilter::default()).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

pub async fn test_task_list_filter_unassigned(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();

    s.create_task(mk_task("unassigned")).await.unwrap();
    s.create_task(CreateTaskInput {
        assigned_agent_id: Some("alice".to_string()),
        ..mk_task("assigned")
    })
    .await
    .unwrap();

    let unassigned = s
        .list_tasks(ListTasksFilter {
            unassigned_only: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(unassigned.len(), 1);
    assert_eq!(unassigned[0].title, "unassigned");
}

pub async fn test_task_list_filter_by_status(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();

    let t1 = s.create_task(mk_task("open-task")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();

    s.claim_task(ClaimTaskInput {
        task_id: t1.id,
        agent_id: "alice".to_string(),
        session_id: ci.session.id,
    })
    .await
    .unwrap();

    s.create_task(mk_task("another-open")).await.unwrap();

    let open_tasks = s
        .list_tasks(ListTasksFilter {
            statuses: Some(vec![TaskStatus::Open]),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(open_tasks.len(), 1);
    assert_eq!(open_tasks[0].title, "another-open");

    let claimed_tasks = s
        .list_tasks(ListTasksFilter {
            statuses: Some(vec![TaskStatus::Claimed]),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(claimed_tasks.len(), 1);
    assert_eq!(claimed_tasks[0].title, "open-task");
}

pub async fn test_task_claim(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();
    let task = s.create_task(mk_task("work")).await.unwrap();

    let claimed = s
        .claim_task(ClaimTaskInput {
            task_id: task.id,
            agent_id: "alice".to_string(),
            session_id: ci.session.id,
        })
        .await
        .unwrap();

    assert_eq!(claimed.status, TaskStatus::Claimed);
    assert_eq!(claimed.assigned_agent_id.as_deref(), Some("alice"));
}

pub async fn test_task_claim_already_claimed_fails(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    let ci_a = s.check_in(mk_session("alice")).await.unwrap();
    let ci_b = s.check_in(mk_session("bob")).await.unwrap();
    let task = s.create_task(mk_task("contested")).await.unwrap();

    s.claim_task(ClaimTaskInput {
        task_id: task.id,
        agent_id: "alice".to_string(),
        session_id: ci_a.session.id,
    })
    .await
    .unwrap();

    let err = s
        .claim_task(ClaimTaskInput {
            task_id: task.id,
            agent_id: "bob".to_string(),
            session_id: ci_b.session.id,
        })
        .await;

    assert!(err.is_err());
}

pub async fn test_task_assign_by_coordinator(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("coordinator")).await.unwrap();
    s.create_agent(mk_agent("worker")).await.unwrap();
    let task = s.create_task(mk_task("do-it")).await.unwrap();

    let assigned = s
        .assign_task(AssignTaskInput {
            task_id: task.id,
            assigned_by: "coordinator".to_string(),
            assigned_to: Some("worker".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(assigned.assigned_agent_id.as_deref(), Some("worker"));
    assert_eq!(assigned.status, TaskStatus::Claimed);
}

pub async fn test_task_unassign(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("coordinator")).await.unwrap();
    s.create_agent(mk_agent("worker")).await.unwrap();
    let task = s.create_task(mk_task("do-it")).await.unwrap();

    s.assign_task(AssignTaskInput {
        task_id: task.id,
        assigned_by: "coordinator".to_string(),
        assigned_to: Some("worker".to_string()),
    })
    .await
    .unwrap();

    let unassigned = s
        .assign_task(AssignTaskInput {
            task_id: task.id,
            assigned_by: "coordinator".to_string(),
            assigned_to: None,
        })
        .await
        .unwrap();

    assert!(unassigned.assigned_agent_id.is_none());
    assert_eq!(unassigned.status, TaskStatus::Open);
}

pub async fn test_task_update_status(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();
    let task = s.create_task(mk_task("work")).await.unwrap();

    s.claim_task(ClaimTaskInput {
        task_id: task.id,
        agent_id: "alice".to_string(),
        session_id: ci.session.id,
    })
    .await
    .unwrap();

    let done = s
        .update_task_status(UpdateTaskStatusInput {
            task_id: task.id,
            status: TaskStatus::Done,
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(done.status, TaskStatus::Done);
}

// ── Locks ─────────────────────────────────────────────────────────────────────

pub async fn test_lock_acquire_and_release(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();

    let lock = s.acquire_lock(mk_lock("alice", ci.session.id)).await.unwrap();
    assert_eq!(lock.owner_agent_id, "alice");

    s.release_lock(ReleaseLockInput {
        lock_id: lock.id,
        owner_session_id: ci.session.id,
    })
    .await
    .unwrap();

    // After release, the same scope can be locked again.
    let lock2 = s.acquire_lock(mk_lock("alice", ci.session.id)).await.unwrap();
    assert_ne!(lock.id, lock2.id);
}

pub async fn test_lock_conflict(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    let ci_a = s.check_in(mk_session("alice")).await.unwrap();
    let ci_b = s.check_in(mk_session("bob")).await.unwrap();

    s.acquire_lock(mk_lock("alice", ci_a.session.id)).await.unwrap();

    let err = s.acquire_lock(mk_lock("bob", ci_b.session.id)).await;
    assert!(matches!(err, Err(WorkspaceError::LockConflict(_))));
}

pub async fn test_expire_stale_locks(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();

    s.acquire_lock(AcquireLockInput {
        ttl_secs: 3600,
        ..mk_lock("alice", ci.session.id)
    })
    .await
    .unwrap();

    let expired = s.expire_stale_locks().await.unwrap();
    assert_eq!(expired, 0, "no locks should be expired yet");
}

// ── Events ────────────────────────────────────────────────────────────────────

pub async fn test_events_emitted_on_checkin(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.check_in(mk_session("alice")).await.unwrap();

    let events = s.list_events(Some("alice"), 10).await.unwrap();
    assert!(!events.is_empty(), "check_in should emit at least one event");
    let kinds: Vec<_> = events.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.contains(&"session.checked_in"));
}

pub async fn test_events_emitted_on_send_message(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::ChatMessage,
        payload: serde_json::json!({}),
        deliver_to_inbox: false,
    })
    .await
    .unwrap();

    let events = s.list_events(Some("alice"), 10).await.unwrap();
    let kinds: Vec<_> = events.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.contains(&"message.sent"));
}

pub async fn test_list_events_global(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    s.check_in(mk_session("alice")).await.unwrap();
    s.check_in(mk_session("bob")).await.unwrap();

    let all_events = s.list_events(None, 100).await.unwrap();
    assert!(all_events.len() >= 2, "should have events from both agents");
}

// ── Handoffs ──────────────────────────────────────────────────────────────────

pub async fn test_handoff_create_and_list(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();

    s.create_handoff(CreateHandoffInput {
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        source_session_id: ci.session.id,
        task_id: None,
        summary: "resuming market scan".to_string(),
        payload: Some(serde_json::json!({"last_price": 95000})),
    })
    .await
    .unwrap();

    let handoffs = s.list_handoffs("bob").await.unwrap();
    assert_eq!(handoffs.len(), 1);
    assert_eq!(handoffs[0].summary, "resuming market scan");
}

// ── Dependencies ──────────────────────────────────────────────────────────────

pub async fn test_dependency_upsert_and_get(s: &impl WorkspaceStorage) {
    s.upsert_dependency(UpsertDependencyInput {
        key: "db.primary".to_string(),
        state: DependencyState::Healthy,
        details: Some("latency 2ms".to_string()),
    })
    .await
    .unwrap();

    let dep = s.get_dependency("db.primary").await.unwrap();
    assert!(dep.is_some());
    assert_eq!(dep.unwrap().state, DependencyState::Healthy);
}

pub async fn test_dependency_upsert_updates_existing(s: &impl WorkspaceStorage) {
    s.upsert_dependency(UpsertDependencyInput {
        key: "svc.auth".to_string(),
        state: DependencyState::Healthy,
        details: None,
    })
    .await
    .unwrap();

    s.upsert_dependency(UpsertDependencyInput {
        key: "svc.auth".to_string(),
        state: DependencyState::Degraded,
        details: Some("high latency".to_string()),
    })
    .await
    .unwrap();

    let dep = s.get_dependency("svc.auth").await.unwrap().unwrap();
    assert_eq!(dep.state, DependencyState::Degraded);
    assert_eq!(dep.details.as_deref(), Some("high latency"));
}

// ── Workspace summary ─────────────────────────────────────────────────────────

pub async fn test_workspace_summary(s: &impl WorkspaceStorage) {
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    let ci_a = s.check_in(mk_session("alice")).await.unwrap();
    s.check_in(mk_session("bob")).await.unwrap();

    s.create_task(mk_task("t1")).await.unwrap();
    s.create_task(mk_task("t2")).await.unwrap();

    s.acquire_lock(mk_lock("alice", ci_a.session.id)).await.unwrap();

    s.send_message(SendMessageInput {
        workspace_id: "main".to_string(),
        from_agent_id: "alice".to_string(),
        to_agent_id: Some("bob".to_string()),
        channel_id: None,
        thread_id: None,
        kind: MessageKind::Alert,
        payload: serde_json::json!({}),
        deliver_to_inbox: true,
    })
    .await
    .unwrap();

    let summary = s.get_workspace_summary().await.unwrap();
    assert_eq!(summary.active_agents.len(), 2);
    assert_eq!(summary.open_tasks.len(), 2);
    assert_eq!(summary.pending_inbox_total, 1);
    assert_eq!(summary.active_locks_count, 1);
}

// ── Macro ─────────────────────────────────────────────────────────────────────

/// Generate the full integration test suite for a storage implementation.
///
/// `$make_storage` must be the name of an `async fn` in scope that returns
/// a type implementing `WorkspaceStorage`. It will be called once per test,
/// giving each test a fresh isolated storage instance.
///
/// # Example
/// ```rust,ignore
/// async fn make_storage() -> impl WorkspaceStorage { ... }
/// aw_storage_tests::define_storage_tests!(make_storage);
/// ```
#[macro_export]
macro_rules! define_storage_tests {
    ($make_storage:ident) => {
        // ── Agents ────────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_agent_create_and_get() {
            aw_storage_tests::test_agent_create_and_get(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_agent_get_missing_returns_none() {
            aw_storage_tests::test_agent_get_missing_returns_none(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_agent_upsert_is_idempotent() {
            aw_storage_tests::test_agent_upsert_is_idempotent(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_agent_list() {
            aw_storage_tests::test_agent_list(&$make_storage().await).await;
        }

        // ── Sessions ──────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_session_checkin_creates_session() {
            aw_storage_tests::test_session_checkin_creates_session(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_session_checkin_returns_pending_tasks() {
            aw_storage_tests::test_session_checkin_returns_pending_tasks(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_session_heartbeat() {
            aw_storage_tests::test_session_heartbeat(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_session_checkout_ends_session() {
            aw_storage_tests::test_session_checkout_ends_session(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_list_active_sessions() {
            aw_storage_tests::test_list_active_sessions(&$make_storage().await).await;
        }

        // ── Messages ──────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_send_message_requires_existing_recipient() {
            aw_storage_tests::test_send_message_requires_existing_recipient(
                &$make_storage().await,
            )
            .await;
        }
        #[tokio::test]
        async fn test_send_message_and_list_by_channel() {
            aw_storage_tests::test_send_message_and_list_by_channel(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_list_messages_for_agent_includes_sent_and_received() {
            aw_storage_tests::test_list_messages_for_agent_includes_sent_and_received(
                &$make_storage().await,
            )
            .await;
        }

        // ── Inbox ─────────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_inbox_ack_done() {
            aw_storage_tests::test_inbox_ack_done(&$make_storage().await).await;
        }

        // ── Tasks ─────────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_task_create_and_list() {
            aw_storage_tests::test_task_create_and_list(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_list_filter_unassigned() {
            aw_storage_tests::test_task_list_filter_unassigned(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_list_filter_by_status() {
            aw_storage_tests::test_task_list_filter_by_status(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_claim() {
            aw_storage_tests::test_task_claim(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_claim_already_claimed_fails() {
            aw_storage_tests::test_task_claim_already_claimed_fails(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_assign_by_coordinator() {
            aw_storage_tests::test_task_assign_by_coordinator(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_unassign() {
            aw_storage_tests::test_task_unassign(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_task_update_status() {
            aw_storage_tests::test_task_update_status(&$make_storage().await).await;
        }

        // ── Locks ─────────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_lock_acquire_and_release() {
            aw_storage_tests::test_lock_acquire_and_release(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_lock_conflict() {
            aw_storage_tests::test_lock_conflict(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_expire_stale_locks() {
            aw_storage_tests::test_expire_stale_locks(&$make_storage().await).await;
        }

        // ── Events ────────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_events_emitted_on_checkin() {
            aw_storage_tests::test_events_emitted_on_checkin(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_events_emitted_on_send_message() {
            aw_storage_tests::test_events_emitted_on_send_message(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_list_events_global() {
            aw_storage_tests::test_list_events_global(&$make_storage().await).await;
        }

        // ── Handoffs ──────────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_handoff_create_and_list() {
            aw_storage_tests::test_handoff_create_and_list(&$make_storage().await).await;
        }

        // ── Dependencies ──────────────────────────────────────────────────────
        #[tokio::test]
        async fn test_dependency_upsert_and_get() {
            aw_storage_tests::test_dependency_upsert_and_get(&$make_storage().await).await;
        }
        #[tokio::test]
        async fn test_dependency_upsert_updates_existing() {
            aw_storage_tests::test_dependency_upsert_updates_existing(&$make_storage().await)
                .await;
        }

        // ── Workspace summary ─────────────────────────────────────────────────
        #[tokio::test]
        async fn test_workspace_summary() {
            aw_storage_tests::test_workspace_summary(&$make_storage().await).await;
        }
    };
}
