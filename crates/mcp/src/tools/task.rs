use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::Deserialize;

use aw_domain::entities::{
    AssignTaskInput, ClaimTaskInput, CreateTaskInput, ListTasksFilter, Task, TaskKind, TaskPriority,
    TaskStatus, UpdateTaskStatusInput,
};

use crate::server::WorkspaceServer;

// ── CreateTask ────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct CreateTaskParams {
    pub title: String,
    pub description: String,
    /// "analysis" | "write_document" | "review" | "email_read" |
    /// "health_check" | "sync" | "summarization" | "approval" | "custom:<name>"
    pub kind: String,
    /// "low" | "normal" | "high" | "critical" (default: "normal")
    pub priority: Option<String>,
    /// Pre-assign to an agent ID (optional — can also be claimed later).
    pub assigned_agent_id: Option<String>,
    /// External reference (ticket ID, URL, etc.).
    pub source_ref: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub struct CreateTaskTool;
impl ToolBase for CreateTaskTool {
    type Parameter = CreateTaskParams;
    type Output = Task;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.create_task".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Create a new task in the workspace. Returns the task object with its assigned UUID. Other agents can then claim it with workspace.claim_task.".into())
    }
}
impl AsyncTool<WorkspaceServer> for CreateTaskTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let kind = parse_task_kind(&param.kind)?;
        let priority = param.priority.as_deref().map(parse_task_priority).transpose()?
            .unwrap_or(TaskPriority::Normal);
        service.storage.create_task(CreateTaskInput {
            title: param.title,
            description: param.description,
            kind,
            priority,
            assigned_agent_id: param.assigned_agent_id,
            source_ref: param.source_ref,
            metadata: param.metadata,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── ClaimTask ─────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct ClaimTaskParams {
    /// UUID of the task to claim.
    pub task_id: String,
    /// Agent ID claiming the task.
    pub agent_id: String,
    /// Session UUID of the claiming agent.
    pub session_id: String,
}

pub struct ClaimTaskTool;
impl ToolBase for ClaimTaskTool {
    type Parameter = ClaimTaskParams;
    type Output = Task;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.claim_task".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Claim an open task, assigning it to your agent and session. Returns the updated task. Only one agent can hold a task at a time.".into())
    }
}
impl AsyncTool<WorkspaceServer> for ClaimTaskTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let task_id = param.task_id.parse()
            .map_err(|_| ErrorData::invalid_params("task_id must be a valid UUID", None))?;
        let session_id = param.session_id.parse()
            .map_err(|_| ErrorData::invalid_params("session_id must be a valid UUID", None))?;
        service.storage.claim_task(ClaimTaskInput {
            task_id,
            agent_id: param.agent_id,
            session_id,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── UpdateTaskStatus ──────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct UpdateTaskStatusParams {
    /// UUID of the task to update.
    pub task_id: String,
    /// New status: "open" | "claimed" | "in_progress" | "done" | "failed" | "cancelled"
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct UpdateTaskStatusTool;
impl ToolBase for UpdateTaskStatusTool {
    type Parameter = UpdateTaskStatusParams;
    type Output = Task;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.update_task_status".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Update the status of a task. Use \"in_progress\" when starting work, \"done\" when complete, \"failed\" on unrecoverable error.".into())
    }
}
impl AsyncTool<WorkspaceServer> for UpdateTaskStatusTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let task_id = param.task_id.parse()
            .map_err(|_| ErrorData::invalid_params("task_id must be a valid UUID", None))?;
        let status = parse_task_status(&param.status)?;
        service.storage.update_task_status(UpdateTaskStatusInput {
            task_id,
            status,
            metadata: param.metadata,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── ListTasks ─────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct ListTasksParams {
    /// Filter by statuses. Comma-separated: "open,claimed,in_progress". Empty = all.
    pub statuses: Option<String>,
    /// If true, only return tasks with no assigned agent. Default false.
    pub unassigned_only: Option<bool>,
    /// Only return tasks assigned to this agent ID.
    pub assigned_to: Option<String>,
    /// Max results (default 100).
    pub limit: Option<u32>,
}

#[derive(serde::Serialize, JsonSchema)]
pub struct TaskListOutput {
    pub tasks: Vec<Task>,
    pub count: usize,
}

pub struct ListTasksTool;
impl ToolBase for ListTasksTool {
    type Parameter = ListTasksParams;
    type Output = TaskListOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.list_tasks".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some(
            "List tasks with optional filters. A coordinator agent uses this to see the full queue \
             of work. Set unassigned_only=true to find tasks that need an owner. \
             Filter by statuses e.g. \"open,claimed\" to focus on active work."
                .into(),
        )
    }
}
impl AsyncTool<WorkspaceServer> for ListTasksTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let statuses = param.statuses.as_deref().map(|s| {
            s.split(',').filter_map(|p| parse_task_status(p.trim()).ok()).collect::<Vec<_>>()
        });
        let tasks = service.storage.list_tasks(ListTasksFilter {
            statuses,
            unassigned_only: param.unassigned_only,
            assigned_to: param.assigned_to,
            limit: param.limit,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let count = tasks.len();
        Ok(TaskListOutput { tasks, count })
    }
}

// ── AssignTask ────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct AssignTaskParams {
    /// UUID of the task to assign.
    pub task_id: String,
    /// Agent doing the assignment (coordinator).
    pub assigned_by: String,
    /// Agent ID to assign the task to. Omit or null to unassign.
    pub assigned_to: Option<String>,
}

