/** Typed error classes for the Agent Workspace SDK. */

export class WorkspaceError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
  ) {
    super(message);
    this.name = "WorkspaceError";
  }
}

export class AuthError extends WorkspaceError {
  constructor(message: string) {
    super(message, 401);
    this.name = "AuthError";
  }
}

export class ForbiddenError extends WorkspaceError {
  constructor(message: string) {
    super(message, 403);
    this.name = "ForbiddenError";
  }
}

export class NotFoundError extends WorkspaceError {
  constructor(message: string) {
    super(message, 404);
    this.name = "NotFoundError";
  }
}

export class LockConflictError extends WorkspaceError {
  constructor(message: string) {
    super(message, 409);
    this.name = "LockConflictError";
  }
}

export class TaskConflictError extends WorkspaceError {
  constructor(message: string) {
    super(message, 409);
    this.name = "TaskConflictError";
  }
}
