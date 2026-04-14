use axum::{extract::State, http::StatusCode, Json};
use aw_domain::entities::{AgentSession, CheckInInput, CheckInResult, CheckOutInput, HeartbeatInput};

use crate::{error::ApiResult, state::AppState};

pub async fn check_in(
    State(state): State<AppState>,
    Json(input): Json<CheckInInput>,
) -> ApiResult<Json<CheckInResult>> {
    let result = state.storage.check_in(input).await?;
    Ok(Json(result))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Json(input): Json<HeartbeatInput>,
) -> ApiResult<Json<AgentSession>> {
    let session = state.storage.heartbeat(input).await?;
    Ok(Json(session))
}

pub async fn check_out(
    State(state): State<AppState>,
    Json(input): Json<CheckOutInput>,
) -> ApiResult<StatusCode> {
    state.storage.check_out(input).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_active(State(state): State<AppState>) -> ApiResult<Json<Vec<AgentSession>>> {
    let sessions = state.storage.list_active_sessions().await?;
    Ok(Json(sessions))
}
