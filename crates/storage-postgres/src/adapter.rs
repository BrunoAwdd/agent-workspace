//! PostgreSQL implementation of WorkspaceStorage.
//! Queries mirror the SQLite adapter but use Postgres syntax and JSONB.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use aw_domain::entities::*;
use aw_domain::error::{Result, WorkspaceError};
use aw_domain::storage::WorkspaceStorage;

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Full implementation mirrors SqliteStorage with Postgres syntax.
// Marked todo!() to keep this compilable while implementation lands incrementally.
#[async_trait]
impl WorkspaceStorage for PostgresStorage {
    async fn create_agent(&self, _input: CreateAgentInput) -> Result<Agent> { todo!("postgres: create_agent") }
    async fn get_agent(&self, _agent_id: &str) -> Result<Option<Agent>> { todo!("postgres: get_agent") }
    async fn list_agents(&self) -> Result<Vec<Agent>> { todo!("postgres: list_agents") }

    async fn check_in(&self, _input: CheckInInput) -> Result<AgentSession> { todo!("postgres: check_in") }
    async fn heartbeat(&self, _input: HeartbeatInput) -> Result<AgentSession> { todo!("postgres: heartbeat") }
    async fn check_out(&self, _input: CheckOutInput) -> Result<()> { todo!("postgres: check_out") }
    async fn get_session(&self, _session_id: Uuid) -> Result<Option<AgentSession>> { todo!("postgres: get_session") }
    async fn active_session(&self, _agent_id: &str) -> Result<Option<AgentSession>> { todo!("postgres: active_session") }

    async fn send_message(&self, _input: SendMessageInput) -> Result<Message> { todo!("postgres: send_message") }
    async fn list_messages(&self, _channel_id: &str, _limit: u32) -> Result<Vec<Message>> { todo!("postgres: list_messages") }

    async fn list_inbox(&self, _agent_id: &str) -> Result<Vec<InboxItem>> { todo!("postgres: list_inbox") }
    async fn ack_inbox_item(&self, _input: AckInboxItemInput) -> Result<()> { todo!("postgres: ack_inbox_item") }

    async fn create_task(&self, _input: CreateTaskInput) -> Result<Task> { todo!("postgres: create_task") }
    async fn get_task(&self, _task_id: Uuid) -> Result<Option<Task>> { todo!("postgres: get_task") }
    async fn claim_task(&self, _input: ClaimTaskInput) -> Result<Task> { todo!("postgres: claim_task") }
    async fn update_task_status(&self, _input: UpdateTaskStatusInput) -> Result<Task> { todo!("postgres: update_task_status") }
    async fn list_tasks_for_agent(&self, _agent_id: &str) -> Result<Vec<Task>> { todo!("postgres: list_tasks_for_agent") }

    async fn acquire_lock(&self, _input: AcquireLockInput) -> Result<Lock> { todo!("postgres: acquire_lock") }
    async fn renew_lock(&self, _input: RenewLockInput) -> Result<Lock> { todo!("postgres: renew_lock") }
    async fn release_lock(&self, _input: ReleaseLockInput) -> Result<()> { todo!("postgres: release_lock") }
    async fn expire_stale_locks(&self) -> Result<u64> { todo!("postgres: expire_stale_locks") }

    async fn append_event(&self, _input: AppendEventInput) -> Result<Event> { todo!("postgres: append_event") }
    async fn list_events(&self, _agent_id: Option<&str>, _limit: u32) -> Result<Vec<Event>> { todo!("postgres: list_events") }

    async fn create_handoff(&self, _input: CreateHandoffInput) -> Result<Handoff> { todo!("postgres: create_handoff") }
    async fn list_handoffs(&self, _agent_id: &str) -> Result<Vec<Handoff>> { todo!("postgres: list_handoffs") }

    async fn upsert_dependency(&self, _input: UpsertDependencyInput) -> Result<Dependency> { todo!("postgres: upsert_dependency") }
    async fn get_dependency(&self, _key: &str) -> Result<Option<Dependency>> { todo!("postgres: get_dependency") }
    async fn list_dependencies(&self) -> Result<Vec<Dependency>> { todo!("postgres: list_dependencies") }
}
