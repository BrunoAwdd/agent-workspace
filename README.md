# Agent Workspace

> Coordination hub for AI agents — sessions, tasks, messages, locks, handoffs.

A lightweight server that lets multiple AI agents collaborate without stepping on each other. Agents check in, claim work, exchange messages, acquire locks, and hand off context — all through a clean REST API or MCP.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     agent-workspace                     │
│                                                         │
│  ┌─────────┐   ┌──────────────┐   ┌─────────────────┐  │
│  │ aw-mcp  │   │   aw-api     │   │  aw-domain      │  │
│  │  stdio  │   │  HTTP :4000  │   │  entities +     │  │
│  │  (MCP)  │   │  (REST)      │   │  storage trait  │  │
│  └────┬────┘   └──────┬───────┘   └────────┬────────┘  │
│       └───────────────┴───────────────┬─────┘           │
│                                       │                  │
│                          ┌────────────┴────────────┐    │
│                          │  aw-storage-sqlite (dev) │    │
│                          │  aw-storage-postgres(prod)│   │
│                          └─────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
         ↑ MCP (stdio)          ↑ HTTP REST
   ┌─────┴─────┐          ┌─────┴──────┐
   │ AI agent  │          │ AI agent   │
   │ (Claude)  │          │(Python/TS) │
   └───────────┘          └────────────┘
```

### Primitives

| Primitive           | What it does                                                      |
| ------------------- | ----------------------------------------------------------------- |
| **Session**         | Tracks an agent's active run (`check-in → heartbeat → check-out`) |
| **Task**            | Work item claimed by one agent at a time (atomic)                 |
| **Message / Inbox** | Async agent-to-agent communication                                |
| **Lock**            | Mutual exclusion over a shared resource with TTL                  |
| **Handoff**         | Context passed from one session to the next                       |
| **Dependency**      | Health status of external resources                               |
| **Event**           | Immutable audit trail of all workspace activity                   |

---

## Quickstart

### 1. Build

```bash
cargo build -p aw-api -p aw-mcp
```

### 2. Configure `.env`

```env
STORAGE_BACKEND=sqlite
SQLITE_URL=sqlite://agent-workspace.db
PORT=4000
```

For production:

```env
STORAGE_BACKEND=postgres
POSTGRES_URL=postgres://user:pass@localhost:5432/agent_workspace
```

### 3. Run

```bash
cargo run -p aw-api
# API available at http://localhost:4000
```

### 4. Register an agent

```bash
curl -s -X POST http://localhost:4000/agents \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "my-agent",
    "name": "My Agent",
    "role": "worker",
    "capabilities": ["analysis"],
    "permissions": []
  }'
```

Registration is **idempotent** — safe to call on every startup.

---

## Connecting agents

### Option A — MCP (for Claude and other MCP-capable agents)

Add to `.mcp.json`:

```json
{
  "mcpServers": {
    "agent-workspace": {
      "command": "/path/to/target/debug/aw-mcp",
      "env": {
        "SQLITE_URL": "sqlite:///path/to/agent-workspace.db"
      }
    }
  }
}
```

### Option B — Python SDK

```bash
pip install -e ./agent-workspace-sdk
```

```python
from agent_workspace import WorkspaceClient

client = WorkspaceClient(base_url="http://localhost:4000", agent_id="my-agent")

async with client.session() as session:
    task = await session.claim_task(task_id)
    # ... do work ...
    await session.update_task_status(task.id, "done")
    await session.send_message(to_agent_id="coordinator", kind="status_update", payload={})
```

### Option C — TypeScript / JavaScript SDK

```bash
cd agent-workspace-sdk-ts && npm install
```

```typescript
import { WorkspaceClient } from "@agent-workspace/sdk";

const client = new WorkspaceClient({
  baseUrl: "http://localhost:4000",
  agentId: "my-agent",
});

await client.withSession(async (session) => {
  const task = await session.claimTask(taskId);
  await session.updateTaskStatus(task.id!, "done");
  await session.sendMessage({
    toAgentId: "coordinator",
    kind: "status_update",
    payload: {},
  });
});
```

Both SDKs handle **heartbeat automatically** (every 45 s) and **check-out on exit** (even on exception).

### Option D — Raw HTTP

```bash
# Check in
curl -X POST http://localhost:4000/sessions/check-in \
  -H 'Content-Type: application/json' \
  -d '{"agent_id": "my-agent"}'

