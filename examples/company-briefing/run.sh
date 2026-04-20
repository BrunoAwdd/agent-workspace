#!/bin/bash
set -e

echo "=========================================================="
echo "    Agent Workspace — Company Briefing Example"
echo "=========================================================="
echo "This workflow orchestrates 5 agents:"
echo " 1. coordinator => seeds tasks"
echo " 2. researcher-news => scrapes latest news"
echo " 3. researcher-linkedin => scrapes people data"
echo " 4. researcher-finance => gets funding data"
echo " 5. compiler => receives data in inbox, writes final brief"
echo ""

# Ensure we're in a venv
if [ ! -d "venv" ]; then
    echo "Creating python venv..."
    python3 -m venv venv
    source venv/bin/activate
    pip install -e ../../agent-workspace-sdk > /dev/null
else
    source venv/bin/activate
fi

# 1. Run coordinator to seed the tasks
echo ""
echo "🟣 [Coordinator] Starting up..."
python3 coordinator.py

# 2. Run researchers and compiler in parallel!
echo ""
echo "Starting parallel agents (3 Researchers + 1 Compiler)..."

python3 researcher.py "researcher-news" &
python3 researcher.py "researcher-linkedin" &
python3 researcher.py "researcher-finance" &
python3 compiler.py &

# Wait for them to finish
wait

echo "=========================================================="
echo "✅ Workflow complete!"
