# Support Handoff Example (Cross-Language)

This example demonstrates how **Agent Workspace** enables interoperability between agents written in different languages using the `Handoff` primitive.

1. **L1 Support (TypeScript)**: Simulates a front-line bot interacting with a user. It gathers context and realizes the issue is too complex or requires database access. It checks out and explicitly creates a **handoff** containing the conversation summary and the raw payload for the L2 support.
2. **L2 Support (Python)**: An advanced engineering agent. When it checks in, it instantly receives pending handoffs. It reads the context from the TypeScript agent, "fixes" the problem, and marks the task as done.

## Prerequisites

Start the API server (e.g. `cargo run -p aw-api` on port `4000`), then:

```bash
# Setup Python environment
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Setup TypeScript environment
npm install
```

## Running the example

You can use the helper script to run them both sequentially (this replicates a real handoff flow):

```bash
./run.sh
```

Or manually:

1. `npm run start:l1` (Wait for it to finish and create the handoff)
2. `python l2_support.py` (It will pick up the handoff immediately upon check-in)
