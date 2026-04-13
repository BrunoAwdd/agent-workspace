use async_trait::async_trait;

use crate::entities::*;
use crate::error::Result;

/// The single storage contract. Both SQLite and Postgres adapters implement this.
#[async_trait]
pub trait WorkspaceStorage: Send + Sync {
    // ── Agents ────────────────────────────────────────────────────────────────

    async fn create_agent(&self, input: CreateAgentInput) -> Result<Agent>;
    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>>;
    async fn list_agents(&self) -> Result<Vec<Agent>>;

    // ── Sessions (check-in / heartbeat / check-out) ───────────────────────────

    async fn check_in(&self, input: CheckInInput) -> Result<AgentSession>;
    async fn heartbeat(&self, input: HeartbeatInput) -> Result<AgentSession>;
    async fn check_out(&self, input: CheckOutInput) -> Result<()>;
    async fn get_session(&self, session_id: uuid::Uuid) -> Result<Option<AgentSession>>;
    async fn active_session(&self, agent_id: &str) -> Result<Option<AgentSession>>;

    // ── Messages ──────────────────────────────────────────────────────────────

    async fn send_message(&self, input: SendMessageInput) -> Result<Message>;
    async fn list_messages(&self, channel_id: &str, limit: u32) -> Result<Vec<Message>>;

    // ── Inbox ─────────────────────────────────────────────────────────────────

    async fn list_inbox(&self, agent_id: &str) -> Result<Vec<InboxItem>>;
    async fn ack_inbox_item(&self, input: AckInboxItemInput) -> Result<()>;

    // ── Tasks ─────────────────────────────────────────────────────────────────

    async fn create_task(&self, input: CreateTaskInput) -> Result<Task>;
    async fn get_task(&self, task_id: uuid::Uuid) -> Result<Option<Task>>;
    async fn claim_task(&self, input: ClaimTaskInput) -> Result<Task>;
    async fn update_task_status(&self, input: UpdateTaskStatusInput) -> Result<Task>;
    async fn list_tasks_for_agent(&self, agent_id: &str) -> Result<Vec<Task>>;

    // ── Locks ─────────────────────────────────────────────────────────────────

    async fn acquire_lock(&self, input: AcquireLockInput) -> Result<Lock>;
    async fn renew_lock(&self, input: RenewLockInput) -> Result<Lock>;
    async fn release_lock(&self, input: ReleaseLockInput) -> Result<()>;
    async fn expire_stale_locks(&self) -> Result<u64>;

    // ── Events ────────────────────────────────────────────────────────────────

    async fn append_event(&self, input: AppendEventInput) -> Result<Event>;
    async fn list_events(&self, agent_id: Option<&str>, limit: u32) -> Result<Vec<Event>>;

    // ── Handoffs ──────────────────────────────────────────────────────────────

    async fn create_handoff(&self, input: CreateHandoffInput) -> Result<Handoff>;
    async fn list_handoffs(&self, agent_id: &str) -> Result<Vec<Handoff>>;

    // ── Dependencies ──────────────────────────────────────────────────────────

    async fn upsert_dependency(&self, input: UpsertDependencyInput) -> Result<Dependency>;
    async fn get_dependency(&self, key: &str) -> Result<Option<Dependency>>;
    async fn list_dependencies(&self) -> Result<Vec<Dependency>>;
}