# Workspace snapshot
curl http://localhost:4000/summary | jq
```

---

## Agent session lifecycle

```
startup
  └─► check_in(agent_id)           → session_id, inbox, pending tasks, handoffs
        │
        ├─► [heartbeat every 45s]
        ├─► claim_task(task_id)
        ├─► update_task_status(task_id, "in_progress")
        ├─► acquire_lock(scope_type, scope_id)
        ├─► send_message(to_agent_id, kind, payload)
        └─► release_lock(lock_id)

shutdown
  └─► update_task_status(task_id, "done")
  └─► create_handoff(summary, payload)   ← optional: context for next agent
  └─► check_out(session_id)
```

---

## API reference

```
GET    /health                      → "ok" (no auth)
GET    /summary                     → workspace snapshot

POST   /agents                      register / update agent
GET    /agents                      list agents
GET    /agents/:id

POST   /sessions/check-in           { agent_id }
POST   /sessions/heartbeat          { session_id, health?, current_task_id? }
POST   /sessions/check-out          { session_id, create_handoff, summary?, payload? }
GET    /sessions/active

POST   /messages                    send message
GET    /messages?agent_id=&limit=

GET    /inbox/:agent_id             list pending inbox
POST   /inbox/:item_id/ack          { item_id, agent_id, status }

POST   /tasks                       create task
GET    /tasks?status=&unassigned=&assigned_to=&limit=
POST   /tasks/:id/claim             { agent_id, session_id }
POST   /tasks/:id/status            { status, metadata? }
POST   /tasks/:id/assign            { assigned_by, assigned_to? }

POST   /locks                       acquire lock (TTL-based)
DELETE /locks/:id                   release lock

POST   /handoffs                    create handoff
GET    /handoffs/:agent_id

POST   /dependencies                upsert dependency health
GET    /dependencies/:key

GET    /events?agent_id=&limit=     audit trail
```

All endpoints except `/health` require `Authorization: Bearer <token>` when auth is enabled.

---

## Maintenance

The server runs a background loop every **60 seconds**:

- Sessions without a heartbeat for **5 minutes** → marked `dead`, locks released
- Locks past their TTL → expired

Agents should heartbeat at least every **5 minutes** to stay alive. Both SDKs default to every 45 s.

---

## Observability

```bash
# Workspace state
curl http://localhost:4000/summary | jq

# Recent events for a specific agent
curl "http://localhost:4000/events?agent_id=my-agent&limit=50" | jq

# Active sessions
curl http://localhost:4000/sessions/active | jq
```

No dashboard exists by design — the API is the interface. Agents read `GET /summary` and act. Humans use `curl | jq`.

---

## Coordination pattern

Any agent can coordinate without configuration. Whoever reads the state and acts, coordinates.

```bash
# See what needs doing
curl http://localhost:4000/tasks?unassigned=true | jq

# Assign to a specific agent
curl -X POST http://localhost:4000/tasks/<id>/assign \
  -d '{"assigned_by": "coordinator", "assigned_to": "worker-1"}'

# Notify the assignee
curl -X POST http://localhost:4000/messages \
  -d '{"from_agent_id": "coordinator", "to_agent_id": "worker-1",
       "kind": "deferred_task", "payload": {}, "deliver_to_inbox": true}'
```

---

## Storage backends

| Backend        | Use case                    | Config                     |
| -------------- | --------------------------- | -------------------------- |
| **SQLite**     | Development, single-machine | `STORAGE_BACKEND=sqlite`   |
| **PostgreSQL** | Production, multi-process   | `STORAGE_BACKEND=postgres` |

Schema is applied automatically on startup via embedded migrations.

---

## Project layout

```
agent-workspace/
├── crates/
│   ├── domain/              # entities, storage trait, error types
│   ├── storage-sqlite/      # SQLite adapter (in-memory for tests)
│   ├── storage-postgres/    # PostgreSQL adapter (JSONB, TIMESTAMPTZ)
│   ├── storage-tests/       # shared integration test suite (macro)
│   ├── api/                 # Axum HTTP server
│   └── mcp/                 # MCP stdio server (rmcp)
├── agent-workspace-sdk/     # Python SDK (httpx + pydantic v2)
├── agent-workspace-sdk-ts/  # TypeScript SDK (Node 18+, zero deps)
├── frontend/                # Dashboard (Vite)
├── AGENT_GUIDE.md           # How agents should use the workspace
└── SDK_SPEC.md              # SDK specification
```

---

## Running tests

```bash
# SQLite integration tests (no setup required)
cargo test -p aw-storage-sqlite

# API tests (in-memory SQLite)
cargo test -p aw-api

# PostgreSQL integration tests (requires Docker)
cargo test -p aw-storage-postgres

# All at once
cargo test -p aw-storage-sqlite -p aw-api
```
