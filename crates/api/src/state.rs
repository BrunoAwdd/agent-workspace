use std::sync::Arc;
use aw_domain::storage::WorkspaceStorage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn WorkspaceStorage>,
}

impl AppState {
    pub fn new(storage: Arc<dyn WorkspaceStorage>) -> Self {
        Self { storage }
    }
}
