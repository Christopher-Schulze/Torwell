use crate::tor_manager::TorManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub tor_manager: Arc<TorManager>,
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tor_manager: Arc::new(TorManager::new()),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl AppState {
    pub async fn add_log(&self, message: String) {
        let mut logs = self.logs.lock().await;
        logs.push(message);
        // Optional: Limit log size to prevent memory issues
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
}