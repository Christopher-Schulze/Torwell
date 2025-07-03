use std::time::Duration;
use torwell84::session::SessionManager;

#[tokio::test]
async fn session_create_and_validate() {
    let mgr = SessionManager::new(Duration::from_millis(200));
    let token = mgr.create_session().await;
    assert!(mgr.validate(&token).await);
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(!mgr.validate(&token).await);
}
