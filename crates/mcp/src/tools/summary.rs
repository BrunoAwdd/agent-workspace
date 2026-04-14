use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::Deserialize;

use aw_domain::entities::WorkspaceSummary;

use crate::server::WorkspaceServer;

#[derive(Deserialize, JsonSchema, Default)]
pub struct GetSummaryParams {}

pub struct GetSummaryTool;
impl ToolBase for GetSummaryTool {
    type Parameter = GetSummaryParams;
    type Output = WorkspaceSummary;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.get_summary".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Get a snapshot of the workspace: active agents, open tasks, total pending inbox items \
             and active lock count. Call this on check-in to orient yourself before picking up work."
                .into(),
        )
    }
}
impl AsyncTool<WorkspaceServer> for GetSummaryTool {
    async fn invoke(service: &WorkspaceServer, _param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        service.storage
            .get_workspace_summary()
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}
