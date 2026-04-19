import type {
  CheckOutInput,
  CreateHandoffInput,
  Handoff,
  InboxItem,
  ListTasksInput,
  Lock,
  LockType,
  SendMessageInput,
  Task,
  TaskStatus,
} from "./types.js";
import {
  AuthError,
  ForbiddenError,
  LockConflictError,
  NotFoundError,
  TaskConflictError,
  WorkspaceError,
} from "./errors.js";

// Re-export so callers don't need a separate import
export {
  AuthError,
  ForbiddenError,
  LockConflictError,
  NotFoundError,
  TaskConflictError,
  WorkspaceError,
};

/** @internal */
export async function raiseForStatus(resp: Response): Promise<void> {
  if (resp.ok) return;
  let message = resp.statusText;
  try {
    const body = (await resp.json()) as { error?: string };
    if (body.error) message = body.error;
  } catch {
    // ignore parse errors
  }
  const code = resp.status;
  if (code === 401) throw new AuthError(message);
  if (code === 403) throw new ForbiddenError(message);
  if (code === 404) throw new NotFoundError(message);
  if (code === 409) {
    if (message.toLowerCase().includes("lock"))
      throw new LockConflictError(message);
    throw new TaskConflictError(message);
  }
  throw new WorkspaceError(message, code);
}

/**
 * Represents an active workspace session.
 *
 * Obtain via `WorkspaceClient.checkIn()` or use `client.withSession(fn)`
 * for automatic check-out.
 */
export class AgentSession {
  readonly id: string;
  readonly agentId: string;
  readonly inbox: InboxItem[];
  readonly pendingTasks: Task[];
  readonly pendingHandoffs: Handoff[];

  private readonly _baseUrl: string;
  private readonly _headers: Record<string, string>;
  private _heartbeatTimer: ReturnType<typeof setInterval> | null = null;

  constructor(opts: {
    id: string;
    agentId: string;
    inbox: InboxItem[];
    pendingTasks: Task[];
    pendingHandoffs: Handoff[];
    baseUrl: string;
    headers: Record<string, string>;
    heartbeatIntervalSecs: number;
    autoHeartbeat: boolean;
  }) {
    this.id = opts.id;
    this.agentId = opts.agentId;
    this.inbox = opts.inbox;
    this.pendingTasks = opts.pendingTasks;
    this.pendingHandoffs = opts.pendingHandoffs;
    this._baseUrl = opts.baseUrl;
    this._headers = opts.headers;

    if (opts.autoHeartbeat) {
      this._heartbeatTimer = setInterval(() => {
        this.heartbeat().catch((err) => {
          console.warn(
            `[agent-workspace] heartbeat failed (session ${this.id}):`,
            err,
          );
        });
      }, opts.heartbeatIntervalSecs * 1000);
    }
  }

  // ------------------------------------------------------------------
  // Internal HTTP helpers
  // ------------------------------------------------------------------

