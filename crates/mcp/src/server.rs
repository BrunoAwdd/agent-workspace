use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    transport,
};

use aw_domain::storage::WorkspaceStorage;

use crate::tools::{agent, dependency, handoff, lock, message, summary, task};

pub struct WorkspaceServer {
    pub storage: Arc<dyn WorkspaceStorage>,
}

impl WorkspaceServer {
    pub fn new(storage: Arc<dyn WorkspaceStorage>) -> Self {
        Self { storage }
    }

    /// Called by the `#[tool_handler]` macro as `Self::tool_router()`.
    pub fn tool_router() -> ToolRouter<Self> {
        ToolRouter::new()
            // Agent lifecycle
            .with_async_tool::<agent::CheckInTool>()
            .with_async_tool::<agent::HeartbeatTool>()
            .with_async_tool::<agent::CheckOutTool>()
            // Messaging
            .with_async_tool::<message::SendMessageTool>()
            .with_async_tool::<message::ReadInboxTool>()
            .with_async_tool::<message::AckInboxTool>()
            // Tasks
            .with_async_tool::<task::CreateTaskTool>()
            .with_async_tool::<task::ClaimTaskTool>()
            .with_async_tool::<task::UpdateTaskStatusTool>()
            .with_async_tool::<task::ListTasksTool>()
            .with_async_tool::<task::AssignTaskTool>()
            // Locks
            .with_async_tool::<lock::AcquireLockTool>()
            .with_async_tool::<lock::ReleaseLockTool>()
            // Handoffs
            .with_async_tool::<handoff::CreateHandoffTool>()
            .with_async_tool::<handoff::ListHandoffsTool>()
            // Dependencies
            .with_async_tool::<dependency::GetDependencyTool>()
            .with_async_tool::<dependency::UpsertDependencyTool>()
            // Workspace
            .with_async_tool::<summary::GetSummaryTool>()
    }
}

#[rmcp::tool_handler]
impl ServerHandler for WorkspaceServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.server_info.name = "agent-workspace".into();
        info.server_info.version = env!("CARGO_PKG_VERSION").into();
        info.instructions = Some(
            "Agent workspace coordination tools. \
             Start by calling workspace.check_in to register your session, then use \
             workspace.heartbeat every 30-60s to stay alive. \
             Call workspace.read_inbox on check-in to receive pending messages. \
             Use workspace.check_out with create_handoff=true when ending your session \
             so the next agent can pick up where you left off."
                .into(),
        );
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info
    }
}

pub async fn run(storage: Arc<dyn WorkspaceStorage>) -> Result<()> {
    tracing::info!("agent-workspace MCP server starting (stdio)");
    let server = WorkspaceServer::new(storage);
    let (stdin, stdout) = transport::stdio();
    rmcp::serve_server(server, (stdin, stdout)).await?;
    Ok(())
}
