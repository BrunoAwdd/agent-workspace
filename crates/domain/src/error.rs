use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("lock conflict: {0}")]
    LockConflict(String),

    #[error("session expired: {0}")]
    SessionExpired(String),

    #[error("precondition failed: {0}")]
    PreconditionFailed(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("storage error: {0}")]
    Storage(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;
