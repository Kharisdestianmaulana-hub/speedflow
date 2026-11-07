use std::sync::Arc;
use std::time::Duration;
use speedflow::services::{BridgeServer, InterceptedDownload};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_bridge_server_ping_and_intercept() {
    let (event_tx, mut event_rx) = mpsc::channel::<InterceptedDownload>(16);
    let port = 19762;
    let server = Arc::new(BridgeServer::new(port, event_tx));

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    // Wait a brief moment for socket to bind
    tokio::time::sleep(Duration::from_millis(150)).await;

    let client = reqwest::Client::new();

    // 1. Test Ping endpoint
    let ping_res = client
        .get(format!("http://127.0.0.1:{}/api/ping", port))
        .send()
        .await
        .unwrap();

    assert!(ping_res.status().is_success());
    let ping_body = ping_res.text().await.unwrap();
    assert!(ping_body.contains("online"));
    assert!(ping_body.contains("SpeedFlow"));

    // 2. Test Download Interception POST
    let download_payload = serde_json::json!({
        "url": "https://example.com/linux-distro.iso",
        "filename": "linux-distro.iso",
        "referrer": "https://distrowatch.com"
    });

    let intercept_res = client
        .post(format!("http://127.0.0.1:{}/api/download", port))
        .json(&download_payload)
        .send()
        .await
        .unwrap();

    assert!(intercept_res.status().is_success());

    // 3. Verify event channel received the intercepted data
    let received = tokio::time::timeout(Duration::from_secs(2), event_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.url, "https://example.com/linux-distro.iso");
    assert_eq!(received.filename.as_deref(), Some("linux-distro.iso"));
    assert_eq!(received.referrer.as_deref(), Some("https://distrowatch.com"));
}
