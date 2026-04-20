import { WorkspaceClient } from "@agent-workspace/sdk";

async function main() {
  const client = new WorkspaceClient({
    baseUrl: "http://localhost:4000",
    agentId: "l1-bot",
  });

  console.log("[L1 Support] Registering agent...");
  await client.registerAgent({
    name: "L1 Support Agent (TS)",
    role: "support",
    capabilities: ["chat"],
  });

  // We do NOT use the withSession helper here because we want manual control over the check-out
  // to pass handoff parameters cleanly at the end.
  const session = await client.checkIn();
  console.log(`[L1 Support] Checked in. Session ID: ${session.id}`);

  console.log("[L1 Support] Chatting with user...");
  await new Promise((r) => setTimeout(r, 1000));
  console.log(
    "[L1 Support] User says: 'I cannot access my billing page, it returns 500.'",
  );

  // Create a task for the issue
  const task = await session.createTask({
    title: "Billing Page 500 Error",
    description: "User customer_123 cannot access billing page.",
    kind: "bug",
    priority: "high",
  });
  console.log(`[L1 Support] Created incident task: ${task.id}`);

  console.log(
    "[L1 Support] This requires database access. Initiating handoff to L2...",
  );

  // Check out WITH a handoff!
  await session.checkOut({
    createHandoff: true,
    handoffSummary:
      "User is experiencing a 500 error on the billing page. I suspect a missing tenant ID.",
    handoffPayload: {
      customer_id: "customer_123",
      user_tier: "enterprise",
      trace_id: "trace-98765",
      original_task_id: task.id,
    },
    // Note: we let the workspace leave to_agent_id empty so any L2 agent can grab it,
    // or we could configure the workspace to auto-route handoffs.
    // For simplicity, the L2 agent will just query its handoffs.
  });

  console.log(
    "[L1 Support] Checked out and handoff created. Exiting gracefully.",
  );
}

main().catch(console.error);
