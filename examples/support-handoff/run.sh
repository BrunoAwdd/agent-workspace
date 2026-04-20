#!/usr/bin/env bash

# Setup TS
echo "Installing TypeScript dependencies..."
npm install > /dev/null 2>&1

# Setup Python
echo "Setting up Python environment..."
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt > /dev/null 2>&1

echo "================================================="
echo "Starting Support Handoff Workflow (TS -> Python)"
echo "================================================="

# Start L1 (TS)
echo -e "\n---> Starting L1 Support Agent (TypeScript)..."
npm run start:l1

# Wait a brief moment to ensure checkout is fully processed by the workspace
sleep 2

# Start L2 (Python)
echo -e "\n---> Starting L2 Engineering Agent (Python)..."
python l2_support.py

echo -e "\n================================================="
echo "Workflow complete!"
