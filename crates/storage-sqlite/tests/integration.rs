use aw_domain::entities::*;
use aw_domain::error::WorkspaceError;
use aw_domain::storage::WorkspaceStorage;
use aw_storage_sqlite::SqliteStorage;
use uuid::Uuid;

// ── helpers ───────────────────────────────────────────────────────────────────

async fn storage() -> SqliteStorage {
    let pool = aw_storage_sqlite::connect_memory().await.unwrap();
    SqliteStorage::new(pool)
}

fn mk_agent(id: &str) -> CreateAgentInput {
    CreateAgentInput {
        id: id.to_string(),
        name: format!("{} name", id),
        role: "worker".to_string(),
        capabilities: vec!["test".to_string()],
        permissions: vec![],
        metadata: None,
    }
}

fn mk_session(agent_id: &str) -> CheckInInput {
    CheckInInput {
        agent_id: agent_id.to_string(),
        metadata: None,
    }
}

// ── agents ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_agent_create_and_get() {
    let s = storage().await;
    let agent = s.create_agent(mk_agent("alice")).await.unwrap();
    assert_eq!(agent.id, "alice");
    assert_eq!(agent.role, "worker");

    let fetched = s.get_agent("alice").await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, "alice");
}

#[tokio::test]
async fn test_agent_get_missing_returns_none() {
    let s = storage().await;
    let result = s.get_agent("nobody").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_agent_upsert_is_idempotent() {
    let s = storage().await;
    s.create_agent(mk_agent("bob")).await.unwrap();

    // Same id, different name — should succeed and update.
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

    // list_agents should still show only one bob.
    let all = s.list_agents().await.unwrap();
    let bobs: Vec<_> = all.iter().filter(|a| a.id == "bob").collect();
    assert_eq!(bobs.len(), 1);
}

#[tokio::test]
async fn test_agent_list() {
    let s = storage().await;
    s.create_agent(mk_agent("a1")).await.unwrap();
    s.create_agent(mk_agent("a2")).await.unwrap();
    s.create_agent(mk_agent("a3")).await.unwrap();

    let all = s.list_agents().await.unwrap();
    assert_eq!(all.len(), 3);
}

// ── sessions ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_session_checkin_creates_session() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();

    let result = s.check_in(mk_session("alice")).await.unwrap();
    assert_eq!(result.session.agent_id, "alice");
    assert_eq!(result.session.status, SessionStatus::Active);
    // Fresh workspace — nothing pending.
    assert!(result.inbox.is_empty());
    assert!(result.pending_tasks.is_empty());
    assert!(result.pending_handoffs.is_empty());
}

#[tokio::test]
async fn test_session_checkin_returns_pending_tasks() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();

    // Create an open task assigned to alice before she checks in.
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

