// Smoke test — requires aw-api running on http://localhost:4000 without auth
import { WorkspaceClient, TaskConflictError } from "../src/index.js";

const client = new WorkspaceClient({
  baseUrl: "http://localhost:4000",
  agentId: "smoke-test-agent-ts",
});

console.log("1. Registering agent...");
const agent = await client.register({
  name: "Smoke Test Agent TS",
  role: "tester",
  capabilities: ["smoke"],
});
console.log(`   OK: ${agent.id}`);

console.log("2. checkIn + enrichment...");
await client.withSession(async (session) => {
  console.log(`   session.id       = ${session.id}`);
  console.log(`   pendingTasks     = ${session.pendingTasks.length}`);
  console.log(`   inbox            = ${session.inbox.length}`);
  console.log(`   pendingHandoffs  = ${session.pendingHandoffs.length}`);

  console.log("3. createTask...");
  const task = await session.createTask(
    "Smoke task TS",
    "Created by smoke test",
    "custom:smoke",
    "low",
  );
  console.log(`   task.id = ${task.id}`);

  console.log("4. claimTask...");
  const claimed = await session.claimTask(task.id!);
  console.log(`   status = ${claimed.status}`);

  console.log("5. conflict detection (claim again)...");
  try {
    await session.claimTask(task.id!);
    console.error("   FAIL: expected TaskConflictError");
    process.exit(1);
  } catch (err) {
    if (err instanceof TaskConflictError) {
      console.log("   OK: TaskConflictError raised correctly");
    } else {
      throw err;
    }
  }

  console.log("6. updateTaskStatus → done...");
  const updated = await session.updateTaskStatus(task.id!, "done");
  console.log(`   status = ${updated.status}`);

  console.log("7. sendMessage...");
  await session.sendMessage({
    toAgentId: "smoke-test-agent-ts",
    kind: "status_update",
    payload: { msg: "smoke complete" },
  });
  console.log("   OK");

  console.log("8. listInbox...");
  const items = await session.listInbox();
  console.log(`   ${items.length} item(s)`);

  console.log("9. workspace summary...");
  const summary = await client.getSummary();
  console.log(`   activeSessions=${summary.activeSessions}`);
});

console.log("\n✅ All TypeScript smoke tests passed");
