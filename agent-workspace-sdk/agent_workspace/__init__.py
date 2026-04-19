"""Agent Workspace SDK for Python.

Quick start::

    from agent_workspace import WorkspaceClient

    client = WorkspaceClient(
        base_url="http://localhost:4000",
        token="eyJhbGci...",
        agent_id="my-agent-1",
    )

    async with client.session() as session:
        task = await session.claim_task(task_id)
        await session.update_task_status(task.id, "done")
"""

from .client import WorkspaceClient
from .exceptions import (
    AuthError,
    ForbiddenError,
    LockConflictError,
    NotFoundError,
    TaskConflictError,
    WorkspaceError,
)
from .models import (
    Agent,
    AgentSessionData,
    Dependency,
    Event,
    Handoff,
    InboxItem,
    Lock,
    Message,
    Task,
    WorkspaceSummary,
)
from .session import AgentSession

__all__ = [
    # Client
    "WorkspaceClient",
    "AgentSession",
    # Models
    "Agent",
    "AgentSessionData",
    "Task",
    "InboxItem",
    "Message",
    "Lock",
    "Handoff",
    "Dependency",
    "Event",
    "WorkspaceSummary",
    # Exceptions
    "WorkspaceError",
    "AuthError",
    "ForbiddenError",
    "NotFoundError",
    "LockConflictError",
    "TaskConflictError",
]
