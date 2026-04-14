use axum::{extract::State, Json};
use aw_domain::entities::WorkspaceSummary;

use crate::{error::ApiResult, state::AppState};

pub async fn get_summary(State(state): State<AppState>) -> ApiResult<Json<WorkspaceSummary>> {
    let summary = state.storage.get_workspace_summary().await?;
    Ok(Json(summary))
}
