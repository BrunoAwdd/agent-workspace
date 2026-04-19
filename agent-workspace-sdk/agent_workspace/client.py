"""WorkspaceClient — the main entry point for the Agent Workspace SDK."""

from __future__ import annotations

import asyncio
import contextlib
from collections.abc import AsyncIterator
from typing import Any

import httpx

from .exceptions import (
    AuthError,
    ForbiddenError,
    LockConflictError,
    NotFoundError,
    TaskConflictError,
    WorkspaceError,
)
from .models import Agent, AgentSessionData, Event, Handoff, InboxItem, Task, WorkspaceSummary
from .session import AgentSession


def _raise_for_status(resp: httpx.Response) -> None:
    """Map HTTP error codes to typed SDK exceptions."""
    if resp.is_success:
        return
    try:
        body = resp.json()
        message: str = body.get("error", resp.text)
    except Exception:
        message = resp.text

    code = resp.status_code
    if code == 401:
        raise AuthError(message, code)
    if code == 403:
        raise ForbiddenError(message, code)
    if code == 404:
        raise NotFoundError(message, code)
    if code == 409:
        low = message.lower()
        if "lock" in low:
            raise LockConflictError(message, code)
        raise TaskConflictError(message, code)
    raise WorkspaceError(message, code)


class WorkspaceClient:
    """Client for the Agent Workspace REST API.

    Args:
        base_url: Base URL of the workspace server (default: http://localhost:4000).
        token: JWT Bearer token. Optional in dev mode (no auth).
        agent_id: ID of the agent using this client.
        heartbeat_interval_secs: How often to send heartbeats (default: 45).
        auto_heartbeat: Whether sessions should heartbeat automatically (default: True).
    """

    def __init__(
        self,
        *,
        base_url: str = "http://localhost:4000",
        token: str | None = None,
        agent_id: str,
        heartbeat_interval_secs: int = 45,
        auto_heartbeat: bool = True,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._agent_id = agent_id
        self._heartbeat_interval = heartbeat_interval_secs
        self._auto_heartbeat = auto_heartbeat

        headers: dict[str, str] = {"Content-Type": "application/json"}
        if token:
            headers["Authorization"] = f"Bearer {token}"

        self._http = httpx.AsyncClient(
            base_url=self._base_url,
            headers=headers,
            timeout=30.0,
        )

    # ------------------------------------------------------------------
    # Low-level HTTP helpers
    # ------------------------------------------------------------------

    async def _get(self, path: str, *, params: dict[str, str] | None = None) -> Any:
        resp = await self._http.get(path, params=params)
        _raise_for_status(resp)
        return resp.json()

    async def _post(self, path: str, body: Any = None) -> Any:
        resp = await self._http.post(path, json=body)
        _raise_for_status(resp)
        return resp.json()

    async def _delete(self, path: str, body: Any = None) -> Any:
        resp = await self._http.request("DELETE", path, json=body)
        _raise_for_status(resp)
        # Some DELETE endpoints return empty body
        if resp.content:
            return resp.json()
        return None

    # ------------------------------------------------------------------
    # Agent registration
    # ------------------------------------------------------------------

    async def register(
        self,
        *,
        id: str | None = None,
        name: str,
        role: str,
        capabilities: list[str] | None = None,
        permissions: list[str] | None = None,
    ) -> Agent:
        """Register (or update) this agent. Idempotent — safe to call on every startup."""
        data = await self._post(
            "/agents",
            {
                "id": id or self._agent_id,
                "name": name,
                "role": role,
                "capabilities": capabilities or [],
                "permissions": permissions or [],
            },
        )
        return Agent.model_validate(data)

    # ------------------------------------------------------------------
    # Session lifecycle
    # ------------------------------------------------------------------

    async def check_in(self) -> AgentSession:
        """Open a new session and return an AgentSession.

        Performs the check-in then fetches inbox, pending tasks and handoffs
        in parallel so the session object is fully populated on return.
        """
        raw = await self._post("/sessions/check-in", {"agent_id": self._agent_id})
        session_data = AgentSessionData.model_validate(raw)

        # Enrich in parallel
        inbox_fut = self._get(f"/inbox/{self._agent_id}")
        tasks_fut = self._get(
            "/tasks",
            params={"assigned_to": self._agent_id, "status": "open,claimed"},
        )
        handoffs_fut = self._get(f"/handoffs/{self._agent_id}")

        inbox_raw, tasks_raw, handoffs_raw = await asyncio.gather(
            inbox_fut, tasks_fut, handoffs_fut, return_exceptions=True
        )

        def _safe_list(raw: Any, model: type) -> list:
            if isinstance(raw, Exception):
                return []
            items = raw.get("items", raw) if isinstance(raw, dict) else raw
            try:
                return [model.model_validate(i) for i in (items or [])]
            except Exception:
                return []

        inbox = _safe_list(inbox_raw, InboxItem)
        pending_tasks = _safe_list(tasks_raw, Task)
        pending_handoffs = _safe_list(handoffs_raw, Handoff)

        return AgentSession(
            client=self,
            session_id=session_data.id,
            agent_id=self._agent_id,
            inbox=inbox,
            pending_tasks=pending_tasks,
            pending_handoffs=pending_handoffs,
            heartbeat_interval_secs=self._heartbeat_interval,
            auto_heartbeat=self._auto_heartbeat,
        )

    @contextlib.asynccontextmanager
    async def session(self) -> AsyncIterator[AgentSession]:
        """Async context manager: checks in, yields the session, checks out on exit."""
        s = await self.check_in()
        try:
            yield s
        finally:
            with contextlib.suppress(Exception):
                await s.check_out()

    # ------------------------------------------------------------------
    # Workspace-level (no session required)
    # ------------------------------------------------------------------

    async def get_summary(self) -> WorkspaceSummary:
        data = await self._get("/summary")
        return WorkspaceSummary.model_validate(data)

    async def list_events(
        self,
        agent_id: str | None = None,
        limit: int = 50,
    ) -> list[Event]:
        params: dict[str, str] = {"limit": str(limit)}
        if agent_id:
            params["agent_id"] = agent_id
        data = await self._get("/events", params=params)
        items = data.get("items", data) if isinstance(data, dict) else data
        return [Event.model_validate(e) for e in items]

    async def list_agents(self) -> list[Agent]:
        data = await self._get("/agents")
        items = data.get("items", data) if isinstance(data, dict) else data
        return [Agent.model_validate(a) for a in items]

    # ------------------------------------------------------------------
    # Cleanup
    # ------------------------------------------------------------------

    async def aclose(self) -> None:
        """Close the underlying HTTP client."""
        await self._http.aclose()

    async def __aenter__(self) -> "WorkspaceClient":
        return self

    async def __aexit__(self, *_: Any) -> None:
        await self.aclose()
