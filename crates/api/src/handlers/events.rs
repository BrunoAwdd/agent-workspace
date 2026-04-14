use axum::{extract::{Query, State}, Json};
use aw_domain::entities::Event;
use serde::Deserialize;

use crate::{error::ApiResult, state::AppState};

#[derive(Deserialize, Default)]
pub struct ListEventsQuery {
    /// Filter by agent ID.
    pub agent_id: Option<String>,
    /// Max results (default 100).
    pub limit: Option<u32>,
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(q): Query<ListEventsQuery>,
) -> ApiResult<Json<Vec<Event>>> {
    let events = state.storage
        .list_events(q.agent_id.as_deref(), q.limit.unwrap_or(100))
        .await?;
    Ok(Json(events))
}
