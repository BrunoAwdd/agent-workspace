use std::sync::Arc;
use anyhow::Result;
use aw_domain::storage::WorkspaceStorage;

pub async fn run(_storage: Arc<dyn WorkspaceStorage>) -> Result<()> {
    tracing::info!("agent-workspace MCP server starting (stdio)");
    // MCP server wiring goes here once rmcp integration is complete.
    // Tools: agent.check_in, agent.heartbeat, agent.check_out,
    //        agent.send_message, agent.read_inbox,
    //        task.create, task.claim, task.update_status,
    //        lock.acquire, lock.release,
    //        handoff.create, dependency.get, dependency.upsert
    tracing::info!("MCP tools: pending rmcp integration");
    Ok(())
}
