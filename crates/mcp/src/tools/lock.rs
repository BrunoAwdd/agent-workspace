use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::Deserialize;

use aw_domain::entities::{AcquireLockInput, Lock, LockType, ReleaseLockInput};

use crate::{server::WorkspaceServer, tools::agent::OkOutput};

// ── AcquireLock ───────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct AcquireLockParams {
    /// Resource category being locked (e.g. "document", "tool", "channel").
    pub scope_type: String,
    /// Specific resource ID within the scope (e.g. the document UUID).
    pub scope_id: String,
    /// Lock strength: "write_lock" | "soft_lock" | "topic_lock" | "artifact_lock" | "lease_lock"
    pub lock_type: String,
    /// Agent ID acquiring the lock.
    pub owner_agent_id: String,
    /// Session UUID of the acquiring agent.
    pub owner_session_id: String,
    /// Lock TTL in seconds. Lock is auto-expired after this time if not released.
    pub ttl_secs: u64,
    pub metadata: Option<serde_json::Value>,
}

pub struct AcquireLockTool;
impl ToolBase for AcquireLockTool {
    type Parameter = AcquireLockParams;
    type Output = Lock;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.acquire_lock".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Acquire a distributed lock on a resource. Returns the Lock object on success, or a CONFLICT error if another agent holds the lock. Always release with workspace.release_lock when done.".into())
    }
}
impl AsyncTool<WorkspaceServer> for AcquireLockTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let owner_session_id = param.owner_session_id.parse()
            .map_err(|_| ErrorData::invalid_params("owner_session_id must be a valid UUID", None))?;
        let lock_type = parse_lock_type(&param.lock_type)?;
        service.storage.acquire_lock(AcquireLockInput {
            scope_type: param.scope_type,
            scope_id: param.scope_id,
            lock_type,
            owner_agent_id: param.owner_agent_id,
            owner_session_id,
            ttl_secs: param.ttl_secs,
            metadata: param.metadata,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── ReleaseLock ───────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct ReleaseLockParams {
    /// UUID of the lock to release.
    pub lock_id: String,
    /// Session UUID — must match the owner_session_id of the lock.
    pub owner_session_id: String,
}

pub struct ReleaseLockTool;
impl ToolBase for ReleaseLockTool {
    type Parameter = ReleaseLockParams;
    type Output = OkOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.release_lock".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Release a lock previously acquired with workspace.acquire_lock. Must be called by the same session that acquired it.".into())
    }
}
impl AsyncTool<WorkspaceServer> for ReleaseLockTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let lock_id = param.lock_id.parse()
            .map_err(|_| ErrorData::invalid_params("lock_id must be a valid UUID", None))?;
        let owner_session_id = param.owner_session_id.parse()
            .map_err(|_| ErrorData::invalid_params("owner_session_id must be a valid UUID", None))?;
        service.storage.release_lock(ReleaseLockInput { lock_id, owner_session_id })
            .await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(OkOutput { ok: true })
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_lock_type(s: &str) -> Result<LockType, ErrorData> {
    match s {
        "write_lock"    => Ok(LockType::WriteLock),
        "soft_lock"     => Ok(LockType::SoftLock),
        "topic_lock"    => Ok(LockType::TopicLock),
        "artifact_lock" => Ok(LockType::ArtifactLock),
        "lease_lock"    => Ok(LockType::LeaseLock),
        other => Err(ErrorData::invalid_params(format!("unknown lock type: {other}"), None)),
    }
}
