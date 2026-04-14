use axum::{extract::{Path, State}, Json};
use aw_domain::entities::{Agent, CreateAgentInput};

use crate::{error::ApiResult, state::AppState};

pub async fn list_agents(State(state): State<AppState>) -> ApiResult<Json<Vec<Agent>>> {
    let agents = state.storage.list_agents().await?;
    Ok(Json(agents))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<Agent>> {
    let agent = state.storage.get_agent(&agent_id).await?
        .ok_or_else(|| aw_domain::error::WorkspaceError::NotFound(agent_id))?;
    Ok(Json(agent))
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(input): Json<CreateAgentInput>,
) -> ApiResult<Json<Agent>> {
    let agent = state.storage.create_agent(input).await?;
    Ok(Json(agent))
}
