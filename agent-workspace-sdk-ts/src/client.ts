import { AgentSession, raiseForStatus } from "./session.js";
import type {
  Agent,
  AgentSessionData,
  Event,
  Handoff,
  InboxItem,
  RegisterInput,
  Task,
  WorkspaceClientOptions,
  WorkspaceSummary,
} from "./types.js";

export { AgentSession } from "./session.js";
export * from "./errors.js";
export type * from "./types.js";

/**
 * Main entry point for the Agent Workspace SDK.
 *
 * ```ts
 * const client = new WorkspaceClient({ agentId: "my-agent" });
 * await client.withSession(async (session) => {
 *   const task = await session.claimTask("uuid");
 *   await session.updateTaskStatus(task.id!, "done");
 * });
 * ```
 */
export class WorkspaceClient {
  private readonly _baseUrl: string;
  private readonly _agentId: string;
  private readonly _headers: Record<string, string>;
  private readonly _heartbeatIntervalSecs: number;
  private readonly _autoHeartbeat: boolean;

  constructor(opts: WorkspaceClientOptions) {
    this._baseUrl = (opts.baseUrl ?? "http://localhost:4000").replace(
      /\/$/,
      "",
    );
    this._agentId = opts.agentId;
    this._heartbeatIntervalSecs = opts.heartbeatIntervalSecs ?? 45;
    this._autoHeartbeat = opts.autoHeartbeat ?? true;

    this._headers = { "Content-Type": "application/json" };
    if (opts.token) {
      this._headers["Authorization"] = `Bearer ${opts.token}`;
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

  private _items<T>(data: Record<string, unknown> | T[]): T[] {
    if (Array.isArray(data)) return data;
    return (data["items"] as T[]) ?? [];
  }

  // ------------------------------------------------------------------
  // Agent registration
  // ------------------------------------------------------------------

  /** Register (or update) an agent. Idempotent — safe to call on every startup. */
  async register(input: RegisterInput): Promise<Agent> {
    return this._post<Agent>("/agents", {
      id: input.id ?? this._agentId,
      name: input.name,
      role: input.role,
      capabilities: input.capabilities ?? [],
      permissions: input.permissions ?? [],
    });
  }

  // ------------------------------------------------------------------
  // Session lifecycle
  // ------------------------------------------------------------------

  /** Check in and return an AgentSession with inbox/tasks/handoffs pre-loaded. */
  async checkIn(): Promise<AgentSession> {
    const raw = await this._post<AgentSessionData>("/sessions/check-in", {
      agent_id: this._agentId,
    });

    // Enrich in parallel — failures are silently ignored
    const [inboxRaw, tasksRaw, handoffsRaw] = await Promise.allSettled([
      this._get<Record<string, unknown> | InboxItem[]>(
        `/inbox/${this._agentId}`,
      ),
      this._get<Record<string, unknown> | Task[]>("/tasks", {
        assigned_to: this._agentId,
        status: "open,claimed",
      }),
      this._get<Record<string, unknown> | Handoff[]>(
        `/handoffs/${this._agentId}`,
      ),
    ]);

    const inbox: InboxItem[] =
      inboxRaw.status === "fulfilled"
        ? this._items<InboxItem>(inboxRaw.value)
        : [];
    const pendingTasks: Task[] =
      tasksRaw.status === "fulfilled" ? this._items<Task>(tasksRaw.value) : [];
    const pendingHandoffs: Handoff[] =
      handoffsRaw.status === "fulfilled"
        ? this._items<Handoff>(handoffsRaw.value)
        : [];

    return new AgentSession({
      id: raw.id,
      agentId: this._agentId,
      inbox,
      pendingTasks,
      pendingHandoffs,
      baseUrl: this._baseUrl,
      headers: this._headers,
      heartbeatIntervalSecs: this._heartbeatIntervalSecs,
      autoHeartbeat: this._autoHeartbeat,
    });
  }

  /**
   * Run a callback with an active session.
   * Check-out is called automatically after the callback (even on error).
   */
  async withSession<T>(fn: (session: AgentSession) => Promise<T>): Promise<T> {
    const session = await this.checkIn();
    try {
      return await fn(session);
    } finally {
      await session.checkOut().catch((err) => {
        console.warn("[agent-workspace] check-out failed:", err);
      });
    }
  }

  // ------------------------------------------------------------------
  // Workspace-level (no session required)
  // ------------------------------------------------------------------

  async getSummary(): Promise<WorkspaceSummary> {
    return this._get<WorkspaceSummary>("/summary");
  }

  async listEvents(agentId?: string, limit = 50): Promise<Event[]> {
    const params: Record<string, string> = { limit: String(limit) };
    if (agentId) params["agent_id"] = agentId;
    const data = await this._get<Record<string, unknown> | Event[]>(
      "/events",
      params,
    );
    return this._items<Event>(data);
  }

  async listAgents(): Promise<Agent[]> {
    const data = await this._get<Record<string, unknown> | Agent[]>("/agents");
    return this._items<Agent>(data);
  }
}
