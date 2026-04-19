# Agent Workspace SDK — Specification

## Context

The Agent Workspace is a coordination hub for AI agents (REST API, running on port 4000 by default).
This document specifies the SDK to be built in **Python** and **TypeScript/JavaScript**.

The SDK must hide all HTTP complexity behind a clean interface. An agent should be able to integrate
in under 10 lines of code. The API reference is in `AGENT_GUIDE.md`.

---

## Target languages

| Language | Package name | Min version |
|---|---|---|
| Python | `agent-workspace-sdk` | Python 3.10+ |
| TypeScript / JavaScript | `@agent-workspace/sdk` | Node 18+ / ES2022 |

Both packages must have identical method names and behavior.

---

## Core design principles

1. **Lifecycle managed by context manager / class** — the developer shouldn't think about
   heartbeat timers or cleanup. The SDK handles it automatically.
2. **Typed inputs and outputs** — all request/response objects are typed (Pydantic for Python,
   TypeScript interfaces for JS).
3. **JWT passed once** — token is configured at client instantiation, attached to every request.
4. **Heartbeat is automatic** — once checked in, the SDK sends a heartbeat every 45 seconds
   in a background thread/interval. Developer can disable this.
5. **Errors are explicit** — typed exceptions, not generic HTTP errors.

---

## Client instantiation

### Python
```python
from agent_workspace import WorkspaceClient

client = WorkspaceClient(
    base_url="http://localhost:4000",
    token="eyJhbGci...",          # JWT Bearer token (optional — dev mode works without)
    agent_id="my-agent-1",
)
```

### TypeScript
```typescript
import { WorkspaceClient } from "@agent-workspace/sdk";

const client = new WorkspaceClient({
  baseUrl: "http://localhost:4000",
  token: "eyJhbGci...",           // JWT Bearer token (optional)
  agentId: "my-agent-1",
});
```

---

## Session lifecycle

This is the most important part. The SDK must manage the full lifecycle automatically.

### Python — context manager (preferred)
```python
async with client.session() as session:
    # heartbeat runs automatically every 45s in background
    tasks = session.pending_tasks       # from check-in response
    inbox = session.inbox               # from check-in response
    handoffs = session.pending_handoffs # from check-in response

    await session.send_message(...)
    await session.claim_task(task_id)

# check_out is called automatically on exit (even on exception)
```

### Python — manual
```python
session = await client.check_in()
try:
    await session.heartbeat()
    # ... work ...
finally:
    await session.check_out(
        create_handoff=True,
        summary="Completed analysis",
        payload={"last_price": 95000}
    )
```

### TypeScript — using statement (preferred)
```typescript
await client.withSession(async (session) => {
  // heartbeat runs automatically
  const { pendingTasks, inbox } = session;

  await session.sendMessage({ ... });
  await session.claimTask(taskId);

  // check_out called automatically
});
```

### TypeScript — manual
```typescript
const session = await client.checkIn();
try {
  await session.heartbeat();
  // ... work ...
} finally {
  await session.checkOut({ createHandoff: true, summary: "Done" });
}
```

---

## Session object — available methods

After `check_in`, all operations are available on the `session` object.
The session object must carry the `session_id` automatically in all calls.

### Properties (populated from check-in response)

```
session.id               → UUID of the session
session.agentId          → agent_id
session.inbox            → List[InboxItem]  — pending inbox at check-in time
session.pendingTasks     → List[Task]       — tasks assigned to this agent
session.pendingHandoffs  → List[Handoff]    — handoffs directed to this agent
```

### Heartbeat
```python
await session.heartbeat(health="healthy", current_task_id=None)
```

### Messages
```python
await session.send_message(
    to_agent_id="other-agent",
    kind="alert",
    payload={"text": "BTC breakout"},
    deliver_to_inbox=True,
)
```

### Inbox
```python
items = await session.list_inbox()
await session.ack(item_id, status="done")         # success
await session.ack(item_id, status="failed")        # triggers retry
```

### Tasks
```python
task  = await session.create_task(title, description, kind, priority)
task  = await session.claim_task(task_id)
tasks = await session.list_tasks(status=["open"], unassigned_only=False)
task  = await session.update_task_status(task_id, status="done")
task  = await session.assign_task(task_id, assigned_to="other-agent")  # coordinator
```

### Locks
```python
lock = await session.acquire_lock(scope_type, scope_id, lock_type="write_lock", ttl_secs=300)
await session.release_lock(lock_id)
```

### Handoffs
```python
await session.create_handoff(
    to_agent_id="coordinator",
    summary="Completed BTC scan",
    payload={"last_price": 95000},
)
handoffs = await session.list_handoffs()
```

### Workspace (read-only, no session required)
```python
summary = await client.get_summary()
events  = await client.list_events(agent_id="analyst-1", limit=50)
agents  = await client.list_agents()
```

---

## Agent registration

```python
agent = await client.register(
    id="my-agent-1",
    name="Market Analyst",
    role="analyst",
    capabilities=["btc", "market_analysis"],
    permissions=[],
)
```

Registration is **idempotent** — safe to call on every startup. If the agent already
exists, it updates the name/role/capabilities.

---

## Automatic heartbeat

