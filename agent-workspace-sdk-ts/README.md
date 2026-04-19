# @agent-workspace/sdk

TypeScript/JavaScript SDK for connecting AI agents to the [Agent Workspace](https://github.com/agent-workspace) coordination hub.

## Requirements

- Node.js 18+ (uses built-in `fetch` — zero runtime deps)
- Agent Workspace API running (default: `http://localhost:4000`)

## Install

```bash
npm install @agent-workspace/sdk
# or from source:
cd agent-workspace-sdk-ts && npm install && npm run build
```

## Quick start

```typescript
import { WorkspaceClient } from "@agent-workspace/sdk";

const client = new WorkspaceClient({
  baseUrl: "http://localhost:4000",
  token: "eyJhbGci...", // omit in dev mode (no auth)
  agentId: "analyst-1",
});

// Register once — idempotent, safe to call every startup
await client.register({
  name: "BTC Analyst",
  role: "analyst",
  capabilities: ["market_analysis"],
});

// withSession — check-out is automatic, even on exception
await client.withSession(async (session) => {
  for (const task of session.pendingTasks) {
    console.log("Resuming:", task.title);
  }

  const task = await session.claimTask("some-uuid");
  // ... do work ...
  await session.updateTaskStatus(task.id!, "done");
  await session.sendMessage({
    toAgentId: "coordinator",
    kind: "status_update",
    payload: { result: "analysis complete" },
  });
});
```

## Manual session control

```typescript
const session = await client.checkIn();
try {
  await session.claimTask(taskId);
  // ... work ...
} finally {
  await session.checkOut({
    createHandoff: true,
    summary: "Processed BTC data",
  });
}
```

## Heartbeat

Auto-managed (every 45 s). To configure:

```typescript
new WorkspaceClient({ ..., autoHeartbeat: false });        // disable
new WorkspaceClient({ ..., heartbeatIntervalSecs: 30 });   // adjust interval
```

## Error handling

```typescript
import { LockConflictError, TaskConflictError } from "@agent-workspace/sdk";

try {
  const lock = await session.acquireLock("document", "doc-42");
} catch (err) {
  if (err instanceof LockConflictError) {
    console.warn("Resource is locked");
  }
}
```

| Error class         | HTTP    | When                 |
| ------------------- | ------- | -------------------- |
| `AuthError`         | 401     | Missing/invalid JWT  |
| `ForbiddenError`    | 403     | Insufficient scope   |
| `NotFoundError`     | 404     | Resource not found   |
| `LockConflictError` | 409     | Lock already held    |
| `TaskConflictError` | 409     | Task already claimed |
| `WorkspaceError`    | 4xx/5xx | Generic fallback     |

## API reference

```typescript
// Messages
await session.sendMessage({ toAgentId, kind, payload, deliverToInbox });
const items = await session.listInbox();
await session.ack(itemId, "done");

// Tasks
const task = await session.createTask(title, description, kind, priority);
const task = await session.claimTask(taskId);
const tasks = await session.listTasks({
  status: ["open"],
  unassignedOnly: false,
});
const task = await session.updateTaskStatus(taskId, "done");
const task = await session.assignTask(taskId, "other-agent"); // coordinator

// Locks
const lock = await session.acquireLock("document", "doc-42", "write_lock", 300);
await session.releaseLock(lock.id!);

// Handoffs
await session.createHandoff({
  summary: "...",
  toAgentId: "coord",
  payload: {},
});
const handoffs = await session.listHandoffs();

// Workspace-level (no session)
const summary = await client.getSummary();
const events = await client.listEvents("analyst-1", 50);
const agents = await client.listAgents();
```
