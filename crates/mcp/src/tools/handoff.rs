use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use aw_domain::entities::{CreateHandoffInput, Handoff};

use crate::server::WorkspaceServer;

// ── CreateHandoff ─────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct CreateHandoffParams {
    /// Agent ID handing off (you).
    pub from_agent_id: String,
    /// Agent ID to hand off to (omit for any available agent).
    pub to_agent_id: Option<String>,
    /// Session UUID that is ending.
    pub source_session_id: String,
    /// UUID of the task being handed off (if any).
    pub task_id: Option<String>,
    /// Human-readable summary of what was done and what remains.
    pub summary: String,
    /// Arbitrary JSON context for the receiving agent.
    pub payload: Option<serde_json::Value>,
}

pub struct CreateHandoffTool;
impl ToolBase for CreateHandoffTool {
    type Parameter = CreateHandoffParams;
    type Output = Handoff;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.create_handoff".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Create a Handoff record to pass context to the next agent that picks up this work. Include a summary of what was accomplished and any relevant state in the payload.".into())
    }
}
impl AsyncTool<WorkspaceServer> for CreateHandoffTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let source_session_id = param.source_session_id.parse()
            .map_err(|_| ErrorData::invalid_params("source_session_id must be a valid UUID", None))?;
        let task_id = param.task_id
            .map(|s| s.parse().map_err(|_| ErrorData::invalid_params("task_id must be a valid UUID", None)))
            .transpose()?;
        service.storage.create_handoff(CreateHandoffInput {
            from_agent_id: param.from_agent_id,
            to_agent_id: param.to_agent_id,
            source_session_id,
            task_id,
            summary: param.summary,
            payload: param.payload,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── ListHandoffs ──────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct ListHandoffsParams {
    /// Agent ID to list incoming handoffs for.
    pub agent_id: String,
}

#[derive(Serialize, JsonSchema)]
pub struct HandoffsOutput {
    pub handoffs: Vec<Handoff>,
    pub count: usize,
}

pub struct ListHandoffsTool;
impl ToolBase for ListHandoffsTool {
    type Parameter = ListHandoffsParams;
    type Output = HandoffsOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.list_handoffs".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("List handoff records addressed to this agent. Read these at check-in to understand what the previous agent left for you.".into())
    }
}
impl AsyncTool<WorkspaceServer> for ListHandoffsTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let handoffs = service.storage.list_handoffs(&param.agent_id)
            .await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let count = handoffs.len();
        Ok(HandoffsOutput { handoffs, count })
    }
}
