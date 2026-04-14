use axum::{extract::{Path, Query, State}, Json};
use aw_domain::entities::{AssignTaskInput, ClaimTaskInput, CreateTaskInput, ListTasksFilter, Task, TaskStatus, UpdateTaskStatusInput};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiResult, state::AppState};

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

pub async fn claim_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(body): Json<ClaimTaskBody>,
) -> ApiResult<Json<Task>> {
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
