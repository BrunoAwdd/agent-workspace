use axum::{extract::{Path, State}, Json};
use aw_domain::entities::{Dependency, UpsertDependencyInput};

use crate::{error::ApiResult, state::AppState};

pub async fn upsert_dependency(
    State(state): State<AppState>,
    Json(input): Json<UpsertDependencyInput>,
) -> ApiResult<Json<Dependency>> {
    let dep = state.storage.upsert_dependency(input).await?;
    Ok(Json(dep))
}

pub async fn get_dependency(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> ApiResult<Json<Dependency>> {
    let dep = state.storage.get_dependency(&key).await?
        .ok_or_else(|| aw_domain::error::WorkspaceError::NotFound(format!("dependency '{key}'")))?;
    Ok(Json(dep))
}
