import asyncio
from agent_workspace import WorkspaceClient

async def main():
    agent_id = "compiler"
    print(f"🟠 [{agent_id}] Formatting Agent checked in. Waiting for 3 research streams...")
    
    payloads_received = []

    client = WorkspaceClient(agent_id=agent_id)
    await client.register(name="Briefing Compiler", role="compiler")
    async with client.session() as session:
        while len(payloads_received) < 3:
            # Poll for completed analysis tasks
            tasks = await session.list_tasks(status=["done"])
            analysis_tasks = [t for t in tasks if t.kind == "analysis"]
            
            payloads_received = []
            for t in analysis_tasks:
                if t.metadata and "source" in t.metadata:
                    payloads_received.append(t.metadata)
                    
            if len(payloads_received) < 3:
                await asyncio.sleep(1.0)
                
        print(f"🟠 [{agent_id}] All 3 research chunks received!")
        print(f"🟠 [{agent_id}] Mocking PDF/Markdown generation...")
        await asyncio.sleep(2.0)
        
        print("================================================")
        print("            FINAL COMPILED BRIEFING             ")
        print("================================================")
        for p in payloads_received:
            print(f"- {p['source']}: {p['data']}")
        print("================================================")
        print(f"🟠 [{agent_id}] Report generated successfully. Shutting down.")

if __name__ == "__main__":
    asyncio.run(main())
