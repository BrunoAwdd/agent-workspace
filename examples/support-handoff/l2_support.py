import asyncio
from agent_workspace import WorkspaceClient

async def main():
    client = WorkspaceClient(base_url="http://localhost:4000", agent_id="l2-bot")

    print("[L2 Support] Registering agent...")
    await client.register_agent(
        name="L2 Engineering Support (Python)",
        role="engineering",
        capabilities=["db_access", "incident_response"]
    )

    async with client.session() as session:
        print(f"[L2 Support] Checked in. Session ID: {session.id}")
        
        # Check for handoffs immediately upon check-in
        # Handoffs are scoped to the agent_id if `to_agent_id` is set,
        # otherwise we can query all pending handoffs (if the API supports it)
        # Note: In the current SDK, check_in() returns pending_handoffs for this agent natively!
        
        # Assuming the workspace returns handoffs during check-in or via a list call:
        # For this example, let's just query the /handoffs endpoint for our agent id
        # (if auto-routing is set up), but since L1 left `to_agent_id` null, 
        # let's assume we can query unassigned tasks or we use the initial check-in data.
        
        # Let's check the session's initial handoffs:
        if not session._initial_handoffs:
            print("[L2 Support] No direct handoffs found. Doing a fallback search if needed...")
            # For demonstration, we simulate finding the task created by L1 if handoff isn't directly routed.
            tasks = await session.list_tasks(status="open")
            bug_tasks = [t for t in tasks if t.kind == "bug"]
            if bug_tasks:
                task = bug_tasks[0]
                print(f"[L2 Support] Found a pending bug task left by L1: {task.id}")
                await session.claim_task(task.id)
                print("[L2 Support] Investigating DB issues...")
                await asyncio.sleep(2)
                print("[L2 Support] Found the missing tenant ID. Fixing...")
                await session.update_task_status(task.id, "done")
                print("[L2 Support] Fixed and marked task as done.")
            else:
                print("[L2 Support] No pending items. Gracefully exiting.")
        else:
            # If the Handoff was routed to us
            for handoff in session._initial_handoffs:
                print(f"[L2 Support] Picked up handoff from {handoff.from_agent_id}!")
                print(f"[L2 Support] Context: {handoff.summary}")
                print(f"[L2 Support] Payload: {handoff.payload}")
                
                # claim the linked task
                task_id = handoff.payload.get("original_task_id")
                if task_id:
                    await session.claim_task(task_id)
                    print(f"[L2 Support] Claimed original task {task_id}.")
                    
                print("[L2 Support] Applying fix to database...")
                await asyncio.sleep(2)
                print("[L2 Support] Fix applied.")
                
                if task_id:
                    await session.update_task_status(task_id, "done")
                    print("[L2 Support] Task completed.")
                
        print("[L2 Support] Workflow finished. Checking out.")

if __name__ == "__main__":
    asyncio.run(main())
