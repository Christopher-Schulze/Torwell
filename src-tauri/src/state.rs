use crate::tor_manager::TorManager;
use std::sync::Arc;

pub struct AppState {
    pub tor_manager: Arc<TorManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tor_manager: Arc::new(TorManager::new()),
        }
    }
}