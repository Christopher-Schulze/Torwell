use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;
use torwell84::tor_manager::{TorManager, TorClientBehavior, TorClientConfig};
use torwell84::error::Error;

#[derive(Clone, Default)]
struct MockTorClient {
    reconfigure_ok: bool,
    build_ok: bool,
}

static CONNECT_RESULTS: Lazy<Mutex<VecDeque<Result<MockTorClient, String>>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

impl MockTorClient {
    fn push_result(res: Result<MockTorClient, String>) {
        CONNECT_RESULTS.lock().unwrap().push_back(res);
    }
}

#[async_trait::async_trait]
impl TorClientBehavior for MockTorClient {
    async fn create_bootstrapped(_config: TorClientConfig) -> std::result::Result<Self, String> {
        CONNECT_RESULTS
            .lock()
            .unwrap()
            .pop_front()
            .expect("no result")
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
            Err("build".into())
        }
    }
}

#[tokio::test]
async fn connect_with_backoff_success() {
    MockTorClient::push_result(Err("fail".into()));
    MockTorClient::push_result(Ok(MockTorClient::default()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager.connect_with_backoff(5).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn connect_with_backoff_error() {
    MockTorClient::push_result(Err("e1".into()));
    MockTorClient::push_result(Err("e2".into()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    let res = manager.connect_with_backoff(1).await;
    assert!(matches!(res, Err(Error::Bootstrap(_))));
}

#[tokio::test]
async fn connect_when_already_connected() {
    MockTorClient::push_result(Ok(MockTorClient::default()));
    MockTorClient::push_result(Ok(MockTorClient::default()));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.connect_with_backoff(0).await;
    assert!(matches!(res, Err(Error::AlreadyConnected)));
}

#[tokio::test]
async fn new_identity_success() {
    MockTorClient::push_result(Ok(MockTorClient { reconfigure_ok: true, build_ok: true }));
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
    MockTorClient::push_result(Ok(MockTorClient { reconfigure_ok: false, build_ok: true }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.new_identity().await;
    assert!(matches!(res, Err(Error::Identity(_))));
}

#[tokio::test]
async fn new_identity_build_error() {
    MockTorClient::push_result(Ok(MockTorClient { reconfigure_ok: true, build_ok: false }));
    let manager: TorManager<MockTorClient> = TorManager::new();
    manager.connect().await.unwrap();
    let res = manager.new_identity().await;
    assert!(matches!(res, Err(Error::Circuit(_))));
}
