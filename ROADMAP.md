# Roadmap

_This roadmap is a living document outlining the direction of Agent Workspace for the Community Edition._

## Core Stable (Current)

- [x] REST API
- [x] Full PostgreSQL storage layer with `pg_notify` / JSONB
- [x] In-memory SQLite for testing/local dev
- [x] Session & Heartbeat tracking
- [x] Task primitives (claim, unassign, status updates)
- [x] PubSub Messaging & Guaranteed Inbox (with ack/retry)
- [x] Typed SDKs for Python (`httpx` + `pydantic v2`) and TypeScript (`fetch`)
- [x] Multi-language Handoff structure
- [x] Distributed TTL-based Locks

## Near Future (0-6 months)

- [ ] **Streaming API / SSE Support**: Enabling frontend dashboards and agents to receive real-time events without polling.
- [ ] **Auto-routing for Handoffs**: Currently handoffs require explicit query or direct targeted `to_agent_id`. We plan to add rule-based or queue-based generic handoffs.
- [ ] **Dependency Graphing Constraints**: allowing tasks to specify "depends_on_task_id", unlocking strict DAG execution paths.
- [ ] **MCP Official Agent Examples**: Expanding `aw-mcp` adoption via standardized Claude Desktop configurations.

## Far Future / Considering (Pro Tier / Enterprise)

- **Advanced Role-based Access Control (RBAC)**: granular permissioning over who can read which messages or claim which tasks.
- **SSO Integration**: For human operators interacting with workspace dashboards.
- **Metrics Exporter**: Prometheus/Grafana native endpoints.
