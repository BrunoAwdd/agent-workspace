/** TypeScript interfaces for all Agent Workspace entities. */

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

export interface Agent {
  id: string;
  name: string;
  role: string;
  capabilities: string[];
  permissions: string[];
  createdAt?: string;
  updatedAt?: string;
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

export interface AgentSessionData {
  id: string;
  agentId: string;
  status: string;
  health?: string;
  currentTaskId?: string;
  metadata?: Record<string, unknown>;
  startedAt?: string;
  lastHeartbeatAt?: string;
  endedAt?: string;
}

// ---------------------------------------------------------------------------
// Message / Inbox
// ---------------------------------------------------------------------------

export interface Message {
  id?: string;
  workspaceId?: string;
  fromAgentId: string;
  toAgentId?: string;
  kind: string;
  payload?: Record<string, unknown>;
  deliverToInbox?: boolean;
  sentAt?: string;
}

export interface InboxItem {
  id: string;
  agentId: string;
  messageId?: string;
  status: string;
  message?: Message;
  createdAt?: string;
  updatedAt?: string;
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

export type TaskStatus =
  | "open"
  | "claimed"
  | "in_progress"
  | "done"
  | "failed"
  | "cancelled";
export type TaskPriority = "low" | "normal" | "high" | "critical";

export interface Task {
  id?: string;
  title: string;
  description?: string;
  kind: string;
  priority: TaskPriority;
  status: TaskStatus;
  assignedAgentId?: string;
  claimedBySessionId?: string;
  metadata?: Record<string, unknown>;
  createdAt?: string;
  updatedAt?: string;
}

// ---------------------------------------------------------------------------
// Lock
// ---------------------------------------------------------------------------

export type LockType =
  | "write_lock"
  | "soft_lock"
  | "topic_lock"
  | "artifact_lock"
  | "lease_lock";

export interface Lock {
  id?: string;
  scopeType: string;
  scopeId: string;
  lockType: LockType;
  ownerAgentId: string;
  ownerSessionId: string;
  ttlSecs: number;
  acquiredAt?: string;
  expiresAt?: string;
}

// ---------------------------------------------------------------------------
// Handoff
// ---------------------------------------------------------------------------

export interface Handoff {
  id?: string;
  fromAgentId: string;
  toAgentId?: string;
  sourceSessionId?: string;
  taskId?: string;
  summary?: string;
  payload?: Record<string, unknown>;
  createdAt?: string;
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

export interface Event {
  id?: string;
  kind: string;
  agentId?: string;
  sessionId?: string;
  taskId?: string;
  payload?: Record<string, unknown>;
  createdAt?: string;
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

export interface WorkspaceSummary {
  activeSessions: number;
  openTasks: number;
  pendingInbox: number;
  activeLocks: number;
  agents: number;
}

// ---------------------------------------------------------------------------
// Input types
// ---------------------------------------------------------------------------

export interface SendMessageInput {
  toAgentId: string;
  kind: string;
  payload?: Record<string, unknown>;
  deliverToInbox?: boolean;
  workspaceId?: string;
}

export interface CheckOutInput {
  createHandoff?: boolean;
  summary?: string;
  payload?: Record<string, unknown>;
}

export interface CreateHandoffInput {
  summary: string;
  toAgentId?: string;
  taskId?: string;
  payload?: Record<string, unknown>;
}

export interface RegisterInput {
  id?: string;
  name: string;
  role: string;
  capabilities?: string[];
  permissions?: string[];
}

export interface ListTasksInput {
  status?: TaskStatus[];
  unassignedOnly?: boolean;
  limit?: number;
}

export interface WorkspaceClientOptions {
  baseUrl?: string;
  token?: string;
  agentId: string;
  heartbeatIntervalSecs?: number;
  autoHeartbeat?: boolean;
}