pub struct AssignTaskTool;
impl ToolBase for AssignTaskTool {
    type Parameter = AssignTaskParams;
    type Output = Task;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.assign_task".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Assign a task to a specific agent, or unassign it (set assigned_to=null). \
             A coordinator uses this to delegate work. After assigning, send an inbox message \
             to notify the agent they have a new task."
                .into(),
        )
    }
}
impl AsyncTool<WorkspaceServer> for AssignTaskTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let task_id = param.task_id.parse()
            .map_err(|_| ErrorData::invalid_params("task_id must be a valid UUID", None))?;
        service.storage.assign_task(AssignTaskInput {
            task_id,
            assigned_by: param.assigned_by,
            assigned_to: param.assigned_to,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_task_kind(s: &str) -> Result<TaskKind, ErrorData> {
    let kind = match s {
        "analysis"       => TaskKind::Analysis,
        "write_document" => TaskKind::WriteDocument,
        "review"         => TaskKind::Review,
        "email_read"     => TaskKind::EmailRead,
        "health_check"   => TaskKind::HealthCheck,
        "sync"           => TaskKind::Sync,
        "summarization"  => TaskKind::Summarization,
        "approval"       => TaskKind::Approval,
        other if other.starts_with("custom:") => TaskKind::Custom(other["custom:".len()..].to_string()),
        other => return Err(ErrorData::invalid_params(format!("unknown task kind: {other}"), None)),
    };
    Ok(kind)
}

fn parse_task_priority(s: &str) -> Result<TaskPriority, ErrorData> {
    match s {
        "low"      => Ok(TaskPriority::Low),
        "normal"   => Ok(TaskPriority::Normal),
        "high"     => Ok(TaskPriority::High),
        "critical" => Ok(TaskPriority::Critical),
        other => Err(ErrorData::invalid_params(format!("unknown priority: {other}"), None)),
    }
}

fn parse_task_status(s: &str) -> Result<TaskStatus, ErrorData> {
    match s {
        "open"        => Ok(TaskStatus::Open),
        "claimed"     => Ok(TaskStatus::Claimed),
        "in_progress" => Ok(TaskStatus::InProgress),
        "done"        => Ok(TaskStatus::Done),
        "failed"      => Ok(TaskStatus::Failed),
        "cancelled"   => Ok(TaskStatus::Cancelled),
        other => Err(ErrorData::invalid_params(format!("unknown task status: {other}"), None)),
    }
}
