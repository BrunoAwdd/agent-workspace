use axum::{extract::{Path, State}, http::StatusCode, Json};
use aw_domain::entities::{AcquireLockInput, Lock, ReleaseLockInput};
use uuid::Uuid;

use crate::{error::ApiResult, state::AppState};

pub async fn acquire_lock(
    State(state): State<AppState>,
    Json(input): Json<AcquireLockInput>,
) -> ApiResult<Json<Lock>> {
    let lock = state.storage.acquire_lock(input).await?;
    Ok(Json(lock))
}

pub async fn release_lock(
    State(state): State<AppState>,
    Path(lock_id): Path<Uuid>,
    Json(mut input): Json<ReleaseLockInput>,
) -> ApiResult<StatusCode> {
    input.lock_id = lock_id;
    state.storage.release_lock(input).await?;
    Ok(StatusCode::NO_CONTENT)
}
