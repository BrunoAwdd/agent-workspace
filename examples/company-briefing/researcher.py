import asyncio
import sys
import random
from agent_workspace import WorkspaceClient

async def main():
    if len(sys.argv) < 2:
        print("Usage: python researcher.py <name>")
        sys.exit(1)

    agent_id = sys.argv[1]
    
    # We assign distinct capabilities based on agent name
    target_keyword = "news"
    if "linkedin" in agent_id:
        target_keyword = "leadership"
    elif "finance" in agent_id:
        target_keyword = "funding"

    print(f"🟢 [{agent_id}] Waking up and checking in...")
    client = WorkspaceClient(agent_id=agent_id)
    await client.register(name=f"{agent_id.replace('-', ' ').title()}", role="researcher")

    async with client.session() as session:
        print(f"🟢 [{agent_id}] Looking for tasks matching my domain ({target_keyword})...")
        
        while True:
            # List unassigned open tasks
            tasks = await session.list_tasks(status=["open"])
            
            target_task = None
            for t in tasks:
                if t.assigned_agent_id is None and target_keyword in t.title.lower():
                    target_task = t
                    break

            if target_task:
                print(f"🟢 [{agent_id}] Found task: {target_task.title}. Claiming {str(target_task.id)[:8]}...")
                await session.claim_task(target_task.id)
                await session.update_task_status(target_task.id, "in_progress")

                # Simulate long heavy research computation
                sleep_time = random.uniform(3.0, 7.0)
                print(f"🟢 [{agent_id}] Researching {target_task.title} for {sleep_time:.1f} seconds...")
                await asyncio.sleep(sleep_time)

                # Mark task as done and attach data to metadata
                print(f"🟢 [{agent_id}] Research done! Attaching payload to Task metadata.")
                await session.update_task_status(
                    task_id=target_task.id,
                    status="done",
                    metadata={"source": agent_id, "data": f"Simulated heavy analysis payload for {target_keyword}."}
                )
                print(f"🟢 [{agent_id}] Task completed and closed. Shutting down.")
                break
            else:
                await asyncio.sleep(1)

if __name__ == "__main__":
    asyncio.run(main())