  private async _post<T>(path: string, body?: unknown): Promise<T> {
    const resp = await fetch(`${this._baseUrl}${path}`, {
      method: "POST",
      headers: this._headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    await raiseForStatus(resp);
    return resp.json() as Promise<T>;
  }

  private async _get<T>(
    path: string,
    params?: Record<string, string>,
  ): Promise<T> {
    const url = new URL(`${this._baseUrl}${path}`);
    if (params) {
      for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v);
    }
    const resp = await fetch(url.toString(), { headers: this._headers });
    await raiseForStatus(resp);
    return resp.json() as Promise<T>;
  }

  private async _delete<T>(path: string, body?: unknown): Promise<T | null> {
    const resp = await fetch(`${this._baseUrl}${path}`, {
      method: "DELETE",
      headers: this._headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    await raiseForStatus(resp);
    const text = await resp.text();
    return text ? (JSON.parse(text) as T) : null;
  }

  // ------------------------------------------------------------------
  // Heartbeat / checkout
  // ------------------------------------------------------------------

  async heartbeat(health = "healthy", currentTaskId?: string): Promise<void> {
    const body: Record<string, unknown> = { session_id: this.id, health };
    if (currentTaskId) body["current_task_id"] = currentTaskId;
    await this._post("/sessions/heartbeat", body);
  }

  private _stopHeartbeat(): void {
    if (this._heartbeatTimer !== null) {
      clearInterval(this._heartbeatTimer);
      this._heartbeatTimer = null;
    }
  }

  async checkOut(opts: CheckOutInput = {}): Promise<void> {
    this._stopHeartbeat();
    await this._post("/sessions/check-out", {
      session_id: this.id,
      create_handoff: opts.createHandoff ?? false,
      handoff_summary: opts.summary,
      handoff_payload: opts.payload,
    });
  }

  // ------------------------------------------------------------------
  // Messages
  // ------------------------------------------------------------------

  async sendMessage(input: SendMessageInput): Promise<void> {
    await this._post("/messages", {
      workspace_id: input.workspaceId ?? "main",
      from_agent_id: this.agentId,
      to_agent_id: input.toAgentId,
      kind: input.kind,
      payload: input.payload ?? {},
      deliver_to_inbox: input.deliverToInbox ?? true,
    });
  }

  // ------------------------------------------------------------------
  // Inbox
  // ------------------------------------------------------------------

  async listInbox(): Promise<InboxItem[]> {
    const data = await this._get<Record<string, unknown> | InboxItem[]>(
      `/inbox/${this.agentId}`,
    );
    const items = Array.isArray(data)
      ? data
      : ((data["items"] as InboxItem[]) ?? []);
    return items;
  }

  async ack(
    itemId: string,
    status: "done" | "failed" | "processing" = "done",
  ): Promise<void> {
    await this._post(`/inbox/${itemId}/ack`, {
      item_id: itemId,
      agent_id: this.agentId,
      status,
    });
  }

  // ------------------------------------------------------------------
  // Tasks
  // ------------------------------------------------------------------

  async createTask(
    title: string,
    description?: string,
    kind = "custom:default",
    priority = "normal",
    metadata?: Record<string, unknown>,
  ): Promise<Task> {
    return this._post<Task>("/tasks", {
      title,
      description,
      kind,
      priority,
      metadata: metadata ?? {},
    });
  }

  async claimTask(taskId: string): Promise<Task> {
    return this._post<Task>(`/tasks/${taskId}/claim`, {
      agent_id: this.agentId,
      session_id: this.id,
    });
  }

  async listTasks(opts: ListTasksInput = {}): Promise<Task[]> {
    const params: Record<string, string> = {};
    if (opts.status?.length) params["status"] = opts.status.join(",");
    if (opts.unassignedOnly) params["unassigned"] = "true";
    if (opts.limit !== undefined) params["limit"] = String(opts.limit);
    const data = await this._get<Record<string, unknown> | Task[]>(
      "/tasks",
      params,
    );
    const items = Array.isArray(data)
      ? data
      : ((data["items"] as Task[]) ?? []);
    return items;
  }

  async updateTaskStatus(
    taskId: string,
    status: TaskStatus,
    metadata?: Record<string, unknown>,
  ): Promise<Task> {
    return this._post<Task>(`/tasks/${taskId}/status`, {
      status,
      metadata: metadata ?? {},
    });
  }

  async assignTask(taskId: string, assignedTo: string | null): Promise<Task> {
    return this._post<Task>(`/tasks/${taskId}/assign`, {
      assigned_by: this.agentId,
      assigned_to: assignedTo,
    });
  }

  // ------------------------------------------------------------------
  // Locks
  // ------------------------------------------------------------------

  async acquireLock(
    scopeType: string,
    scopeId: string,
    lockType: LockType = "write_lock",
    ttlSecs = 300,
  ): Promise<Lock> {
    return this._post<Lock>("/locks", {
      scope_type: scopeType,
      scope_id: scopeId,
      lock_type: lockType,
      owner_agent_id: this.agentId,
      owner_session_id: this.id,
      ttl_secs: ttlSecs,
    });
  }

  async releaseLock(lockId: string): Promise<void> {
    await this._delete(`/locks/${lockId}`, {
      lock_id: lockId,
      owner_session_id: this.id,
    });
  }

  // ------------------------------------------------------------------
  // Handoffs
  // ------------------------------------------------------------------

  async createHandoff(input: CreateHandoffInput): Promise<Handoff> {
    return this._post<Handoff>("/handoffs", {
      from_agent_id: this.agentId,
      source_session_id: this.id,
      to_agent_id: input.toAgentId,
      task_id: input.taskId,
      summary: input.summary,
      payload: input.payload ?? {},
    });
  }

  async listHandoffs(): Promise<Handoff[]> {
    const data = await this._get<Record<string, unknown> | Handoff[]>(
      `/handoffs/${this.agentId}`,
    );
    const items = Array.isArray(data)
      ? data
      : ((data["items"] as Handoff[]) ?? []);
    return items;
  }
}
