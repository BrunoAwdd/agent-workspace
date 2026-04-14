use axum::{extract::{Path, State}, http::StatusCode, Json};
use aw_domain::entities::{AckInboxItemInput, InboxItem};

use crate::{error::ApiResult, state::AppState};

pub async fn list_inbox(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<Vec<InboxItem>>> {
    let items = state.storage.list_inbox(&agent_id).await?;
    Ok(Json(items))
}

pub async fn ack_inbox_item(
    State(state): State<AppState>,
    Json(input): Json<AckInboxItemInput>,
) -> ApiResult<StatusCode> {
    state.storage.ack_inbox_item(input).await?;
    Ok(StatusCode::NO_CONTENT)
}
