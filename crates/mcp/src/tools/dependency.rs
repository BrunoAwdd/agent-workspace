use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::Deserialize;

use aw_domain::entities::{Dependency, DependencyState, UpsertDependencyInput};

use crate::server::WorkspaceServer;

// ── GetDependency ─────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct GetDependencyParams {
    /// Unique key identifying the dependency (e.g. "db:main", "api:billing").
    pub key: String,
}

pub struct GetDependencyTool;
impl ToolBase for GetDependencyTool {
    type Parameter = GetDependencyParams;
    type Output = Dependency;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.get_dependency".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Get the last-known health state of an external dependency. Returns state: \"healthy\" | \"degraded\" | \"unhealthy\" | \"unknown\".".into())
    }
}
impl AsyncTool<WorkspaceServer> for GetDependencyTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        service.storage.get_dependency(&param.key)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params(format!("dependency '{}' not found", param.key), None))
    }
}

// ── UpsertDependency ──────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct UpsertDependencyParams {
    /// Unique key for this dependency.
    pub key: String,
    /// Current state: "healthy" | "degraded" | "unhealthy" | "unknown"
    pub state: String,
    /// Human-readable details about the current state (error message, latency, etc.).
    pub details: Option<String>,
}

pub struct UpsertDependencyTool;
impl ToolBase for UpsertDependencyTool {
    type Parameter = UpsertDependencyParams;
    type Output = Dependency;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.upsert_dependency".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Report the health state of an external dependency. Other agents can read this with workspace.get_dependency to make routing or retry decisions.".into())
    }
}
impl AsyncTool<WorkspaceServer> for UpsertDependencyTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let state = parse_dep_state(&param.state)?;
        service.storage.upsert_dependency(UpsertDependencyInput {
            key: param.key,
            state,
            details: param.details,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_dep_state(s: &str) -> Result<DependencyState, ErrorData> {
    match s {
        "healthy"   => Ok(DependencyState::Healthy),
        "degraded"  => Ok(DependencyState::Degraded),
        "unhealthy" => Ok(DependencyState::Unhealthy),
        "unknown"   => Ok(DependencyState::Unknown),
        other => Err(ErrorData::invalid_params(format!("unknown dependency state: {other}"), None)),
    }
}
