import asyncio
import sys
from agent_workspace import WorkspaceClient

async def main():
    # Provide the base URL of your running Agent Workspace server
    client = WorkspaceClient(base_url="http://localhost:4000", agent_id="coordinator")

    print("[Coordinator] Registering agent...")
    await client.register_agent(
        name="Workflow Coordinator",
        role="coordinator",
        capabilities=["task_management"]
    )

    async with client.session() as session:
        print("[Coordinator] Connected! Creating tasks...")

        # Create a research task
        research_task = await session.create_task(
            title="Research Quantum Computing",
            description="Find recent advancements in quantum computing stability.",
            kind="research",
            priority="high",
            metadata={"topic": "Quantum Computing"}
        )
        print(f"[Coordinator] Created Research Task: {research_task.id}")

        # Create a writing task
        write_task = await session.create_task(
            title="Write Article on Quantum Computing",
            description="Use the research findings to write a short, engaging article.",
            kind="writing",
            priority="normal",
            metadata={"topic": "Quantum Computing", "linked_research_task": str(research_task.id)}
        )
        print(f"[Coordinator] Created Writing Task: {write_task.id}")

        print("[Coordinator] Tasks created successfully. Exiting.")

if __name__ == "__main__":
    asyncio.run(main())
