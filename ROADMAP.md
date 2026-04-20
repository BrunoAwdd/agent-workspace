# Roadmap

_This roadmap is a living document outlining the direction of Agent Workspace._

### 🛠️ Near Term (0-6 months)

- **Examples:** More ready-to-run workflows demonstrating multi-agent patterns in real environments.
- **SDK Polish:** Feature parity across Python and TypeScript SDKs, focusing on auto-heartbeat resilience and strict typing.
- **Docs:** Deep-dive guides into advanced coordination primitives (Locks and Inter-agent Box retries).
- **Testing:** Expanding SQLite and native PostgreSQL test coverage for high-concurrency races.
- **Auth Clarity:** Solidify standard JWT bearer auth paths for the open-source release.

### 🔭 Mid Term (6-12 months)

- **Production Hardening:** Eliminating database contention scenarios in massive deployments.
- **Observability:** Adding official `/metrics` endpoints and tracing hooks natively available for Prometheus/Grafana.
- **Integrations:** More official MCP tooling so Claude Desktop and other UIs can query the workspace flawlessly without coding.

### 🏢 Long Term (12+ months)

- **Enterprise Controls:** Robust Multi-tenant isolation, RBAC (Role-Based Access Control), and SSO mappings.
- **Hosted Version:** True zero-configuration deploy for Agent Workspace.
- **Governance Layer:** Policy engines allowing explicit rules on which agent can claim/see which tasks.

_See [PLANS.md](PLANS.md) for more details on what falls under the Community Edition vs. Future Enterprise additions._
