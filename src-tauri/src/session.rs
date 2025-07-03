use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Manages short-lived session tokens.
pub struct SessionManager {
    sessions: Mutex<HashMap<String, Instant>>,
    ttl: Duration,
}

impl SessionManager {
    /// Create a new session manager with the given time-to-live in seconds.
    pub fn new(ttl: Duration) -> Arc<Self> {
        Arc::new(Self {
            sessions: Mutex::new(HashMap::new()),
            ttl,
        })
    }

    /// Generate a new random session token and store it with an expiry time.
    pub async fn create_session(&self) -> String {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let expiry = Instant::now() + self.ttl;
        self.sessions.lock().await.insert(token.clone(), expiry);
        token
    }

    /// Validate a session token. Expired tokens are removed.
    pub async fn validate(&self, token: &str) -> bool {
        self.cleanup().await;
        self.sessions
            .lock()
            .await
            .get(token)
            .map(|&exp| exp > Instant::now())
            .unwrap_or(false)
    }

    /// Remove expired sessions.
    async fn cleanup(&self) {
        let now = Instant::now();
        self.sessions.lock().await.retain(|_, &mut exp| exp > now);
    }
}
