/** @agent-workspace/sdk — public API */
export { WorkspaceClient, AgentSession } from "./client.js";
export {
  WorkspaceError,
  AuthError,
  ForbiddenError,
  NotFoundError,
  LockConflictError,
  TaskConflictError,
} from "./errors.js";
export type {
  Agent,
  AgentSessionData,
  Task,
  TaskStatus,
  TaskPriority,
  InboxItem,
  Message,
  Lock,
  LockType,
  Handoff,
  Event,
  WorkspaceSummary,
  SendMessageInput,
  CheckOutInput,
  CreateHandoffInput,
  RegisterInput,
  ListTasksInput,
  WorkspaceClientOptions,
} from "./types.js";
