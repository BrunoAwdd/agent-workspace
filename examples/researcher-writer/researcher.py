import asyncio
from agent_workspace import WorkspaceClient, exceptions

async def main():
    client = WorkspaceClient(base_url="http://localhost:4000", agent_id="researcher")

    print("[Researcher] Registering agent...")
    await client.register_agent(
        name="Alice the Researcher",
        role="worker",
        capabilities=["research"]
    )

    async with client.session() as session:
        print("[Researcher] Checked in. Looking for 'research' tasks...")

        while True:
            # Poll for unassigned tasks
            tasks = await session.list_tasks(status="open", unassigned_only=True)
            research_tasks = [t for t in tasks if t.kind == "research"]

            if not research_tasks:
                print("[Researcher] No research tasks found. Waiting...")
                await asyncio.sleep(3)
                continue

            task = research_tasks[0]
            print(f"[Researcher] Found task '{task.title}' (ID: {task.id}). Claiming...")

            try:
                await session.claim_task(task.id)
            except exceptions.TaskConflictError:
                print("[Researcher] Someone else claimed it first!")
                continue

            # Simulate doing the research
            print("[Researcher] Task claimed. Starting research...")
            await asyncio.sleep(2)
            findings = "Quantum stability improved by 40% using new cooling structures."
            print(f"[Researcher] Research complete! Findings: {findings}")

            # Send a message to the writer with the findings
            # Note: We send this to the inbox so the writer gets it even if they check in later
            print("[Researcher] Sending findings to the Writer...")
            await session.send_message(
                to_agent_id="writer",
                kind="research_data",
                payload={"findings": findings, "topic": task.metadata.get("topic", "Unknown")},
                deliver_to_inbox=True
            )

            # Mark task as done
            print("[Researcher] Marking task as done.")
            await session.update_task_status(task.id, "done")
            
            # For this example, we exit after one task
            break

if __name__ == "__main__":
    asyncio.run(main())
