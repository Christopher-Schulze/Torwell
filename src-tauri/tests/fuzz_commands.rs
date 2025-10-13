use rand::{distributions::Alphanumeric, Rng};
use std::path::PathBuf;
use std::sync::Arc;
use torwell84::commands;
use torwell84::error::Error;
use torwell84::secure_http::SecureHttpClient;
use torwell84::session::SessionManager;
use torwell84::state::AppState;
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

#[derive(Clone, Default)]
struct MockTorClient;

#[async_trait::async_trait]
impl TorClientBehavior for MockTorClient {
    async fn create_bootstrapped(_cfg: TorClientConfig) -> std::result::Result<Self, String> {
        Ok(Self)
    }
    async fn create_bootstrapped_with_progress<P>(
        _cfg: TorClientConfig,
        _progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        Ok(Self)
    }
    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        Ok(())
    }
    fn retire_all_circs(&self) {}
    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        Ok(())
    }
}

fn mock_state() -> AppState<MockTorClient> {
    AppState {
        tor_manager: Arc::new(TorManager::new()),
        http_client: Arc::new(SecureHttpClient::new_default().unwrap()),
        log_file: PathBuf::from("fuzz.log"),
        log_lock: Arc::new(Mutex::new(())),
        retry_counter: Arc::new(Mutex::new(0)),
        max_log_lines: Arc::new(Mutex::new(1000)),
        memory_usage: Arc::new(Mutex::new(0)),
        circuit_count: Arc::new(Mutex::new(0)),
        latency_ms: Arc::new(Mutex::new(0)),
        cpu_usage: Arc::new(Mutex::new(0.0)),
        network_throughput: Arc::new(Mutex::new(0)),
        prev_traffic: Arc::new(Mutex::new(0)),
        max_memory_mb: 1024,
        max_circuits: 20,
        session: SessionManager::new(std::time::Duration::from_secs(60)),
        app_handle: Arc::new(Mutex::new(None)),
        tray_warning: Arc::new(Mutex::new(None)),
    }
}

#[tokio::test]
async fn fuzz_network_commands() {
    let mut app = tauri::test::mock_app();
    let state = mock_state();
    app.manage(state);
    let state = app.state::<AppState<MockTorClient>>();
    let token = state.create_session().await;
    let mut rng = rand::thread_rng();

    let mut rate_limited = false;

    for _ in 0..100 {
        match rng.gen_range(0..6) {
            // ping_host with random host and count
            0 => {
                let len = rng.gen_range(1..16);
                let host: String = (&mut rng)
                    .sample_iter(&Alphanumeric)
                    .take(len)
                    .map(char::from)
                    .collect();
                let count = rng.gen_range(0..15);
                let res = commands::ping_host(state, token.clone(), Some(host), Some(count)).await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
            // set_exit_country with random code
            1 => {
                let country: String = (&mut rng)
                    .sample_iter(&Alphanumeric)
                    .take(2)
                    .map(char::from)
                    .collect();
                let res = commands::set_exit_country(state, Some(country)).await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
            // get_active_circuit
            2 => {
                let res = commands::get_active_circuit(state).await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
            // get_circuit_policy_report
            3 => {
                let res = commands::get_circuit_policy_report(state).await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
            4 => {
                let include_bridges = rng.gen_bool(0.5);
                let fast_only = rng.gen_bool(0.5);
                let extra: Option<Vec<String>> = if rng.gen_bool(0.5) {
                    Some(
                        (0..rng.gen_range(0..3))
                            .map(|_| {
                                let len = rng.gen_range(1..4);
                                (&mut rng)
                                    .sample_iter(&Alphanumeric)
                                    .take(len)
                                    .map(char::from)
                                    .collect::<String>()
                            })
                            .collect(),
                    )
                } else {
                    None
                };
                let res =
                    commands::generate_torrc_profile(state, fast_only, extra, include_bridges)
                        .await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
            // get_metrics
            _ => {
                let res = commands::get_metrics(state).await;
                if matches!(res, Err(Error::RateLimitExceeded(_))) {
                    rate_limited = true;
                }
            }
        }
    }

    assert!(rate_limited);
}
