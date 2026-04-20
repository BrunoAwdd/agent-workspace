# Show HN: Agent Workspace – A shared coordination runtime for multi-agent systems

Hi HN,

I noticed a common problem when building multi-agent systems: they easily turn into messy, brittle webs of direct API calls, Kafka topics, or custom pub/sub queues. When you have an agent digging through news, another analyzing financials, and a third drafting a PDF, synchronizing their state across different languages and runtimes becomes a nightmare.

So I built **Agent Workspace**. It’s an open-source, language-agnostic operational workspace designed explicitly for agent coordination. Instead of direct messaging, agents check in to the workspace and coordinate using simple primitives:

- **Tasks** (pull-based distributed workload)
- **Locks** (preventing parallel overwrite conflicts on external resources)
- **Inboxes** (async handoffs and signaling)

It's essentially a fast, dedicated "blackboard" for your agents. It's built in Rust for a tiny memory footprint, exposing a pure REST API. We provide Python and TypeScript SDKs, so your Python researcher can hand off data directly to your TS ticket triage agent with zero extra infra overhead.

To visualize it all, there's a zero-config real-time Dashboard (Vite/React) bundled with the API that gives you a "command center" view of active agent sessions, tasks flowing through the kanban, and locks being acquired/released.

**Repo:** https://github.com/BrunoAwdd/agent-workspace

I've included a purely local `company-briefing` example workflow involving 4 parallel agents (3 async researchers fanning out, 1 compiler reducing) to demonstrate the primitives and dashboard reactivity in action.

I would love your feedback on the architecture and the primitives chosen. I'm considering an Open Core model for enterprise SSO/RBAC later (detailed in `PLANS.md`), but the core engine will remain fully open-source.

Happy to answer any questions!
