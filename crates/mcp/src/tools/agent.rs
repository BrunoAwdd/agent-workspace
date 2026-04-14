use std::borrow::Cow;

use rmcp::{
    ErrorData,
    handler::server::router::tool::{AsyncTool, ToolBase},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use aw_domain::entities::{CheckInInput, CheckInResult, CheckOutInput, HeartbeatInput, AgentSession, SessionHealth};

use crate::server::WorkspaceServer;

// ── CheckIn ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct CheckInParams {
    /// ID of the agent checking in (must already be registered).
    pub agent_id: String,
    /// Optional metadata JSON to attach to this session.
    pub metadata: Option<serde_json::Value>,
}

pub struct CheckInTool;
impl ToolBase for CheckInTool {
    type Parameter = CheckInParams;
    type Output = CheckInResult;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.check_in".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Register an active session for this agent. Returns the full context: session, pending inbox items, active tasks and recent handoffs. Also sweeps dead sessions and expired locks. Call once at startup, then use workspace.heartbeat every 30-60s.".into())
    }
}
impl AsyncTool<WorkspaceServer> for CheckInTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        service.storage.check_in(CheckInInput {
            agent_id: param.agent_id,
            metadata: param.metadata,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── Heartbeat ─────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct HeartbeatParams {
    /// Session UUID returned by workspace.check_in.
    pub session_id: String,
    /// Current health status: "healthy" | "degraded" | "unknown".
    pub health: Option<String>,
    /// UUID of the task currently being worked on (optional).
    pub current_task_id: Option<String>,
}

pub struct HeartbeatTool;
impl ToolBase for HeartbeatTool {
    type Parameter = HeartbeatParams;
    type Output = AgentSession;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.heartbeat".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Renew the keepalive for an active session. Call every 30–60 seconds to prevent the session from being marked dead. Also updates the current task being worked on.".into())
    }
}
impl AsyncTool<WorkspaceServer> for HeartbeatTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let session_id = param.session_id.parse()
            .map_err(|_| ErrorData::invalid_params("session_id must be a valid UUID", None))?;
        let health = param.health.as_deref().map(|h| match h {
            "healthy"  => SessionHealth::Healthy,
            "degraded" => SessionHealth::Degraded,
            _          => SessionHealth::Unknown,
        });
        let current_task_id = param.current_task_id
            .map(|s| s.parse().map_err(|_| ErrorData::invalid_params("current_task_id must be a valid UUID", None)))
            .transpose()?;
        service.storage.heartbeat(HeartbeatInput { session_id, health, current_task_id })
            .await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── CheckOut ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct CheckOutParams {
    /// Session UUID to close.
    pub session_id: String,
    /// If true, a Handoff record is created summarising the session for the next agent.
    pub create_handoff: bool,
    /// Brief summary for the handoff note (only used when create_handoff = true).
    pub handoff_summary: Option<String>,
    /// Arbitrary JSON payload to include in the handoff.
    pub handoff_payload: Option<serde_json::Value>,
}

#[derive(Serialize, JsonSchema)]
pub struct OkOutput { pub ok: bool }

pub struct CheckOutTool;
impl ToolBase for CheckOutTool {
    type Parameter = CheckOutParams;
    type Output = OkOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.check_out".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Gracefully close an agent session. Optionally creates a Handoff record so the next agent can pick up context.".into())
    }
}
impl AsyncTool<WorkspaceServer> for CheckOutTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let session_id = param.session_id.parse()
            .map_err(|_| ErrorData::invalid_params("session_id must be a valid UUID", None))?;
        service.storage.check_out(CheckOutInput {
            session_id,
            create_handoff: param.create_handoff,
            handoff_summary: param.handoff_summary,
            handoff_payload: param.handoff_payload,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(OkOutput { ok: true })
    }
}
