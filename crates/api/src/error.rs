use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use aw_domain::error::WorkspaceError;
use serde_json::json;

pub struct ApiError(WorkspaceError);

impl From<WorkspaceError> for ApiError {
    fn from(e: WorkspaceError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            WorkspaceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            WorkspaceError::AlreadyExists(msg) => (StatusCode::CONFLICT, msg.clone()),
            WorkspaceError::LockConflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            WorkspaceError::SessionExpired(msg) => (StatusCode::GONE, msg.clone()),
            WorkspaceError::PreconditionFailed(msg) => (StatusCode::PRECONDITION_FAILED, msg.clone()),
            WorkspaceError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            WorkspaceError::Storage(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
