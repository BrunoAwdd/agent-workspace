import asyncio
import uuid
import sys
from agent_workspace import WorkspaceClient

async def main():
    print("🟣 [Coordinator] Checking in to Agent Workspace...")
    client = WorkspaceClient(agent_id="coordinator")
    await client.register(name="Task Coordinator", role="coordinator")

    async with client.session() as session:
        print("🟣 [Coordinator] Successfully checked in. Creating research tasks for 'OpenAI' briefing...")

        # 1. News task
        task1 = await session.create_task(
            title="Research OpenAI latest news and product releases",
            description="Scrape web for the latest major announcements over the last 30 days.",
            kind="analysis",
            priority="high"
        )
        print(f"   ✓ Created Task: {str(task1.id)[:8]} (News)")

        # 2. LinkedIn/People task
        task2 = await session.create_task(
            title="Research OpenAI leadership changes",
            description="Find recent executive departures and new hires.",
            kind="analysis",
            priority="normal"
        )
        print(f"   ✓ Created Task: {str(task2.id)[:8]} (People)")

        # 3. Financial task
        task3 = await session.create_task(
            title="Research OpenAI funding and valuation",
            description="Gather latest funding rounds, valuation cap, and major investors.",
            kind="analysis",
            priority="high"
        )
        print(f"   ✓ Created Task: {str(task3.id)[:8]} (Finance)")

        print("🟣 [Coordinator] All tasks seeded into the workspace to be claimed. Exiting.")

if __name__ == "__main__":
    asyncio.run(main())