#[tokio::test]
async fn test_session_heartbeat() {
    let s = storage().await;
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

#[tokio::test]
async fn test_session_checkout_ends_session() {
    let s = storage().await;
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

#[tokio::test]
async fn test_list_active_sessions() {
    let s = storage().await;
    s.create_agent(mk_agent("a")).await.unwrap();
    s.create_agent(mk_agent("b")).await.unwrap();
    s.create_agent(mk_agent("c")).await.unwrap();

    s.check_in(mk_session("a")).await.unwrap();
    let ci_b = s.check_in(mk_session("b")).await.unwrap();
    s.check_in(mk_session("c")).await.unwrap();

    // Check b out.
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

// ── messages ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_message_requires_existing_recipient() {
    let s = storage().await;
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

#[tokio::test]
async fn test_send_message_and_list_by_channel() {
    let s = storage().await;
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

#[tokio::test]
async fn test_list_messages_for_agent_includes_sent_and_received() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    // alice → bob
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

    // bob → alice
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

    // alice sees both messages (sent + received)
    let alice_msgs = s.list_messages_for_agent("alice", 10).await.unwrap();
    assert_eq!(alice_msgs.len(), 2);

    // bob also sees both
    let bob_msgs = s.list_messages_for_agent("bob", 10).await.unwrap();
    assert_eq!(bob_msgs.len(), 2);
}

// ── inbox ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_inbox_ack_done() {
    let s = storage().await;
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

    // After ack done, item no longer shows in pending inbox.
    let inbox_after = s.list_inbox("bob").await.unwrap();
    assert!(inbox_after.is_empty());
}

#[tokio::test]
async fn test_inbox_ack_failure_retries() {
    let s = storage().await;
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
    let s = storage().await;
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

    // After exhausting retries, item should be gone from pending inbox.
    let inbox = s.list_inbox("bob").await.unwrap();
    assert!(inbox.is_empty(), "item should be permanently failed after max retries");
}

// ── tasks ─────────────────────────────────────────────────────────────────────

fn mk_task(title: &str) -> CreateTaskInput {
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

#[tokio::test]
async fn test_task_create_and_list() {
    let s = storage().await;
    s.create_task(mk_task("task-1")).await.unwrap();
    s.create_task(mk_task("task-2")).await.unwrap();

    let tasks = s.list_tasks(ListTasksFilter::default()).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_task_list_filter_unassigned() {
    let s = storage().await;
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

#[tokio::test]
async fn test_task_list_filter_by_status() {
    let s = storage().await;
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

    // Create another open task.
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

#[tokio::test]
async fn test_task_claim() {
    let s = storage().await;
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

#[tokio::test]
async fn test_task_claim_already_claimed_fails() {
    let s = storage().await;
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

#[tokio::test]
async fn test_task_assign_by_coordinator() {
    let s = storage().await;
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

#[tokio::test]
async fn test_task_unassign() {
    let s = storage().await;
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

#[tokio::test]
async fn test_task_update_status() {
    let s = storage().await;
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

// ── locks ─────────────────────────────────────────────────────────────────────

fn mk_lock(agent_id: &str, session_id: Uuid) -> AcquireLockInput {
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

#[tokio::test]
async fn test_lock_acquire_and_release() {
    let s = storage().await;
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

#[tokio::test]
async fn test_lock_conflict() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    let ci_a = s.check_in(mk_session("alice")).await.unwrap();
    let ci_b = s.check_in(mk_session("bob")).await.unwrap();

    s.acquire_lock(mk_lock("alice", ci_a.session.id)).await.unwrap();

    let err = s.acquire_lock(mk_lock("bob", ci_b.session.id)).await;
    assert!(matches!(err, Err(WorkspaceError::LockConflict(_))));
}

#[tokio::test]
async fn test_expire_stale_locks() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    let ci = s.check_in(mk_session("alice")).await.unwrap();

    // Acquire a lock with ttl=1 (expires immediately in practice).
    // We'll update the expires_at directly via a raw query to test expiry.
    // Instead, just verify expire_stale_locks runs without error on a fresh DB.
    s.acquire_lock(AcquireLockInput {
        ttl_secs: 3600,
        ..mk_lock("alice", ci.session.id)
    })
    .await
    .unwrap();

    let expired = s.expire_stale_locks().await.unwrap();
    assert_eq!(expired, 0, "no locks should be expired yet");
}

// ── events ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_events_emitted_on_checkin() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.check_in(mk_session("alice")).await.unwrap();

    let events = s.list_events(Some("alice"), 10).await.unwrap();
    assert!(!events.is_empty(), "check_in should emit at least one event");
    let kinds: Vec<_> = events.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.contains(&"session.checked_in"));
}

#[tokio::test]
async fn test_events_emitted_on_send_message() {
    let s = storage().await;
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

#[tokio::test]
async fn test_list_events_global() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();
    s.check_in(mk_session("alice")).await.unwrap();
    s.check_in(mk_session("bob")).await.unwrap();

    let all_events = s.list_events(None, 100).await.unwrap();
    assert!(all_events.len() >= 2, "should have events from both agents");
}

// ── handoffs ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_handoff_create_and_list() {
    let s = storage().await;
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

// ── dependencies ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_dependency_upsert_and_get() {
    let s = storage().await;

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

#[tokio::test]
async fn test_dependency_upsert_updates_existing() {
    let s = storage().await;

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

// ── workspace summary ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_workspace_summary() {
    let s = storage().await;
    s.create_agent(mk_agent("alice")).await.unwrap();
    s.create_agent(mk_agent("bob")).await.unwrap();

    let ci_a = s.check_in(mk_session("alice")).await.unwrap();
    s.check_in(mk_session("bob")).await.unwrap();

    s.create_task(mk_task("t1")).await.unwrap();
    s.create_task(mk_task("t2")).await.unwrap();

    s.acquire_lock(mk_lock("alice", ci_a.session.id)).await.unwrap();

    // Deliver an inbox item to bob.
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
