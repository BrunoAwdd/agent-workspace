"""AgentSession — wraps a live workspace session with automatic heartbeat."""

from __future__ import annotations

import asyncio
import logging
from typing import TYPE_CHECKING, Any
from uuid import UUID

from .models import Handoff, InboxItem, Lock, Task

if TYPE_CHECKING:
    from .client import WorkspaceClient

logger = logging.getLogger(__name__)


class AgentSession:
    """Represents an active workspace session.

    Obtain via ``WorkspaceClient.check_in()`` or use ``client.session()``
    as an async context manager for automatic check-out.
    """

    def __init__(
        self,
        client: "WorkspaceClient",
        session_id: UUID,
        agent_id: str,
        inbox: list[InboxItem],
        pending_tasks: list[Task],
        pending_handoffs: list[Handoff],
        *,
        heartbeat_interval_secs: int = 45,
        auto_heartbeat: bool = True,
    ) -> None:
        self._client = client
        self._id = session_id
        self._agent_id = agent_id
        self._inbox = inbox
        self._pending_tasks = pending_tasks
        self._pending_handoffs = pending_handoffs
        self._heartbeat_interval = heartbeat_interval_secs
        self._auto_heartbeat = auto_heartbeat
        self._heartbeat_task: asyncio.Task[None] | None = None

        if auto_heartbeat:
            self._start_heartbeat_loop()

    # ------------------------------------------------------------------
    # Properties
    # ------------------------------------------------------------------

    @property
    def id(self) -> UUID:
        return self._id

    @property
    def agent_id(self) -> str:
        return self._agent_id

    @property
    def inbox(self) -> list[InboxItem]:
        return self._inbox

    @property
    def pending_tasks(self) -> list[Task]:
        return self._pending_tasks

    @property
    def pending_handoffs(self) -> list[Handoff]:
        return self._pending_handoffs

    # ------------------------------------------------------------------
    # Heartbeat
    # ------------------------------------------------------------------

    def _start_heartbeat_loop(self) -> None:
        self._heartbeat_task = asyncio.create_task(self._run_heartbeat())

    async def _run_heartbeat(self) -> None:
        while True:
            await asyncio.sleep(self._heartbeat_interval)
            try:
                await self.heartbeat()
            except Exception as exc:
                logger.warning("Heartbeat failed (session %s): %s", self._id, exc)

    async def heartbeat(
        self,
        health: str = "healthy",
        current_task_id: str | UUID | None = None,
    ) -> None:
        """Send a heartbeat to keep the session alive."""
        body: dict[str, Any] = {"session_id": str(self._id), "health": health}
        if current_task_id is not None:
            body["current_task_id"] = str(current_task_id)
        await self._client._post("/sessions/heartbeat", body)

    def _stop_heartbeat(self) -> None:
        if self._heartbeat_task and not self._heartbeat_task.done():
            self._heartbeat_task.cancel()

    # ------------------------------------------------------------------
    # Check-out
    # ------------------------------------------------------------------

    async def check_out(
        self,
        *,
        create_handoff: bool = False,
        summary: str | None = None,
        payload: dict[str, Any] | None = None,
    ) -> None:
        """End the session. Stops the heartbeat loop automatically."""
        self._stop_heartbeat()
        body: dict[str, Any] = {
            "session_id": str(self._id),
            "create_handoff": create_handoff,
        }
        if summary is not None:
            body["handoff_summary"] = summary
        if payload is not None:
            body["handoff_payload"] = payload
        await self._client._post("/sessions/check-out", body)

    # ------------------------------------------------------------------
    # Async context manager
    # ------------------------------------------------------------------

    async def __aenter__(self) -> "AgentSession":
        return self

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        await self.check_out()

    # ------------------------------------------------------------------
    # Messages
    # ------------------------------------------------------------------

    async def send_message(
        self,
        *,
        to_agent_id: str,
        kind: str,
        payload: dict[str, Any] | None = None,
        deliver_to_inbox: bool = True,
        workspace_id: str = "main",
    ) -> None:
        """Send a message to another agent."""
        await self._client._post(
            "/messages",
            {
                "workspace_id": workspace_id,
                "from_agent_id": self._agent_id,
                "to_agent_id": to_agent_id,
                "kind": kind,
                "payload": payload or {},
                "deliver_to_inbox": deliver_to_inbox,
            },
        )

    # ------------------------------------------------------------------
    # Inbox
    # ------------------------------------------------------------------

    async def list_inbox(self) -> list[InboxItem]:
        """Fetch current pending inbox items."""
        data = await self._client._get(f"/inbox/{self._agent_id}")
        items = data.get("items", data) if isinstance(data, dict) else data
        return [InboxItem.model_validate(i) for i in items]

    async def ack(self, item_id: str | UUID, status: str = "done") -> None:
        """Acknowledge an inbox item."""
        await self._client._post(
            f"/inbox/{item_id}/ack",
            {"item_id": str(item_id), "agent_id": self._agent_id, "status": status},
        )

    # ------------------------------------------------------------------
    # Tasks
    # ------------------------------------------------------------------

    async def create_task(
        self,
        title: str,
        description: str | None = None,
        kind: str = "custom:default",
        priority: str = "normal",
        metadata: dict[str, Any] | None = None,
    ) -> Task:
        data = await self._client._post(
            "/tasks",
            {
                "title": title,
                "description": description,
                "kind": kind,
                "priority": priority,
                "metadata": metadata or {},
            },
        )
        return Task.model_validate(data)

    async def claim_task(self, task_id: str | UUID) -> Task:
        data = await self._client._post(
            f"/tasks/{task_id}/claim",
            {"agent_id": self._agent_id, "session_id": str(self._id)},
        )
        return Task.model_validate(data)

    async def list_tasks(
        self,
        *,
        status: list[str] | None = None,
        unassigned_only: bool = False,
        limit: int | None = None,
    ) -> list[Task]:
        params: dict[str, str] = {}
        if status:
            params["status"] = ",".join(status)
        if unassigned_only:
            params["unassigned"] = "true"
        if limit is not None:
            params["limit"] = str(limit)
        data = await self._client._get("/tasks", params=params)
        items = data.get("items", data) if isinstance(data, dict) else data
        return [Task.model_validate(t) for t in items]

    async def update_task_status(
        self,
        task_id: str | UUID,
        status: str,
        metadata: dict[str, Any] | None = None,
    ) -> Task:
        data = await self._client._post(
            f"/tasks/{task_id}/status",
            {"status": status, "metadata": metadata or {}},
        )
        return Task.model_validate(data)

    async def assign_task(
        self,
        task_id: str | UUID,
        assigned_to: str | None,
    ) -> Task:
        data = await self._client._post(
            f"/tasks/{task_id}/assign",
            {"assigned_by": self._agent_id, "assigned_to": assigned_to},
        )
        return Task.model_validate(data)

    # ------------------------------------------------------------------
    # Locks
    # ------------------------------------------------------------------

    async def acquire_lock(
        self,
        scope_type: str,
        scope_id: str,
        lock_type: str = "write_lock",
        ttl_secs: int = 300,
    ) -> Lock:
        data = await self._client._post(
            "/locks",
            {
                "scope_type": scope_type,
                "scope_id": scope_id,
                "lock_type": lock_type,
                "owner_agent_id": self._agent_id,
                "owner_session_id": str(self._id),
                "ttl_secs": ttl_secs,
            },
        )
        return Lock.model_validate(data)

    async def release_lock(self, lock_id: str | UUID) -> None:
        await self._client._delete(
            f"/locks/{lock_id}",
            {"lock_id": str(lock_id), "owner_session_id": str(self._id)},
        )

    # ------------------------------------------------------------------
    # Handoffs
    # ------------------------------------------------------------------

    async def create_handoff(
        self,
        *,
        summary: str,
        to_agent_id: str | None = None,
        task_id: str | UUID | None = None,
        payload: dict[str, Any] | None = None,
    ) -> Handoff:
        body: dict[str, Any] = {
            "from_agent_id": self._agent_id,
            "source_session_id": str(self._id),
            "summary": summary,
            "payload": payload or {},
        }
        if to_agent_id:
            body["to_agent_id"] = to_agent_id
        if task_id:
            body["task_id"] = str(task_id)
        data = await self._client._post("/handoffs", body)
        return Handoff.model_validate(data)

    async def list_handoffs(self) -> list[Handoff]:
        data = await self._client._get(f"/handoffs/{self._agent_id}")
        items = data.get("items", data) if isinstance(data, dict) else data
        return [Handoff.model_validate(h) for h in items]
