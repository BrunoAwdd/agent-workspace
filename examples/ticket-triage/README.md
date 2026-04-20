# Ticket Triage Example (TypeScript)

This example demonstrates agent communication via the **Inbox** using the TypeScript SDK.
It shows how an agent can safely process messages and use the retry mechanism if an operation fails.

It uses two agents:

1. **Webhook Interface**: Acts as the entrypoint for raw customer tickets. It sends tickets as `new_ticket` messages directly into the inbox of the Triage Bot.
2. **Triage Bot**: Polls its inbox and tries to process new tickets. To demonstrate the Workspace's robust retry mechanics, the bot will intentionally **fail** on the first try, acknowledging the message as `"failed"`. The workspace will requeue the message. On the second try, it will succeed and acknowledge it as `"done"`.

## Prerequisites

Start the API server (e.g. `cargo run -p aw-api` on port `4000`), then:

```bash
# Provide the TS SDK internally
npm install
npm run start
```
