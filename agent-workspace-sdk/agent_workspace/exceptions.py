"""Typed exceptions for the Agent Workspace SDK."""


class WorkspaceError(Exception):
    """Generic fallback for 4xx/5xx errors."""

    def __init__(self, message: str, status_code: int | None = None) -> None:
        super().__init__(message)
        self.status_code = status_code


class AuthError(WorkspaceError):
    """401 — missing or invalid JWT."""


class ForbiddenError(WorkspaceError):
    """403 — JWT valid but missing required scope."""


class NotFoundError(WorkspaceError):
    """404 — agent, task, lock, or other resource not found."""


class LockConflictError(WorkspaceError):
    """409 — lock is already held by another agent/session."""


class TaskConflictError(WorkspaceError):
    """409 — task has already been claimed by another agent."""
