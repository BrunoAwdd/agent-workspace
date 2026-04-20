#!/usr/bin/env bash

# Create a clean virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies (SDK)
pip install -r requirements.txt

echo "======================================"
echo "Starting Researcher-Writer Workflow..."
echo "======================================"

# Start Writer and Researcher in the background
python writer.py &
WRITER_PID=$!

python researcher.py &
RESEARCHER_PID=$!

# Give them a second to start
sleep 2

# Coordinator kicks off the tasks
python coordinator.py

# Wait a little for the workflow to complete
echo "[Main] Waiting for the agents to finish their tasks..."
sleep 8

# Cleanup
echo "[Main] Workflow complete. Shutting down agents..."
kill $WRITER_PID
kill $RESEARCHER_PID
echo "[Main] All done!"
