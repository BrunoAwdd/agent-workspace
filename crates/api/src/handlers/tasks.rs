use axum::{extract::{Path, Query, State}, Json};
use aw_domain::entities::{AssignTaskInput, ClaimTaskInput, CreateTaskInput, ListTasksFilter, Task, TaskKind, TaskStatus, UpdateTaskStatusInput};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::{ApiError, ApiResult}, state::AppState};
use aw_domain::error::WorkspaceError;

pub async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<CreateTaskInput>,
) -> ApiResult<Json<Task>> {
    let task = state.storage.create_task(input).await?;
    Ok(Json(task))
}

// Body DTOs — task_id comes from the URL path, not the body.
#[derive(Deserialize)]
pub struct ClaimTaskBody {
    agent_id: String,
    session_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdateTaskStatusBody {
    status: TaskStatus,
    metadata: Option<serde_json::Value>,
}

async fn check_task_eligibility(
    state: &AppState,
    agent_id: &str,
    task_kind: &TaskKind,
    action: &str,
) -> ApiResult<()> {
    let kind_str = match task_kind {
        TaskKind::Analysis => "analysis",
        TaskKind::WriteDocument => "write_document",
        TaskKind::Review => "review",
        TaskKind::EmailRead => "email_read",
        TaskKind::HealthCheck => "health_check",
        TaskKind::Sync => "sync",
        TaskKind::Summarization => "summarization",
        TaskKind::Approval => "approval",
        TaskKind::Custom(c) => c,
    };
    
    if let Some(policy) = state.storage.get_eligibility_policy(kind_str).await? {
        let rule = match action {
            "claim" => &policy.rules.claim,
            "review" => &policy.rules.review,
            "approve" => &policy.rules.approve,
            _ => &None,
        };
        
        if let Some(r) = rule {
            if r.requires.is_empty() { return Ok(()); }
            let caps = state.storage.list_capabilities(agent_id).await?;
            let mut missing = Vec::new();
            
            for req in &r.requires {
                let current_level = caps.iter().find(|c| c.domain == req.domain).map(|c| c.level).unwrap_or(0);
                if current_level < req.min {
                    missing.push(format!("{} >= {}", req.domain, req.min));
                }
            }
            
            if !missing.is_empty() {
                return Err(WorkspaceError::Forbidden(format!("Eligibility failed: missing {}", missing.join(", "))).into());
            }
        }
    }
    Ok(())
}

pub async fn claim_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(body): Json<ClaimTaskBody>,
) -> ApiResult<Json<Task>> {
    let task = state.storage.get_task(task_id).await?.ok_or_else(|| WorkspaceError::NotFound("Task not found".into()))?;
    check_task_eligibility(&state, &body.agent_id, &task.kind, "claim").await?;

    let task = state.storage.claim_task(ClaimTaskInput {
        task_id,
        agent_id: body.agent_id,
        session_id: body.session_id,
    }).await?;
    Ok(Json(task))
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(body): Json<UpdateTaskStatusBody>,
) -> ApiResult<Json<Task>> {
    let task = state.storage.update_task_status(UpdateTaskStatusInput {
        task_id,
        status: body.status,
        metadata: body.metadata,
    }).await?;
    Ok(Json(task))
}

/// Query params for GET /tasks
#[derive(Deserialize, Default)]
pub struct ListTasksQuery {
    /// Comma-separated statuses: open,claimed,in_progress,done,failed,cancelled
    pub status: Option<String>,
    /// Only unassigned tasks
    pub unassigned: Option<bool>,
    /// Tasks assigned to this agent
    pub assigned_to: Option<String>,
    pub limit: Option<u32>,
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(q): Query<ListTasksQuery>,
) -> ApiResult<Json<Vec<Task>>> {
    let statuses = q.status.as_deref().map(|s| {
        s.split(',').filter_map(|p| match p.trim() {
            "open"        => Some(TaskStatus::Open),
            "claimed"     => Some(TaskStatus::Claimed),
            "in_progress" => Some(TaskStatus::InProgress),
            "done"        => Some(TaskStatus::Done),
            "failed"      => Some(TaskStatus::Failed),
            "cancelled"   => Some(TaskStatus::Cancelled),
            _             => None,
        }).collect::<Vec<_>>()
    });

    let tasks = state.storage.list_tasks(ListTasksFilter {
        statuses,
        unassigned_only: q.unassigned,
        assigned_to: q.assigned_to,
        limit: q.limit,
    }).await?;
    Ok(Json(tasks))
}

pub async fn assign_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(mut input): Json<AssignTaskInput>,
) -> ApiResult<Json<Task>> {
    input.task_id = task_id;
    let task = state.storage.assign_task(input).await?;
    Ok(Json(task))
}
