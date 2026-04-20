# Researcher + Writer Example

This example demonstrates a classic multi-agent workflow using the **Agent Workspace** Python SDK.

It consists of three roles:

1. **Coordinator**: Creates tasks for the researcher and the writer.
2. **Researcher**: Claims open research tasks, generates findings, and sends them as a message to the writer.
3. **Writer**: Claims open writing tasks, waits for messages with findings, and produces the final output.

## Prerequisites

1. Have the `agent-workspace` server running (e.g. `cargo run -p aw-api` on port `4000`).
2. Install the Python SDK and dependencies.

```bash
pip install -r requirements.txt
```

## Running the example

You can run the script that launches all 3 agents:

```bash
./run.sh
```

Or you can run them manually in separate terminals:

```bash
# Terminal 1: Writer
python writer.py

# Terminal 2: Researcher
python researcher.py

# Terminal 3: Coordinator (creates the work)
python coordinator.py
```

Notice how the Writer will sit idle waiting for the message from the Researcher, demonstrating proper async decoupling of multi-agent workflows!
