"""Pydantic v2 models for all Agent Workspace entities."""

from __future__ import annotations

from datetime import datetime
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field


# ---------------------------------------------------------------------------
# Agent
# ---------------------------------------------------------------------------


class Agent(BaseModel):
    id: str
    name: str
    role: str
    capabilities: list[str] = Field(default_factory=list)
    permissions: list[str] = Field(default_factory=list)
    created_at: datetime | None = None
    updated_at: datetime | None = None


# ---------------------------------------------------------------------------
# Session
# ---------------------------------------------------------------------------


class AgentSessionData(BaseModel):
    id: UUID
    agent_id: str
    status: str
    health: str | None = None
    current_task_id: UUID | None = None
    metadata: dict[str, Any] | None = Field(default_factory=dict)
    started_at: datetime | None = None
    last_heartbeat_at: datetime | None = None
    ended_at: datetime | None = None


# ---------------------------------------------------------------------------
# Message / Inbox
# ---------------------------------------------------------------------------


class Message(BaseModel):
    id: UUID | None = None
    workspace_id: str | None = None
    from_agent_id: str
    to_agent_id: str | None = None
    kind: str
    payload: dict[str, Any] | None = Field(default_factory=dict)
    deliver_to_inbox: bool = False
    sent_at: datetime | None = None


class InboxItem(BaseModel):
    id: UUID
    target_agent_id: str
    source_agent_id: str | None = None
    message_id: UUID | None = None
    status: str
    message: Message | None = None
    created_at: datetime | None = None
    updated_at: datetime | None = None


# ---------------------------------------------------------------------------
# Task
# ---------------------------------------------------------------------------


class Task(BaseModel):
    id: UUID | None = None
    title: str
    description: str | None = None
    kind: Any
    priority: str = "normal"
    status: str = "open"
    assigned_agent_id: str | None = None
    claimed_by_session_id: UUID | None = None
    metadata: dict[str, Any] | None = Field(default_factory=dict)
    created_at: datetime | None = None
    updated_at: datetime | None = None


# ---------------------------------------------------------------------------
# Lock
# ---------------------------------------------------------------------------


class Lock(BaseModel):
    id: UUID | None = None
    scope_type: str
    scope_id: str
    lock_type: str
    owner_agent_id: str
    owner_session_id: UUID
    ttl_secs: int = 300
    acquired_at: datetime | None = None
    expires_at: datetime | None = None


# ---------------------------------------------------------------------------
# Handoff
# ---------------------------------------------------------------------------


class Handoff(BaseModel):
    id: UUID | None = None
    from_agent_id: str
    to_agent_id: str | None = None
    source_session_id: UUID | None = None
    task_id: UUID | None = None
    summary: str | None = None
    payload: dict[str, Any] | None = Field(default_factory=dict)
    created_at: datetime | None = None


# ---------------------------------------------------------------------------
# Dependency
# ---------------------------------------------------------------------------


class Dependency(BaseModel):
    key: str
    state: str
    details: str | None = None
    checked_at: datetime | None = None
    updated_at: datetime | None = None


# ---------------------------------------------------------------------------
# Event
# ---------------------------------------------------------------------------


class Event(BaseModel):
    id: UUID | None = None
    kind: str
    agent_id: str | None = None
    session_id: UUID | None = None
    task_id: UUID | None = None
    payload: dict[str, Any] | None = Field(default_factory=dict)
    created_at: datetime | None = None


# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------


class WorkspaceSummary(BaseModel):
    active_sessions: int = 0
    open_tasks: int = 0
    pending_inbox: int = 0
    active_locks: int = 0
    agents: int = 0
