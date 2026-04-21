//! PostgreSQL implementation of WorkspaceStorage.
//! Uses native Postgres types: UUID, TIMESTAMPTZ, JSONB, BOOLEAN.
//! Placeholders use $N syntax.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use aw_domain::entities::*;
use aw_domain::error::{Result, WorkspaceError};
use aw_domain::storage::WorkspaceStorage;

use crate::rows::*;

const HEARTBEAT_TIMEOUT_SECS: i64 = 300;

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn emit(&self, agent_id: Option<&str>, session_id: Option<Uuid>, kind: &str, payload: serde_json::Value) {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let _ = sqlx::query(
            "INSERT INTO events (id, agent_id, session_id, kind, payload, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(agent_id)
        .bind(session_id)
        .bind(kind)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await;
    }
}

// ─── Row structs ──────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct AgentRow {
    id: String,
    name: String,
    role: String,
    capabilities: serde_json::Value,
    permissions: serde_json::Value,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AgentRow> for Agent {
    fn from(r: AgentRow) -> Self {
        Agent {
            id: r.id,
            name: r.name,
            role: r.role,
            capabilities: serde_json::from_value(r.capabilities).unwrap_or_default(),
            permissions: serde_json::from_value(r.permissions).unwrap_or_default(),
            status: parse_agent_status(&r.status),
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    agent_id: String,
    status: String,
    started_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    health: String,
    current_task_id: Option<Uuid>,
    metadata: serde_json::Value,
}

impl From<SessionRow> for AgentSession {
    fn from(r: SessionRow) -> Self {
        AgentSession {
            id: r.id,
            agent_id: r.agent_id,
            status: parse_session_status(&r.status),
            started_at: r.started_at,
            last_seen_at: r.last_seen_at,
            ended_at: r.ended_at,
            health: parse_session_health(&r.health),
            current_task_id: r.current_task_id,
            metadata: r.metadata,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    workspace_id: String,
    channel_id: Option<String>,
    thread_id: Option<Uuid>,
    from_agent_id: String,
    to_agent_id: Option<String>,
    kind: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<MessageRow> for Message {
    fn from(r: MessageRow) -> Self {
        Message {
            id: r.id,
            workspace_id: r.workspace_id,
            channel_id: r.channel_id,
            thread_id: r.thread_id,
            from_agent_id: r.from_agent_id,
            to_agent_id: r.to_agent_id,
            kind: parse_message_kind(&r.kind),
            payload: r.payload,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct InboxRow {
    id: Uuid,
    target_agent_id: String,
    source_agent_id: Option<String>,
    kind: String,
    status: String,
    payload: serde_json::Value,
    deliver_on_checkin: bool,
    created_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
}

impl From<InboxRow> for InboxItem {
    fn from(r: InboxRow) -> Self {
        InboxItem {
            id: r.id,
            target_agent_id: r.target_agent_id,
            source_agent_id: r.source_agent_id,
            kind: parse_message_kind(&r.kind),
            status: parse_inbox_status(&r.status),
            payload: r.payload,
            deliver_on_checkin: r.deliver_on_checkin,
            created_at: r.created_at,
            processed_at: r.processed_at,
            expires_at: r.expires_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    title: String,
    description: String,
    kind: String,
    status: String,
    priority: String,
    assigned_agent_id: Option<String>,
    source_ref: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TaskRow> for Task {
    fn from(r: TaskRow) -> Self {
        Task {
            id: r.id,
            title: r.title,
            description: r.description,
            kind: serde_json::from_str(&r.kind).unwrap_or(TaskKind::Custom(r.kind.clone())),
            status: parse_task_status(&r.status),
            priority: parse_task_priority(&r.priority),
            assigned_agent_id: r.assigned_agent_id,
            source_ref: r.source_ref,
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LockRow {
    id: Uuid,
    scope_type: String,
    scope_id: String,
    lock_type: String,
    owner_agent_id: String,
    owner_session_id: Uuid,
    acquired_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    metadata: serde_json::Value,
}

impl From<LockRow> for Lock {
    fn from(r: LockRow) -> Self {
        Lock {
            id: r.id,
            scope_type: r.scope_type,
            scope_id: r.scope_id,
            lock_type: parse_lock_type(&r.lock_type),
            owner_agent_id: r.owner_agent_id,
            owner_session_id: r.owner_session_id,
            acquired_at: r.acquired_at,
            expires_at: r.expires_at,
            metadata: r.metadata,
        }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    workspace_id: Option<String>,
    agent_id: Option<String>,
    session_id: Option<Uuid>,
    kind: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<EventRow> for Event {
    fn from(r: EventRow) -> Self {
        Event {
            id: r.id,
            workspace_id: r.workspace_id,
            agent_id: r.agent_id,
            session_id: r.session_id,
            kind: r.kind,
            payload: r.payload,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct HandoffRow {
    id: Uuid,
    from_agent_id: String,
    to_agent_id: Option<String>,
    source_session_id: Uuid,
    task_id: Option<Uuid>,
    summary: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<HandoffRow> for Handoff {
    fn from(r: HandoffRow) -> Self {
        Handoff {
            id: r.id,
            from_agent_id: r.from_agent_id,
            to_agent_id: r.to_agent_id,
            source_session_id: r.source_session_id,
            task_id: r.task_id,
            summary: r.summary,
            payload: r.payload,
            created_at: r.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DependencyRow {
    key: String,
    state: String,
    details: Option<String>,
    checked_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<DependencyRow> for Dependency {
    fn from(r: DependencyRow) -> Self {
        Dependency {
            key: r.key,
            state: parse_dep_state(&r.state),
            details: r.details,
            checked_at: r.checked_at,
            updated_at: r.updated_at,
        }
    }
}

// ─── WorkspaceStorage impl ────────────────────────────────────────────────────

#[async_trait]
impl WorkspaceStorage for PostgresStorage {
    // ── Agents ────────────────────────────────────────────────────────────────

    async fn create_agent(&self, input: CreateAgentInput) -> Result<Agent> {
        let now = Utc::now();
        let caps = serde_json::to_value(&input.capabilities).unwrap_or(serde_json::json!([]));
        let perms = serde_json::to_value(&input.permissions).unwrap_or(serde_json::json!([]));
        let meta = serde_json::to_value(input.metadata.unwrap_or_default()).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO agents (id, name, role, capabilities, permissions, status, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'offline', $6, $7, $7)
             ON CONFLICT(id) DO UPDATE SET
               name = EXCLUDED.name,
               role = EXCLUDED.role,
               capabilities = EXCLUDED.capabilities,
               permissions = EXCLUDED.permissions,
               metadata = EXCLUDED.metadata,
               updated_at = EXCLUDED.updated_at",
        )
        .bind(&input.id)
        .bind(&input.name)
        .bind(&input.role)
        .bind(&caps)
        .bind(&perms)
        .bind(&meta)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        self.get_agent(&input.id).await?.ok_or_else(|| {
            WorkspaceError::Storage(anyhow::anyhow!("agent not found after insert"))
        })
    }

    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>> {
        let row = sqlx::query_as::<_, AgentRow>(
            "SELECT id, name, role, capabilities, permissions, status, metadata, created_at, updated_at
             FROM agents WHERE id = $1",
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(row.map(Agent::from))
    }

    async fn list_agents(&self) -> Result<Vec<Agent>> {
        let rows = sqlx::query_as::<_, AgentRow>(
            "SELECT id, name, role, capabilities, permissions, status, metadata, created_at, updated_at
             FROM agents ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Agent::from).collect())
    }

    // ── Maintenance ───────────────────────────────────────────────────────────

    async fn sweep_dead_sessions(&self, timeout_secs: u64) -> Result<u64> {
        let now = Utc::now();
        let cutoff = now - chrono::Duration::seconds(timeout_secs as i64);

        sqlx::query(
            "DELETE FROM locks WHERE owner_session_id IN (
                SELECT id FROM agent_sessions WHERE status = 'active' AND last_seen_at < $1
             )",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let swept = sqlx::query(
            "UPDATE agent_sessions SET status = 'dead'
             WHERE status = 'active' AND last_seen_at < $1",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .rows_affected();

        sqlx::query(
            "UPDATE agents SET status = 'offline', updated_at = $1
             WHERE status = 'active'
               AND id NOT IN (SELECT DISTINCT agent_id FROM agent_sessions WHERE status = 'active')",
        )
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(swept)
    }

    // ── Sessions ──────────────────────────────────────────────────────────────

    async fn check_in(&self, input: CheckInInput) -> Result<CheckInResult> {
        let now = Utc::now();

        let dead_sessions_swept = self.sweep_dead_sessions(HEARTBEAT_TIMEOUT_SECS as u64).await?;
        let locks_expired = self.expire_stale_locks().await?;

        let id = Uuid::new_v4();
        let meta = serde_json::to_value(input.metadata.unwrap_or_default()).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO agent_sessions (id, agent_id, status, started_at, last_seen_at, health, metadata)
             VALUES ($1, $2, 'active', $3, $3, 'healthy', $4)",
        )
        .bind(id)
        .bind(&input.agent_id)
        .bind(now)
        .bind(&meta)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        sqlx::query("UPDATE agents SET status = 'active', updated_at = $1 WHERE id = $2")
            .bind(now)
            .bind(&input.agent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let session = self.get_session(id).await?.ok_or_else(|| {
            WorkspaceError::Storage(anyhow::anyhow!("session not found after check-in"))
        })?;

        let inbox = self.list_inbox(&input.agent_id).await?;

        let pending_tasks: Vec<Task> = sqlx::query_as::<_, TaskRow>(
            "SELECT id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at
             FROM tasks
             WHERE assigned_agent_id = $1 AND status NOT IN ('done', 'failed', 'cancelled')
             ORDER BY created_at ASC",
        )
        .bind(&input.agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .into_iter()
        .map(Task::from)
        .collect();

        let pending_handoffs = self.list_handoffs(&input.agent_id).await?;

        self.emit(
            Some(&input.agent_id),
            Some(session.id),
            "session.checked_in",
            serde_json::json!({
                "session_id": session.id,
                "inbox_count": inbox.len(),
                "pending_tasks": pending_tasks.len() as i64,
                "dead_sessions_swept": dead_sessions_swept,
                "locks_expired": locks_expired,
            }),
        ).await;

        Ok(CheckInResult {
            session,
            inbox,
            pending_tasks,
            pending_handoffs,
            dead_sessions_swept,
            locks_expired,
        })
    }

    async fn heartbeat(&self, input: HeartbeatInput) -> Result<AgentSession> {
        let now = Utc::now();
        let health = fmt_session_health(&input.health.unwrap_or(SessionHealth::Healthy));

        sqlx::query(
            "UPDATE agent_sessions SET last_seen_at = $1, health = $2, current_task_id = $3
             WHERE id = $4",
        )
        .bind(now)
        .bind(health)
        .bind(input.current_task_id)
        .bind(input.session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        self.get_session(input.session_id).await?.ok_or_else(|| {
            WorkspaceError::NotFound(input.session_id.to_string())
        })
    }

    async fn check_out(&self, input: CheckOutInput) -> Result<()> {
        let now = Utc::now();

        let session = self
            .get_session(input.session_id)
            .await?
            .ok_or_else(|| WorkspaceError::NotFound(input.session_id.to_string()))?;

        sqlx::query(
            "UPDATE agent_sessions SET status = 'checked_out', ended_at = $1 WHERE id = $2",
        )
        .bind(now)
        .bind(input.session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        sqlx::query("UPDATE agents SET status = 'offline', updated_at = $1 WHERE id = $2")
            .bind(now)
            .bind(&session.agent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;

        if input.create_handoff {
            self.create_handoff(CreateHandoffInput {
                from_agent_id: session.agent_id.clone(),
                to_agent_id: None,
                source_session_id: input.session_id,
                task_id: session.current_task_id,
                summary: input.handoff_summary.unwrap_or_else(|| "check-out".to_string()),
                payload: input.handoff_payload,
            })
            .await?;
        }

        sqlx::query("DELETE FROM locks WHERE owner_session_id = $1")
            .bind(input.session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;

        self.emit(
            Some(&session.agent_id),
            Some(input.session_id),
            "session.checked_out",
            serde_json::json!({
                "session_id": input.session_id,
                "created_handoff": input.create_handoff,
            }),
        ).await;

        Ok(())
    }

    async fn get_session(&self, session_id: Uuid) -> Result<Option<AgentSession>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, agent_id, status, started_at, last_seen_at, ended_at, health, current_task_id, metadata
             FROM agent_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(row.map(AgentSession::from))
    }

    async fn active_session(&self, agent_id: &str) -> Result<Option<AgentSession>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, agent_id, status, started_at, last_seen_at, ended_at, health, current_task_id, metadata
             FROM agent_sessions WHERE agent_id = $1 AND status = 'active'
             ORDER BY started_at DESC LIMIT 1",
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(row.map(AgentSession::from))
    }

    async fn list_active_sessions(&self) -> Result<Vec<AgentSession>> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, agent_id, status, started_at, last_seen_at, ended_at, health, current_task_id, metadata
             FROM agent_sessions WHERE status = 'active'
             ORDER BY started_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(AgentSession::from).collect())
    }

    // ── Messages ──────────────────────────────────────────────────────────────

    async fn send_message(&self, input: SendMessageInput) -> Result<Message> {
        if let Some(ref to) = input.to_agent_id {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agents WHERE id = $1)")
                .bind(to)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| WorkspaceError::Storage(e.into()))?;
            if !exists {
                return Err(WorkspaceError::NotFound(format!("agent '{to}' does not exist")));
            }
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let kind_str = fmt_message_kind(&input.kind);
        let payload = serde_json::to_value(&input.payload).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO messages (id, workspace_id, channel_id, thread_id, from_agent_id, to_agent_id, kind, payload, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(&input.workspace_id)
        .bind(&input.channel_id)
        .bind(input.thread_id)
        .bind(&input.from_agent_id)
        .bind(&input.to_agent_id)
        .bind(kind_str)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        if input.deliver_to_inbox {
            if let Some(ref to) = input.to_agent_id {
                sqlx::query(
                    "INSERT INTO inbox_items (id, target_agent_id, source_agent_id, kind, status, payload, deliver_on_checkin, created_at)
                     VALUES ($1, $2, $3, $4, 'pending', $5, true, $6)",
                )
                .bind(Uuid::new_v4())
                .bind(to)
                .bind(&input.from_agent_id)
                .bind(kind_str)
                .bind(&payload)
                .bind(now)
                .execute(&self.pool)
                .await
                .map_err(|e| WorkspaceError::Storage(e.into()))?;
            }
        }

        let msg = Message {
            id,
            workspace_id: input.workspace_id,
            channel_id: input.channel_id,
            thread_id: input.thread_id,
            from_agent_id: input.from_agent_id,
            to_agent_id: input.to_agent_id,
            kind: input.kind,
            payload: input.payload,
            created_at: now,
        };

        self.emit(
            Some(&msg.from_agent_id),
            None,
            "message.sent",
            serde_json::json!({
                "message_id": msg.id,
                "to": msg.to_agent_id,
                "kind": fmt_message_kind(&msg.kind),
                "deliver_to_inbox": input.deliver_to_inbox,
            }),
        ).await;

        Ok(msg)
    }

    async fn list_messages(&self, channel_id: &str, limit: u32) -> Result<Vec<Message>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT id, workspace_id, channel_id, thread_id, from_agent_id, to_agent_id, kind, payload, created_at
             FROM messages WHERE channel_id = $1 ORDER BY created_at ASC LIMIT $2",
        )
        .bind(channel_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Message::from).collect())
    }

    async fn list_messages_for_agent(&self, agent_id: &str, limit: u32) -> Result<Vec<Message>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT id, workspace_id, channel_id, thread_id, from_agent_id, to_agent_id, kind, payload, created_at
             FROM messages WHERE from_agent_id = $1 OR to_agent_id = $1
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(agent_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Message::from).collect())
    }

    // ── Inbox ─────────────────────────────────────────────────────────────────

    async fn list_inbox(&self, agent_id: &str) -> Result<Vec<InboxItem>> {
        let rows = sqlx::query_as::<_, InboxRow>(
            "SELECT id, target_agent_id, source_agent_id, kind, status, payload, deliver_on_checkin,
                    created_at, processed_at, expires_at
             FROM inbox_items WHERE target_agent_id = $1 AND status IN ('pending', 'processing')
             ORDER BY created_at ASC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(InboxItem::from).collect())
    }

    async fn ack_inbox_item(&self, input: AckInboxItemInput) -> Result<()> {
        let now = Utc::now();
        let status = fmt_inbox_status(&input.status);

        sqlx::query(
            "UPDATE inbox_items SET status = $1, processed_at = $2
             WHERE id = $3 AND target_agent_id = $4",
        )
        .bind(status)
        .bind(now)
        .bind(input.item_id)
        .bind(&input.agent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(())
    }

    // ── Tasks ─────────────────────────────────────────────────────────────────

    async fn create_task(&self, input: CreateTaskInput) -> Result<Task> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let kind = serde_json::to_string(&input.kind).unwrap_or_default();
        let priority = fmt_task_priority(&input.priority);
        let meta = serde_json::to_value(input.metadata.clone().unwrap_or_default()).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO tasks (id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'open', $5, $6, $7, $8, $9, $9)",
        )
        .bind(id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&kind)
        .bind(priority)
        .bind(&input.assigned_agent_id)
        .bind(&input.source_ref)
        .bind(&meta)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        self.get_task(id).await?.ok_or_else(|| {
            WorkspaceError::Storage(anyhow::anyhow!("task not found after insert"))
        })
    }

    async fn get_task(&self, task_id: Uuid) -> Result<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>(
            "SELECT id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at
             FROM tasks WHERE id = $1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(row.map(Task::from))
    }

    async fn claim_task(&self, input: ClaimTaskInput) -> Result<Task> {
        let now = Utc::now();

        let affected = sqlx::query(
            "UPDATE tasks SET status = 'claimed', assigned_agent_id = $1, updated_at = $2
             WHERE id = $3 AND status = 'open'",
        )
        .bind(&input.agent_id)
        .bind(now)
        .bind(input.task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .rows_affected();

        if affected == 0 {
            return Err(WorkspaceError::LockConflict(format!(
                "task {} is not open or does not exist",
                input.task_id
            )));
        }

        let task = self.get_task(input.task_id).await?.ok_or_else(|| {
            WorkspaceError::NotFound(input.task_id.to_string())
        })?;

        self.emit(
            Some(&input.agent_id),
            None,
            "task.claimed",
            serde_json::json!({
                "task_id": task.id,
                "title": task.title,
                "agent_id": input.agent_id,
            }),
        ).await;

        Ok(task)
    }

    async fn update_task_status(&self, input: UpdateTaskStatusInput) -> Result<Task> {
        let now = Utc::now();
        let status = fmt_task_status(&input.status);

        sqlx::query("UPDATE tasks SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status)
            .bind(now)
            .bind(input.task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;

        self.get_task(input.task_id).await?.ok_or_else(|| {
            WorkspaceError::NotFound(input.task_id.to_string())
        })
    }

    async fn list_tasks_for_agent(&self, agent_id: &str) -> Result<Vec<Task>> {
        let rows = sqlx::query_as::<_, TaskRow>(
            "SELECT id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at
             FROM tasks WHERE assigned_agent_id = $1 ORDER BY created_at ASC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Task::from).collect())
    }

    async fn list_tasks(&self, filter: ListTasksFilter) -> Result<Vec<Task>> {
        let limit = filter.limit.unwrap_or(100) as i64;

        let mut conditions: Vec<String> = Vec::new();
        if filter.unassigned_only == Some(true) {
            conditions.push("assigned_agent_id IS NULL".into());
        } else if let Some(ref agent) = filter.assigned_to {
            conditions.push(format!("assigned_agent_id = '{}'", agent.replace('\'', "''")));
        }
        if let Some(ref statuses) = filter.statuses {
            if !statuses.is_empty() {
                let vals: Vec<String> = statuses.iter()
                    .map(|s| format!("'{}'", fmt_task_status(s)))
                    .collect();
                conditions.push(format!("status IN ({})", vals.join(",")));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at
             FROM tasks {where_clause}
             ORDER BY CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'normal' THEN 2 ELSE 3 END, created_at ASC
             LIMIT {limit}",
        );

        let rows = sqlx::query_as::<_, TaskRow>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Task::from).collect())
    }

    async fn assign_task(&self, input: AssignTaskInput) -> Result<Task> {
        let now = Utc::now();

        let affected = sqlx::query(
            "UPDATE tasks SET assigned_agent_id = $1, status = CASE
                WHEN $1 IS NOT NULL AND status = 'open' THEN 'claimed'
                WHEN $1 IS NULL THEN 'open'
                ELSE status
             END, updated_at = $2
             WHERE id = $3",
        )
        .bind(&input.assigned_to)
        .bind(now)
        .bind(input.task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .rows_affected();

        if affected == 0 {
            return Err(WorkspaceError::NotFound(input.task_id.to_string()));
        }

        let task = self.get_task(input.task_id).await?.ok_or_else(|| {
            WorkspaceError::NotFound(input.task_id.to_string())
        })?;

        self.emit(
            Some(&input.assigned_by),
            None,
            "task.assigned",
            serde_json::json!({
                "task_id": task.id,
                "title": task.title,
                "assigned_by": input.assigned_by,
                "assigned_to": input.assigned_to,
            }),
        ).await;

        Ok(task)
    }

    // ── Locks ─────────────────────────────────────────────────────────────────

    async fn acquire_lock(&self, input: AcquireLockInput) -> Result<Lock> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(input.ttl_secs as i64);
        let lock_type = fmt_lock_type(&input.lock_type);
        let metadata = input.metadata.unwrap_or_default();
        let meta = serde_json::to_value(&metadata).unwrap_or(serde_json::json!({}));

        let result = sqlx::query(
            "INSERT INTO locks (id, scope_type, scope_id, lock_type, owner_agent_id, owner_session_id, acquired_at, expires_at, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(&input.scope_type)
        .bind(&input.scope_id)
        .bind(lock_type)
        .bind(&input.owner_agent_id)
        .bind(input.owner_session_id)
        .bind(now)
        .bind(expires)
        .bind(&meta)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(Lock {
                id,
                scope_type: input.scope_type,
                scope_id: input.scope_id,
                lock_type: input.lock_type,
                owner_agent_id: input.owner_agent_id,
                owner_session_id: input.owner_session_id,
                acquired_at: now,
                expires_at: expires,
                metadata,
            }),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                Err(WorkspaceError::LockConflict(format!(
                    "{}/{} is already locked",
                    input.scope_type, input.scope_id
                )))
            }
            Err(e) => Err(WorkspaceError::Storage(e.into())),
        }
    }

    async fn renew_lock(&self, input: RenewLockInput) -> Result<Lock> {
        let expires = Utc::now() + chrono::Duration::seconds(input.ttl_secs as i64);

        let affected = sqlx::query(
            "UPDATE locks SET expires_at = $1 WHERE id = $2 AND owner_session_id = $3",
        )
        .bind(expires)
        .bind(input.lock_id)
        .bind(input.owner_session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .rows_affected();

        if affected == 0 {
            return Err(WorkspaceError::NotFound(input.lock_id.to_string()));
        }

        let row = sqlx::query_as::<_, LockRow>(
            "SELECT id, scope_type, scope_id, lock_type, owner_agent_id, owner_session_id, acquired_at, expires_at, metadata
             FROM locks WHERE id = $1",
        )
        .bind(input.lock_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(Lock::from(row))
    }

    async fn release_lock(&self, input: ReleaseLockInput) -> Result<()> {
        sqlx::query("DELETE FROM locks WHERE id = $1 AND owner_session_id = $2")
            .bind(input.lock_id)
            .bind(input.owner_session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?;
        Ok(())
    }

    async fn expire_stale_locks(&self) -> Result<u64> {
        let now = Utc::now();
        let affected = sqlx::query("DELETE FROM locks WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(e.into()))?
            .rows_affected();
        Ok(affected)
    }

    // ── Events ────────────────────────────────────────────────────────────────

    async fn append_event(&self, input: AppendEventInput) -> Result<Event> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let payload = serde_json::to_value(&input.payload).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO events (id, workspace_id, agent_id, session_id, kind, payload, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id)
        .bind(&input.workspace_id)
        .bind(&input.agent_id)
        .bind(input.session_id)
        .bind(&input.kind)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(Event {
            id,
            workspace_id: input.workspace_id,
            agent_id: input.agent_id,
            session_id: input.session_id,
            kind: input.kind,
            payload: input.payload,
            created_at: now,
        })
    }

    async fn list_events(&self, agent_id: Option<&str>, limit: u32) -> Result<Vec<Event>> {
        let rows = if let Some(aid) = agent_id {
            sqlx::query_as::<_, EventRow>(
                "SELECT id, workspace_id, agent_id, session_id, kind, payload, created_at
                 FROM events WHERE agent_id = $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(aid)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, EventRow>(
                "SELECT id, workspace_id, agent_id, session_id, kind, payload, created_at
                 FROM events ORDER BY created_at DESC LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Event::from).collect())
    }

    // ── Handoffs ──────────────────────────────────────────────────────────────

    async fn create_handoff(&self, input: CreateHandoffInput) -> Result<Handoff> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let handoff_payload = input.payload.unwrap_or_default();
        let payload = serde_json::to_value(&handoff_payload).unwrap_or(serde_json::json!({}));

        sqlx::query(
            "INSERT INTO handoffs (id, from_agent_id, to_agent_id, source_session_id, task_id, summary, payload, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(id)
        .bind(&input.from_agent_id)
        .bind(&input.to_agent_id)
        .bind(input.source_session_id)
        .bind(input.task_id)
        .bind(&input.summary)
        .bind(&payload)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(Handoff {
            id,
            from_agent_id: input.from_agent_id,
            to_agent_id: input.to_agent_id,
            source_session_id: input.source_session_id,
            task_id: input.task_id,
            summary: input.summary,
            payload: handoff_payload,
            created_at: now,
        })
    }

    async fn list_handoffs(&self, agent_id: &str) -> Result<Vec<Handoff>> {
        let rows = sqlx::query_as::<_, HandoffRow>(
            "SELECT id, from_agent_id, to_agent_id, source_session_id, task_id, summary, payload, created_at
             FROM handoffs WHERE from_agent_id = $1 OR to_agent_id = $1
             ORDER BY created_at DESC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Handoff::from).collect())
    }

    // ── Dependencies ──────────────────────────────────────────────────────────

    async fn upsert_dependency(&self, input: UpsertDependencyInput) -> Result<Dependency> {
        let now = Utc::now();
        let state = fmt_dep_state(&input.state);

        sqlx::query(
            "INSERT INTO dependencies (key, state, details, checked_at, updated_at)
             VALUES ($1, $2, $3, $4, $4)
             ON CONFLICT(key) DO UPDATE SET
               state = EXCLUDED.state,
               details = EXCLUDED.details,
               checked_at = EXCLUDED.checked_at,
               updated_at = EXCLUDED.updated_at",
        )
        .bind(&input.key)
        .bind(state)
        .bind(&input.details)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(Dependency {
            key: input.key,
            state: input.state,
            details: input.details,
            checked_at: now,
            updated_at: now,
        })
    }

    async fn get_dependency(&self, key: &str) -> Result<Option<Dependency>> {
        let row = sqlx::query_as::<_, DependencyRow>(
            "SELECT key, state, details, checked_at, updated_at FROM dependencies WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(row.map(Dependency::from))
    }

    async fn list_dependencies(&self) -> Result<Vec<Dependency>> {
        let rows = sqlx::query_as::<_, DependencyRow>(
            "SELECT key, state, details, checked_at, updated_at FROM dependencies ORDER BY key ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(rows.into_iter().map(Dependency::from).collect())
    }

    // ── Workspace summary ─────────────────────────────────────────────────────

    async fn get_workspace_summary(&self) -> Result<WorkspaceSummary> {
        let active_agents = sqlx::query_as::<_, AgentRow>(
            "SELECT id, name, role, capabilities, permissions, status, metadata, created_at, updated_at
             FROM agents WHERE status = 'active' ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .into_iter()
        .map(Agent::from)
        .collect();

        let open_tasks = sqlx::query_as::<_, TaskRow>(
            "SELECT id, title, description, kind, status, priority, assigned_agent_id, source_ref, metadata, created_at, updated_at
             FROM tasks WHERE status NOT IN ('done', 'failed', 'cancelled')
             ORDER BY CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'normal' THEN 2 ELSE 3 END, created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .into_iter()
        .map(Task::from)
        .collect();

        let pending_inbox_total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM inbox_items WHERE status IN ('pending', 'processing')",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let now = Utc::now();
        let active_locks_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM locks WHERE expires_at > $1",
        )
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(WorkspaceSummary {
            active_agents,
            open_tasks,
            pending_inbox_total: pending_inbox_total as u64,
            active_locks_count: active_locks_count as u64,
        })
    }

    // ── Reputation ────────────────────────────────────────────────────────────

    async fn upsert_review(&self, input: CreateReviewInput) -> Result<AgentReview> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO agent_reviews (id, agent_id, reviewer_id, score, review_text, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             ON CONFLICT(agent_id, reviewer_id) DO UPDATE SET
               score = EXCLUDED.score,
               review_text = EXCLUDED.review_text,
               updated_at = EXCLUDED.updated_at",
        )
        .bind(id)
        .bind(&input.agent_id)
        .bind(&input.reviewer_id)
        .bind(input.score as i32)
        .bind(&input.review_text)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let row = sqlx::query_as::<_, (Uuid, String, String, i32, Option<String>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, agent_id, reviewer_id, score, review_text, created_at, updated_at
             FROM agent_reviews WHERE agent_id = $1 AND reviewer_id = $2",
        )
        .bind(&input.agent_id)
        .bind(&input.reviewer_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(AgentReview {
            id: row.0,
            agent_id: row.1,
            reviewer_id: row.2,
            score: row.3 as u8,
            review_text: row.4,
            created_at: row.5,
            updated_at: row.6,
        })
    }

    async fn create_endorsement(&self, input: CreateEndorsementInput) -> Result<AgentEndorsement> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let sentiment = input.sentiment.unwrap_or_else(|| "positive".to_string());

        sqlx::query(
            "INSERT INTO agent_endorsements (id, to_agent_id, from_agent_id, sentiment, reason, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(&input.to_agent_id)
        .bind(&input.from_agent_id)
        .bind(&sentiment)
        .bind(&input.reason)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(AgentEndorsement { id, to_agent_id: input.to_agent_id, from_agent_id: input.from_agent_id, sentiment, reason: input.reason, created_at: now })
    }

    async fn get_reputation(&self, agent_id: &str) -> Result<AgentReputation> {
        let stats = sqlx::query_as::<_, (Option<f64>, i64)>(
            "SELECT AVG(score::float), COUNT(*) FROM agent_reviews WHERE agent_id = $1",
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let reviews: Vec<AgentReview> = sqlx::query_as::<_, (Uuid, String, String, i32, Option<String>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, agent_id, reviewer_id, score, review_text, created_at, updated_at
             FROM agent_reviews WHERE agent_id = $1 ORDER BY updated_at DESC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?
        .into_iter()
        .map(|(id, ag, rev, sc, txt, ca, ua)| AgentReview { id, agent_id: ag, reviewer_id: rev, score: sc as u8, review_text: txt, created_at: ca, updated_at: ua })
        .collect();

        let endorsement_rows: Vec<(Uuid, String, String, String, Option<String>, chrono::DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, to_agent_id, from_agent_id, sentiment, reason, created_at
             FROM agent_endorsements WHERE to_agent_id = $1 ORDER BY created_at DESC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        let positive_endorsements = endorsement_rows.iter().filter(|r| r.3 == "positive").count() as u32;
        let negative_endorsements = endorsement_rows.iter().filter(|r| r.3 == "negative").count() as u32;
        let endorsements: Vec<AgentEndorsement> = endorsement_rows.into_iter()
            .map(|(id, to, from, sentiment, reason, ca)| AgentEndorsement { id, to_agent_id: to, from_agent_id: from, sentiment, reason, created_at: ca })
            .collect();

        Ok(AgentReputation {
            agent_id: agent_id.to_string(),
            avg_score: stats.0,
            review_count: stats.1 as u32,
            positive_endorsements,
            negative_endorsements,
            reviews,
            endorsements,
        })
    }

    // ── Reputation Phase 1 (dual-channel + capabilities) ─────────────────────

    async fn upsert_human_review(&self, input: CreateHumanReviewInput) -> Result<HumanReview> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO human_reviews (id,agent_id,reviewer_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$9)
             ON CONFLICT(agent_id,reviewer_id) DO UPDATE SET
               stars=EXCLUDED.stars,praise=EXCLUDED.praise,criticism=EXCLUDED.criticism,
               domain_context=EXCLUDED.domain_context,updated_at=EXCLUDED.updated_at",
        )
        .bind(id).bind(&input.agent_id).bind(&input.reviewer_id).bind(&input.task_id)
        .bind(input.stars as i32).bind(&input.praise).bind(&input.criticism).bind(&input.domain_context).bind(now)
        .execute(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;

        let r = sqlx::query_as::<_, (Uuid,String,String,Option<String>,i32,Option<String>,Option<String>,Option<String>,chrono::DateTime<Utc>,chrono::DateTime<Utc>)>(
            "SELECT id,agent_id,reviewer_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at
             FROM human_reviews WHERE agent_id=$1 AND reviewer_id=$2",
        ).bind(&input.agent_id).bind(&input.reviewer_id).fetch_one(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        Ok(HumanReview { id: r.0, agent_id: r.1, reviewer_id: r.2, task_id: r.3, stars: r.4 as u8,
            praise: r.5, criticism: r.6, domain_context: r.7, created_at: r.8, updated_at: r.9 })
    }

    async fn upsert_agent_peer_review(&self, input: CreateAgentPeerReviewInput) -> Result<AgentPeerReview> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO agent_peer_reviews (id,to_agent_id,from_agent_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$9)
             ON CONFLICT(to_agent_id,from_agent_id) DO UPDATE SET
               stars=EXCLUDED.stars,praise=EXCLUDED.praise,criticism=EXCLUDED.criticism,
               domain_context=EXCLUDED.domain_context,updated_at=EXCLUDED.updated_at",
        )
        .bind(id).bind(&input.to_agent_id).bind(&input.from_agent_id).bind(&input.task_id)
        .bind(input.stars as i32).bind(&input.praise).bind(&input.criticism).bind(&input.domain_context).bind(now)
        .execute(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;

        let r = sqlx::query_as::<_, (Uuid,String,String,Option<String>,i32,Option<String>,Option<String>,Option<String>,chrono::DateTime<Utc>,chrono::DateTime<Utc>)>(
            "SELECT id,to_agent_id,from_agent_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at
             FROM agent_peer_reviews WHERE to_agent_id=$1 AND from_agent_id=$2",
        ).bind(&input.to_agent_id).bind(&input.from_agent_id).fetch_one(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        Ok(AgentPeerReview { id: r.0, to_agent_id: r.1, from_agent_id: r.2, task_id: r.3, stars: r.4 as u8,
            praise: r.5, criticism: r.6, domain_context: r.7, created_at: r.8, updated_at: r.9 })
    }

    async fn upsert_capability(&self, input: UpsertCapabilityInput) -> Result<AgentCapability> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let source = input.source.unwrap_or_else(|| "manual".to_string());
        let confidence = input.confidence.unwrap_or(1.0);
        sqlx::query(
            "INSERT INTO agent_capabilities (id,agent_id,domain,level,source,confidence,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7)
             ON CONFLICT(agent_id,domain) DO UPDATE SET
               level=EXCLUDED.level,source=EXCLUDED.source,confidence=EXCLUDED.confidence,updated_at=EXCLUDED.updated_at",
        )
        .bind(id).bind(&input.agent_id).bind(&input.domain)
        .bind(input.level as i32).bind(&source).bind(confidence).bind(now)
        .execute(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;

        let r = sqlx::query_as::<_, (Uuid,String,String,i32,String,f64,chrono::DateTime<Utc>)>(
            "SELECT id,agent_id,domain,level,source,confidence,updated_at FROM agent_capabilities WHERE agent_id=$1 AND domain=$2",
        ).bind(&input.agent_id).bind(&input.domain).fetch_one(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        Ok(AgentCapability { id: r.0, agent_id: r.1, domain: r.2, level: r.3 as u8, source: r.4, confidence: r.5, updated_at: r.6 })
    }

    async fn list_capabilities(&self, agent_id: &str) -> Result<Vec<AgentCapability>> {
        let rows = sqlx::query_as::<_, (Uuid,String,String,i32,String,f64,chrono::DateTime<Utc>)>(
            "SELECT id,agent_id,domain,level,source,confidence,updated_at FROM agent_capabilities WHERE agent_id=$1 ORDER BY level DESC",
        ).bind(agent_id).fetch_all(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        Ok(rows.into_iter().map(|(id,ag,dom,lv,src,conf,ua)| AgentCapability {
            id, agent_id: ag, domain: dom, level: lv as u8, source: src, confidence: conf, updated_at: ua
        }).collect())
    }

    async fn get_full_reputation(&self, agent_id: &str) -> Result<AgentReputationFull> {
        let h_stats = sqlx::query_as::<_, (Option<f64>,i64)>(
            "SELECT AVG(stars::float),COUNT(*) FROM human_reviews WHERE agent_id=$1",
        ).bind(agent_id).fetch_one(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;

        let h_rows = sqlx::query_as::<_, (Uuid,String,String,Option<String>,i32,Option<String>,Option<String>,Option<String>,chrono::DateTime<Utc>,chrono::DateTime<Utc>)>(
            "SELECT id,agent_id,reviewer_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at
             FROM human_reviews WHERE agent_id=$1 ORDER BY updated_at DESC LIMIT 20",
        ).bind(agent_id).fetch_all(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        let recent_human_praise: Vec<String> = h_rows.iter().filter_map(|r| r.5.clone()).take(5).collect();
        let recent_human_criticism: Vec<String> = h_rows.iter().filter_map(|r| r.6.clone()).take(5).collect();
        let human_reviews: Vec<HumanReview> = h_rows.into_iter().map(|(id,ag,rev,task,stars,pr,cr,dc,ca,ua)| HumanReview {
            id, agent_id: ag, reviewer_id: rev, task_id: task, stars: stars as u8,
            praise: pr, criticism: cr, domain_context: dc, created_at: ca, updated_at: ua,
        }).collect();

        let a_stats = sqlx::query_as::<_, (Option<f64>,i64)>(
            "SELECT AVG(stars::float),COUNT(*) FROM agent_peer_reviews WHERE to_agent_id=$1",
        ).bind(agent_id).fetch_one(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;

        let a_rows = sqlx::query_as::<_, (Uuid,String,String,Option<String>,i32,Option<String>,Option<String>,Option<String>,chrono::DateTime<Utc>,chrono::DateTime<Utc>)>(
            "SELECT id,to_agent_id,from_agent_id,task_id,stars,praise,criticism,domain_context,created_at,updated_at
             FROM agent_peer_reviews WHERE to_agent_id=$1 ORDER BY updated_at DESC LIMIT 20",
        ).bind(agent_id).fetch_all(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        let recent_agent_praise: Vec<String> = a_rows.iter().filter_map(|r| r.5.clone()).take(5).collect();
        let recent_agent_criticism: Vec<String> = a_rows.iter().filter_map(|r| r.6.clone()).take(5).collect();
        let agent_peer_reviews: Vec<AgentPeerReview> = a_rows.into_iter().map(|(id,to,from,task,stars,pr,cr,dc,ca,ua)| AgentPeerReview {
            id, to_agent_id: to, from_agent_id: from, task_id: task, stars: stars as u8,
            praise: pr, criticism: cr, domain_context: dc, created_at: ca, updated_at: ua,
        }).collect();

        let end_rows = sqlx::query_as::<_, (Uuid,String,String,String,Option<String>,chrono::DateTime<Utc>)>(
            "SELECT id,to_agent_id,from_agent_id,sentiment,reason,created_at FROM agent_endorsements WHERE to_agent_id=$1 ORDER BY created_at DESC LIMIT 20",
        ).bind(agent_id).fetch_all(&self.pool).await.map_err(|e| WorkspaceError::Storage(e.into()))?;
        let endorsements: Vec<AgentEndorsement> = end_rows.into_iter().map(|(id,to,from,sentiment,reason,ca)| AgentEndorsement {
            id, to_agent_id: to, from_agent_id: from, sentiment, reason, created_at: ca
        }).collect();

        let capabilities = self.list_capabilities(agent_id).await?;

        Ok(AgentReputationFull {
            agent_id: agent_id.to_string(),
            human_star_avg: h_stats.0, human_review_count: h_stats.1 as u32,
            recent_human_praise, recent_human_criticism, human_reviews,
            agent_star_avg: a_stats.0, agent_review_count: a_stats.1 as u32,
            recent_agent_praise, recent_agent_criticism, agent_peer_reviews,
            endorsements, capabilities,
        })
    }

    // ─── Phase 2 — Eligibility Gates ──────────────────────────────────────────

    async fn get_eligibility_policy(&self, task_kind: &str) -> aw_domain::error::Result<Option<EligibilityPolicy>> {
        let row = sqlx::query_as::<_, (String, serde_json::Value)>(
            "SELECT task_kind, rules FROM eligibility_policies WHERE task_kind=$1",
        )
        .bind(task_kind)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        if let Some((kind_str, rules_val)) = row {
            let task_kind_enum: TaskKind = serde_json::from_str(&format!("\"{}\"", kind_str)).unwrap_or(TaskKind::Custom(kind_str.clone()));
            let rules: EligibilityRules = serde_json::from_value(rules_val).unwrap_or_default();
            Ok(Some(EligibilityPolicy {
                task_kind: task_kind_enum,
                rules,
            }))
        } else {
            Ok(None)
        }
    }

    async fn upsert_eligibility_policy(&self, policy: EligibilityPolicy) -> aw_domain::error::Result<EligibilityPolicy> {
        let kind_str = serde_json::to_string(&policy.task_kind).unwrap().trim_matches('"').to_string();
        let rules_val = serde_json::to_value(&policy.rules).unwrap();

        sqlx::query(
            "INSERT INTO eligibility_policies (task_kind, rules) VALUES ($1, $2)
             ON CONFLICT(task_kind) DO UPDATE SET rules=EXCLUDED.rules",
        )
        .bind(&kind_str)
        .bind(rules_val)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(e.into()))?;

        Ok(policy)
    }
}
