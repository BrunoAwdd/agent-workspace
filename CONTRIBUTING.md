# Contributing to Agent Workspace

First of all, thank you for your interest! The goal of **Agent Workspace** is to provide the standard orchestration layer for multi-agent applications. To get there quickly, we welcome community feedback, pull requests, and usage examples.

## We operate on an Open Core model (Community + Pro)

The code in this repository represents the fully featured, production-ready **Community Edition**, licensed under the permissive Apache 2.0 license. This builds the core multi-agent coordination layer (sessions, messaging, inbox, tasks, locks, events).

In the future, we may introduce a distinct Pro/Enterprise tier designed for advanced fleet management, RBAC, SSO, and massive horizontal scaling, but the core primitives will always remain open and robust here.

## How to Contribute

### 1. Examples and Integrations

We love seeing how you're using Agent Workspace! If you have built an exciting toy-project or agent workflow (using any LLM framework or standalone code), feel free to submit it to the `examples/` directory.

### 2. Bug Reports

If you run into panics, locking failures, or strange inbox retry behaviors:

1. Ensure you are on the latest `master` branch.
2. Check the existing issues.
3. Open a new Bug Report using the issue template. Include logs and steps to reproduce.

### 3. Feature Requests

We prioritize features that align with the "Workspace" philosophy: _primitives for agents to coordinate, not the agents themselves_. If you have an idea, open a specific Feature Request using the issue template. Let's discuss the API design before you start coding!

### 4. Code Contributions

1. Fork the repository and create a branch.
2. Write tests for your changes. `crates/storage-tests` provides a unified test suite to guarantee adapter parity.
3. Keep PRs small and focused.
4. Ensure `cargo test` passes cleanly.

## Local Development Setup

To run the full suite locally:

```bash
# Rust components (API and SQLite storage)
cargo build
cargo test

# Docker is required for the PostgreSQL integration tests
cargo test -p aw-storage-postgres

# TypeScript SDK
cd agent-workspace-sdk-ts
npm install
npm run build

# Python SDK
cd agent-workspace-sdk
pip install -e .
```
