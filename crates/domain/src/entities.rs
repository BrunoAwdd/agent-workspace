use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Agent ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub status: AgentStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Active,
    Idle,
    Offline,
    Suspended,
}

// ── AgentSession ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentSession {
    pub id: Uuid,
    pub agent_id: String,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub health: SessionHealth,
    pub current_task_id: Option<Uuid>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Idle,
    Dead,
    CheckedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionHealth {
    Healthy,
    Degraded,
    Unknown,
}

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub id: Uuid,
    pub workspace_id: String,
    pub channel_id: Option<String>,
    pub thread_id: Option<Uuid>,
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub kind: MessageKind,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    ChatMessage,
    ReviewRequest,
    ApprovalRequest,
    HandoffNote,
    Alert,
    StatusUpdate,
    DeferredTask,
    ConditionalInstruction,
}

// ── InboxItem ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InboxItem {
    pub id: Uuid,
    pub target_agent_id: String,
    pub source_agent_id: Option<String>,
    pub kind: MessageKind,
    pub status: InboxStatus,
    pub payload: serde_json::Value,
    pub deliver_on_checkin: bool,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InboxStatus {
    Pending,
    Processing,
    Done,
    Failed,
    Expired,
}

// ── Task ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub kind: TaskKind,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assigned_agent_id: Option<String>,
    pub source_ref: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Analysis,
    WriteDocument,
    Review,
    EmailRead,
    HealthCheck,
    Sync,
    Summarization,
    Approval,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    Claimed,
    InProgress,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

// ── Lock ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Lock {
    pub id: Uuid,
    pub scope_type: String,
    pub scope_id: String,
    pub lock_type: LockType,
    pub owner_agent_id: String,
    pub owner_session_id: Uuid,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    WriteLock,
    SoftLock,
    TopicLock,
    ArtifactLock,
    LeaseLock,
}

// ── Event ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Event {
    pub id: Uuid,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<Uuid>,
    pub kind: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── Handoff ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Handoff {
    pub id: Uuid,
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub source_session_id: Uuid,
    pub task_id: Option<Uuid>,
    pub summary: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── Dependency ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    pub key: String,
    pub state: DependencyState,
    pub details: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

// ── Composite results ─────────────────────────────────────────────────────────

/// Returned by check_in. Bundles the new session with everything the agent
/// needs to resume work: pending inbox, active tasks and recent handoffs.
/// Also reports how many stale sessions and locks were swept during the call.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInResult {
    pub session: AgentSession,
    pub inbox: Vec<InboxItem>,
    pub pending_tasks: Vec<Task>,
    pub pending_handoffs: Vec<Handoff>,
    pub dead_sessions_swept: u64,
    pub locks_expired: u64,
}

/// Snapshot of workspace-wide operational state, returned by get_workspace_summary.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceSummary {
    pub active_agents: Vec<Agent>,
    pub open_tasks: Vec<Task>,
    pub pending_inbox_total: u64,
    pub active_locks_count: u64,
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAgentInput {
    pub id: String,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInInput {
    pub agent_id: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HeartbeatInput {
    pub session_id: Uuid,
    pub health: Option<SessionHealth>,
    pub current_task_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckOutInput {
    pub session_id: Uuid,
    pub create_handoff: bool,
    pub handoff_summary: Option<String>,
    pub handoff_payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageInput {
    pub workspace_id: String,
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub channel_id: Option<String>,
    pub thread_id: Option<Uuid>,
    pub kind: MessageKind,
    pub payload: serde_json::Value,
    /// If true, also creates an InboxItem for the target agent.
    pub deliver_to_inbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AckInboxItemInput {
    pub item_id: Uuid,
    pub agent_id: String,
    pub status: InboxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: String,
    pub kind: TaskKind,
    pub priority: TaskPriority,
    pub assigned_agent_id: Option<String>,
    pub source_ref: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClaimTaskInput {
    pub task_id: Uuid,
    pub agent_id: String,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTaskStatusInput {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssignTaskInput {
    pub task_id: Uuid,
    /// Agent doing the assignment (coordinator or anyone with write access).
    pub assigned_by: String,
    /// Agent receiving the task. None = unassign.
    pub assigned_to: Option<String>,
}

/// Filters for listing tasks. All fields are optional — omit to get everything.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListTasksFilter {
    /// Filter by status. Multiple values allowed. Empty = all statuses.
    pub statuses: Option<Vec<TaskStatus>>,
    /// If true, only return tasks with no assigned agent.
    pub unassigned_only: Option<bool>,
    /// Only return tasks assigned to this agent.
    pub assigned_to: Option<String>,
    /// Max results (default 100).
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcquireLockInput {
    pub scope_type: String,
    pub scope_id: String,
    pub lock_type: LockType,
    pub owner_agent_id: String,
    pub owner_session_id: Uuid,
    pub ttl_secs: u64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenewLockInput {
    pub lock_id: Uuid,
    pub owner_session_id: Uuid,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseLockInput {
    pub lock_id: Uuid,
    pub owner_session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppendEventInput {
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<Uuid>,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateHandoffInput {
    pub from_agent_id: String,
    pub to_agent_id: Option<String>,
    pub source_session_id: Uuid,
    pub task_id: Option<Uuid>,
    pub summary: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpsertDependencyInput {
    pub key: String,
    pub state: DependencyState,
    pub details: Option<String>,
}
