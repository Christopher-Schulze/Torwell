use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Manages short-lived session tokens.
pub struct SessionManager {
    sessions: Mutex<HashMap<String, Instant>>,
    ttl: Duration,
    startup_token: Mutex<Option<String>>,
}

impl SessionManager {
    /// Create a new session manager with the given time-to-live in seconds.
    /// A startup token is generated immediately and can be retrieved using
    /// [`take_startup_token`].
    pub fn new(ttl: Duration) -> Arc<Self> {
        let token = Self::generate_token();
        let expiry = Instant::now() + ttl;
        let mut map = HashMap::new();
        map.insert(token.clone(), expiry);
        let mgr = Arc::new(Self {
            sessions: Mutex::new(map),
            ttl,
            startup_token: Mutex::new(Some(token)),
        });
        mgr.clone().start_cleanup_task();
        mgr
    }

    fn generate_token() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
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

    /// Retrieve the startup token if it hasn't been taken yet.
    pub async fn take_startup_token(&self) -> Option<String> {
        self.startup_token.lock().await.take()
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

    /// Spawn a background task that periodically cleans up expired sessions.
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                self.cleanup().await;
            }
        });
    }
}
