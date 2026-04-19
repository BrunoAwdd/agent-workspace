use std::sync::Arc;
use aw_domain::storage::WorkspaceStorage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn WorkspaceStorage>,
    /// HMAC-SHA256 secret for JWT validation. None = auth disabled (dev mode).
    pub jwt_secret: Option<String>,
}

impl AppState {
    pub fn new(storage: Arc<dyn WorkspaceStorage>) -> Self {
        Self { storage, jwt_secret: None }
    }

    pub fn with_jwt(mut self, secret: String) -> Self {
        self.jwt_secret = Some(secret);
        self
    }
}
