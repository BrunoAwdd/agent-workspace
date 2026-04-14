use axum::{extract::{Query, State}, Json};
use aw_domain::entities::{Message, SendMessageInput};
use serde::Deserialize;

use crate::{error::ApiResult, state::AppState};

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    /// Filter by channel. Mutually exclusive with agent_id.
    pub channel_id: Option<String>,
    /// Filter by agent (from or to). Mutually exclusive with channel_id.
    pub agent_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

pub async fn send_message(
    State(state): State<AppState>,
    Json(input): Json<SendMessageInput>,
) -> ApiResult<Json<Message>> {
    let msg = state.storage.send_message(input).await?;
    Ok(Json(msg))
}

pub async fn list_messages(
    State(state): State<AppState>,
    Query(q): Query<ListMessagesQuery>,
) -> ApiResult<Json<Vec<Message>>> {
    let msgs = if let Some(ref agent_id) = q.agent_id {
        state.storage.list_messages_for_agent(agent_id, q.limit).await?
    } else {
        let channel_id = q.channel_id.unwrap_or_default();
        state.storage.list_messages(&channel_id, q.limit).await?
    };
    Ok(Json(msgs))
}
