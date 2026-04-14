use axum::{extract::{Path, State}, Json};
use aw_domain::entities::{CreateHandoffInput, Handoff};

use crate::{error::ApiResult, state::AppState};

pub async fn create_handoff(
    State(state): State<AppState>,
    Json(input): Json<CreateHandoffInput>,
) -> ApiResult<Json<Handoff>> {
    let handoff = state.storage.create_handoff(input).await?;
    Ok(Json(handoff))
}

pub async fn list_handoffs(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<Vec<Handoff>>> {
    let handoffs = state.storage.list_handoffs(&agent_id).await?;
    Ok(Json(handoffs))
}
