use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;
use torwell84::error::Error;
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

#[derive(Clone)]
struct MockStats {
    sent: u64,
    received: u64,
}

impl MockStats {
    fn bytes_written(&self) -> u64 {
        self.sent
    }
    fn bytes_read(&self) -> u64 {
        self.received
    }
}

#[derive(Clone)]
struct MockMetricsClient {
    stats: MockStats,
}

impl MockMetricsClient {
    fn new(sent: u64, received: u64) -> Self {
        Self {
            stats: MockStats { sent, received },
        }
    }

    fn push_client(client: MockMetricsClient) {
        CONNECT_RESULTS.lock().unwrap().push_back(client);
    }

    fn traffic_stats(&self) -> MockStats {
        self.stats.clone()
    }
}

static CONNECT_RESULTS: Lazy<Mutex<VecDeque<MockMetricsClient>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

#[async_trait::async_trait]
impl TorClientBehavior for MockMetricsClient {
    async fn create_bootstrapped(_c: TorClientConfig) -> std::result::Result<Self, String> {
        Ok(CONNECT_RESULTS
            .lock()
            .unwrap()
            .pop_front()
            .expect("no mock client"))
    }

    async fn create_bootstrapped_with_progress<P>(
        c: TorClientConfig,
        progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        let client = Self::create_bootstrapped(c).await?;
        progress(100, "done".into());
        Ok(client)
    }

    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        Ok(())
    }

    fn retire_all_circs(&self) {}

    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        Ok(())
    }
}

#[tokio::test]
async fn traffic_stats_not_connected() {
    let manager: TorManager<MockMetricsClient> = TorManager::new();
    let res = manager.traffic_stats().await;
    assert!(matches!(res, Err(Error::NotConnected)));
}

#[tokio::test]
async fn traffic_stats_success() {
    MockMetricsClient::push_client(MockMetricsClient::new(10, 20));
    let manager: TorManager<MockMetricsClient> = TorManager::new();
    manager.connect().await.unwrap();
    let stats = manager.traffic_stats().await.unwrap();
    assert_eq!(stats.bytes_sent, 10);
    assert_eq!(stats.bytes_received, 20);
}

#[tokio::test]
async fn circuit_metrics_not_connected() {
    let manager: TorManager<MockMetricsClient> = TorManager::new();
    let res = manager.circuit_metrics().await;
    assert!(matches!(res, Err(Error::NotConnected)));
}

#[tokio::test]
async fn circuit_metrics_connected() {
    MockMetricsClient::push_client(MockMetricsClient::new(0, 0));
    let manager: TorManager<MockMetricsClient> = TorManager::new();
    manager.connect().await.unwrap();
    let metrics = manager.circuit_metrics().await.unwrap();
    assert_eq!(metrics.count, 0);
    assert_eq!(metrics.oldest_age, 0);
    assert_eq!(metrics.avg_create_ms, 0);
    assert_eq!(metrics.failed_attempts, 0);
    assert!(!metrics.complete);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_metrics_benchmark() {
    MockMetricsClient::push_client(MockMetricsClient::new(0, 0));
    let manager: TorManager<MockMetricsClient> = TorManager::new();
    manager.connect().await.unwrap();

    use sysinfo::{PidExt, System, SystemExt};

    let mut sys = System::new();
    let pid = sysinfo::get_current_pid().unwrap();
    sys.refresh_process(pid);
    let start_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

    let start = std::time::Instant::now();
    let mut handles = Vec::new();
    for _ in 0..8 {
        let m = manager.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..50 {
                let _ = m.traffic_stats().await;
                let _ = m.circuit_metrics().await;
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    let elapsed = start.elapsed();

    sys.refresh_process(pid);
    let end_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

    println!(
        "parallel metrics: {:?}, mem diff: {} KB",
        elapsed,
        end_mem.saturating_sub(start_mem)
    );
}
