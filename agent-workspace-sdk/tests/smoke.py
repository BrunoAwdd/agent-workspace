"""Smoke test — requires aw-api running on http://localhost:4000 without auth."""

import asyncio
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from agent_workspace import WorkspaceClient, TaskConflictError


async def main() -> None:
    client = WorkspaceClient(base_url="http://localhost:4000", agent_id="smoke-test-agent")

    print("1. Registering agent...")
    agent = await client.register(name="Smoke Test Agent", role="tester", capabilities=["smoke"])
    print(f"   OK: {agent.id}")

    print("2. check_in + enrichment...")
    async with client.session() as session:
        print(f"   session.id      = {session.id}")
        print(f"   pending_tasks   = {len(session.pending_tasks)}")
        print(f"   inbox           = {len(session.inbox)}")
        print(f"   pending_handoffs= {len(session.pending_handoffs)}")

        print("3. create_task...")
        task = await session.create_task(
            "Smoke task", "Created by smoke test", kind="custom:smoke", priority="low"
        )
        print(f"   task.id = {task.id}")

        print("4. claim_task...")
        claimed = await session.claim_task(str(task.id))
        print(f"   status = {claimed.status}")

        print("5. conflict detection (claim again)...")
        try:
            await session.claim_task(str(task.id))
            print("   FAIL: expected TaskConflictError")
        except TaskConflictError:
            print("   OK: TaskConflictError raised correctly")

        print("6. update_task_status → done...")
        updated = await session.update_task_status(str(task.id), "done")
        print(f"   status = {updated.status}")

        print("7. send_message...")
        await session.send_message(
            to_agent_id="smoke-test-agent",
            kind="status_update",
            payload={"msg": "smoke complete"},
        )
        print("   OK")

        print("8. list_inbox...")
        items = await session.list_inbox()
        print(f"   {len(items)} item(s)")

        print("9. workspace summary...")
        summary = await client.get_summary()
        print(f"   active_sessions={summary.active_sessions}")

    print("\n✅ All smoke tests passed")


asyncio.run(main())