After `check_in`, the SDK must automatically send heartbeats every 45 seconds using:
- Python: `asyncio.create_task` with a background coroutine
- TypeScript: `setInterval`

The heartbeat loop must:
- Stop automatically on `check_out`
- Stop automatically on session exception
- Be configurable: `heartbeat_interval_secs=45` (client or session level)
- Be disableable: `auto_heartbeat=False`

If a heartbeat fails (network error), the SDK logs a warning but does NOT raise.
The session remains usable.

---

## Error types

Define typed exceptions/errors (do not use generic HTTP errors):

| Exception | When |
|---|---|
| `AuthError` | 401 — missing or invalid JWT |
| `ForbiddenError` | 403 — JWT valid but missing required scope |
| `NotFoundError` | 404 — agent, task, lock not found |
| `LockConflictError` | 409 — lock already held |
| `TaskConflictError` | 409 — task already claimed |
| `WorkspaceError` | other 4xx/5xx — generic fallback |

---

## HTTP conventions

All requests:
- `Content-Type: application/json`
- `Authorization: Bearer <token>` (if token configured)

Responses are JSON. On error, the body may contain `{ "error": "..." }`.

Base URL is configurable. Default: `http://localhost:4000`.

---

## Minimal working example (Python)

```python
import asyncio
from agent_workspace import WorkspaceClient

async def main():
    client = WorkspaceClient(
        base_url="http://localhost:4000",
        token="eyJhbGci...",
        agent_id="analyst-1",
    )

    # Register once (idempotent)
    await client.register(
        id="analyst-1",
        name="BTC Analyst",
        role="analyst",
        capabilities=["market_analysis"],
        permissions=[],
    )

    # Run a session
    async with client.session() as session:
        for task in session.pending_tasks:
            print(f"Resuming task: {task.title}")

        task = await session.claim_task(task_id="some-uuid")
        # ... do work ...
        await session.update_task_status(task.id, status="done")
        await session.send_message(
            to_agent_id="coordinator",
            kind="status_update",
            payload={"result": "analysis complete"},
        )

asyncio.run(main())
```

---

## Minimal working example (TypeScript)

```typescript
import { WorkspaceClient } from "@agent-workspace/sdk";

const client = new WorkspaceClient({
  baseUrl: "http://localhost:4000",
  token: "eyJhbGci...",
  agentId: "analyst-1",
});

await client.register({
  id: "analyst-1",
  name: "BTC Analyst",
  role: "analyst",
  capabilities: ["market_analysis"],
  permissions: [],
});

await client.withSession(async (session) => {
  for (const task of session.pendingTasks) {
    console.log("Resuming:", task.title);
  }

  const task = await session.claimTask("some-uuid");
  // ... do work ...
  await session.updateTaskStatus(task.id, "done");
  await session.sendMessage({
    toAgentId: "coordinator",
    kind: "status_update",
    payload: { result: "analysis complete" },
  });
});
```

---

## Deliverables

1. **Python package** (`agent-workspace-sdk/`)
   - `pyproject.toml` with `httpx` as HTTP client (async-native)
   - `agent_workspace/__init__.py` — exports `WorkspaceClient`
   - `agent_workspace/client.py` — `WorkspaceClient`
   - `agent_workspace/session.py` — `AgentSession` + heartbeat loop
   - `agent_workspace/models.py` — Pydantic models for all entities
   - `agent_workspace/exceptions.py` — typed exceptions
   - `README.md` with minimal example

2. **TypeScript package** (`agent-workspace-sdk-ts/`)
   - `package.json` with `fetch` (built-in Node 18+) — no extra HTTP deps
   - `src/index.ts` — exports `WorkspaceClient`
   - `src/client.ts` — `WorkspaceClient`
   - `src/session.ts` — `AgentSession` + heartbeat interval
   - `src/types.ts` — TypeScript interfaces for all entities
   - `src/errors.ts` — typed error classes
   - `README.md` with minimal example

---

## API endpoints reference (for the implementer)

```
POST   /agents                      register/update agent
GET    /agents                      list agents
GET    /agents/:id                  get agent

POST   /sessions/check-in           { agent_id }
POST   /sessions/heartbeat          { session_id, health?, current_task_id? }
POST   /sessions/check-out          { session_id, create_handoff, handoff_summary?, handoff_payload? }
GET    /sessions/active             list active sessions

POST   /messages                    send message
GET    /messages?agent_id=&limit=   list messages

GET    /inbox/:agent_id             list pending inbox
POST   /inbox/:item_id/ack          { item_id, agent_id, status }

POST   /tasks                       create task
GET    /tasks?status=&unassigned=&assigned_to=&limit=
POST   /tasks/:id/claim             { agent_id, session_id }
POST   /tasks/:id/status            { status, metadata? }
POST   /tasks/:id/assign            { assigned_by, assigned_to? }

POST   /locks                       acquire lock
DELETE /locks/:id                   { lock_id, owner_session_id }

POST   /handoffs                    create handoff
GET    /handoffs/:agent_id          list handoffs

POST   /dependencies                upsert dependency
GET    /dependencies/:key           get dependency

GET    /events?agent_id=&limit=     list events
GET    /summary                     workspace snapshot
GET    /health                      "ok" (no auth required)
```
