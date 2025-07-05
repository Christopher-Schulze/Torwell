use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;
use torwell84::error::Error;
use torwell84::tor_manager::{TorClientBehavior, TorClientConfig, TorManager};

#[derive(Clone, Default)]
struct MockTorClient {
    reconfigure_ok: bool,
    build_ok: bool,
    build_err: Option<String>,
}

static CONNECT_RESULTS: Lazy<Mutex<VecDeque<Result<MockTorClient, String>>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));
static CAPTURED_CONFIGS: Lazy<Mutex<Vec<TorClientConfig>>> = Lazy::new(|| Mutex::new(Vec::new()));

impl MockTorClient {
    fn push_result(res: Result<MockTorClient, String>) {
        CONNECT_RESULTS.lock().unwrap().push_back(res);
    }
}

#[async_trait::async_trait]
impl TorClientBehavior for MockTorClient {
    async fn create_bootstrapped(config: TorClientConfig) -> std::result::Result<Self, String> {
        CAPTURED_CONFIGS.lock().unwrap().push(config);
        CONNECT_RESULTS
            .lock()
            .unwrap()
            .pop_front()
            .expect("no result")
    }

    async fn create_bootstrapped_with_progress<P>(
        config: TorClientConfig,
        progress: &mut P,
    ) -> std::result::Result<Self, String>
    where
        P: FnMut(u8, String) + Send,
    {
        let res = Self::create_bootstrapped(config).await;
        if res.is_ok() {
            progress(100, "done".into());
        }
        res
    }

    fn reconfigure(&self, _config: &TorClientConfig) -> std::result::Result<(), String> {
        if self.reconfigure_ok {
            Ok(())
        } else {
            Err("reconf".into())
        }
    }

    fn retire_all_circs(&self) {}

    async fn build_new_circuit(&self) -> std::result::Result<(), String> {
        if self.build_ok {
            Ok(())
        } else {
            Err(self.build_err.clone().unwrap_or_else(|| "build".into()))
        }
    }
}

#[tokio::test]
async fn connect_with_backoff_success() {
    MockTorClient::push_result(Err("fail".into()));
    MockTorClient::push_result(Ok(MockTorClient::default()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager
        .connect_with_backoff(
            5,
            std::time::Duration::from_secs(10),
            |_a, _d, _e| {},
            |_| {},
        )
        .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn connect_with_backoff_error() {
    MockTorClient::push_result(Err("e1".into()));
    MockTorClient::push_result(Err("e2".into()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager
        .connect_with_backoff(
            1,
            std::time::Duration::from_secs(5),
            |_a, _d, _e| {},
            |_| {},
        )
        .await;
    assert!(matches!(res, Err(Error::RetriesExceeded { .. })));
}

#[tokio::test]
async fn connect_when_already_connected() {
    MockTorClient::push_result(Ok(MockTorClient::default()));
    MockTorClient::push_result(Ok(MockTorClient::default()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager
        .connect_with_backoff(
            0,
            std::time::Duration::from_secs(5),
            |_a, _d, _e| {},
            |_| {},
        )
        .await;
    assert!(matches!(res, Err(Error::AlreadyConnected)));
}

#[tokio::test]
async fn connect_with_backoff_timeout() {
    MockTorClient::push_result(Err("e1".into()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager
        .connect_with_backoff(
            5,
            std::time::Duration::from_secs(0),
            |_a, _d, _e| {},
            |_| {},
        )
        .await;
    assert!(matches!(res, Err(Error::Timeout)));
}

#[tokio::test]
async fn connect_error_simple() {
    MockTorClient::push_result(Err("boom".into()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager.connect().await;
    assert!(matches!(res, Err(Error::ConnectError(_))));
}

#[tokio::test]
async fn bridge_parse_error() {
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager
        .set_bridges(vec!["bad bridge".into()])
        .await
        .unwrap();
    let res = manager.connect().await;
    assert!(matches!(res, Err(Error::BridgeParse(_))));
}

#[tokio::test]
async fn lookup_country_error() {
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager.lookup_country_code("?.?.?.?").await;
    match res {
        Err(Error::Lookup(msg)) => assert!(msg.contains("?.?.?.?")),
        _ => panic!("expected lookup error"),
    }
}

#[tokio::test]
async fn new_identity_success() {
    MockTorClient::push_result(Ok(MockTorClient {
        reconfigure_ok: true,
        build_ok: true,
    }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    assert!(manager.new_identity().await.is_ok());
}

#[tokio::test]
async fn new_identity_not_connected() {
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager.new_identity().await;
    assert!(matches!(res, Err(Error::NotConnected)));
}

#[tokio::test]
async fn new_identity_reconfigure_error() {
    MockTorClient::push_result(Ok(MockTorClient {
        reconfigure_ok: false,
        build_ok: true,
    }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.new_identity().await;
    match res {
        Err(Error::Identity(msg)) => assert!(msg.contains("reconf")),
        _ => panic!("expected identity error"),
    }
}

#[tokio::test]
async fn new_identity_build_error() {
    MockTorClient::push_result(Ok(MockTorClient {
        reconfigure_ok: true,
        build_ok: false,
        build_err: None,
    }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.new_identity().await;
    match res {
        Err(Error::Circuit(msg)) => assert!(msg.contains("build")),
        _ => panic!("expected circuit error"),
    }
}

#[tokio::test]
async fn new_identity_timeout_error() {
    MockTorClient::push_result(Ok(MockTorClient {
        reconfigure_ok: true,
        build_ok: false,
        build_err: Some("timeout".into()),
    }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.new_identity().await;
    assert!(matches!(res, Err(Error::CircuitTimeout)));
}
