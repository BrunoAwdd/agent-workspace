import { WorkspaceClient, InboxStatus } from "@agent-workspace/sdk";

const API_URL = "http://localhost:4000";

async function main() {
  console.log("Starting Ticket Triage Example...\n");

  const triageBot = new WorkspaceClient({
    baseUrl: API_URL,
    agentId: "triage-bot",
  });
  const webhookInterface = new WorkspaceClient({
    baseUrl: API_URL,
    agentId: "webhook",
  });

  // Register agents
  await triageBot.registerAgent({
    name: "Triage Bot",
    role: "worker",
    capabilities: ["triage"],
  });
  await webhookInterface.registerAgent({
    name: "Webhook Entrypoint",
    role: "webhook",
    capabilities: [],
  });

  // Step 1: Webhook receives a ticket and pushes to the Triage Bot's inbox
  await webhookInterface.withSession(async (session) => {
    console.log(
      "[Webhook] Received raw ticket #9901 from customer. Pushing to triage...",
    );
    await session.sendMessage({
      toAgentId: "triage-bot",
      kind: "new_ticket",
      payload: {
        ticket_id: "9901",
        content: "My database is down!",
        urgency: "high",
      },
      deliverToInbox: true,
    });
    console.log("[Webhook] Ticket pushed. My job is done.");
  });

  // Step 2: Triage bot processes its inbox
  await triageBot.withSession(async (session) => {
    let attempts = 0;
    let processed = false;

    console.log(
      `[Triage Bot] Checking in... Inbox size: ${session.inbox.length}`,
    );

    while (!processed) {
      const inbox = await session.listInbox();

      if (inbox.length === 0) {
        console.log("[Triage Bot] Inbox empty, waiting...");
        await new Promise((r) => setTimeout(r, 2000));
        continue;
      }

      const item = inbox[0];
      attempts++;
      console.log(
        `\n[Triage Bot] Processing inbox item ${item.id} (Attempt ${attempts})...`,
      );

      if (attempts === 1) {
        // Simulate an API failure or transient error
        console.log(
          "[Triage Bot] Oh no! External AI provider timeout. Failing the item...",
        );
        await session.ackInboxItem(item.id, InboxStatus.Failed);
        console.log(
          "[Triage Bot] Item acked as FAILED. It will stay in the inbox for a retry.\n",
        );

        // Sleep to show the retry visually
        await new Promise((r) => setTimeout(r, 2000));
      } else {
        // Success on the second attempt
        console.log("[Triage Bot] External API is back. Processing ticket...");
        console.log(
          `[Triage Bot] Ticket ${item.payload.ticket_id} context: ${item.payload.content}`,
        );

        // Create an actual task now that triage is done
        const task = await session.createTask({
          title: `Fix DB for Ticket ${item.payload.ticket_id}`,
          description: "Database is down according to the customer.",
          kind: "engineering",
          priority: "high",
        });

        console.log(`[Triage Bot] Escalated to engineering task: ${task.id}`);

        // Mark the inbox item as done
        await session.ackInboxItem(item.id, InboxStatus.Done);
        console.log(
          "[Triage Bot] Item acked as DONE. It is now cleared from the inbox.",
        );
        processed = true;
      }
    }
  });

  console.log("\nWorkflow complete!");
}

main().catch(console.error);
