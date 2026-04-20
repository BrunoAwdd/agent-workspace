import asyncio
from agent_workspace import WorkspaceClient, exceptions

async def main():
    client = WorkspaceClient(base_url="http://localhost:4000", agent_id="writer")

    print("[Writer] Registering agent...")
    await client.register_agent(
        name="Bob the Writer",
        role="worker",
        capabilities=["writing"]
    )

    async with client.session() as session:
        print(f"[Writer] Checked in! Found {len(session.inbox)} items in inbox.")

        while True:
            # 1. Claim a writing task
            tasks = await session.list_tasks(status="open", unassigned_only=True)
            writing_tasks = [t for t in tasks if t.kind == "writing"]

            if not writing_tasks:
                print("[Writer] No writing tasks found. Waiting...")
                await asyncio.sleep(3)
                continue

            task = writing_tasks[0]
            print(f"[Writer] Found task '{task.title}' (ID: {task.id}). Claiming...")

            try:
                await session.claim_task(task.id)
            except exceptions.TaskConflictError:
                print("[Writer] Task claimed by someone else!")
                continue

            print("[Writer] Task claimed. Now waiting for research data...")
            
            # 2. Wait for research data in the inbox
            findings = None
            while not findings:
                # Refresh inbox
                inbox = await session.list_inbox()
                
                for item in inbox:
                    if item.kind == "research_data":
                        findings = item.payload.get("findings")
                        print(f"[Writer] Received research data in inbox: {findings}")
                        # Acknowledge the inbox item so it doesn't appear again
                        await session.ack_inbox_item(item.id, "done")
                        break
                
                if not findings:
                    print("[Writer] Still waiting for research data... (sleeping 2s)")
                    await asyncio.sleep(2)

            # 3. Simulate writing
            print("[Writer] Drafting article based on findings...")
            await asyncio.sleep(2)
            draft = f"Wow! Did you know: {findings}"
            print(f"[Writer] Draft complete: \n\n{draft}\n")

            # 4. Mark task as done
            print("[Writer] Marking task as done.")
            await session.update_task_status(task.id, "done")
            
            # End after one loop
            break

if __name__ == "__main__":
    asyncio.run(main())
