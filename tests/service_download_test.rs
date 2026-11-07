use std::sync::Arc;
use speedflow::app::DownloadService;
use speedflow::engine::SpeedLimiter;
use speedflow::storage::Database;
use speedflow::storage::models::DownloadRow;
use speedflow::storage::repository::DownloadRepository;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_service_lifecycle_and_history() {
    let temp_dir = std::env::temp_dir().join(format!("speedflow_test_{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await.unwrap();
    let db_path = temp_dir.join("test.db");

    let database = Arc::new(Database::init(Some(db_path)).await.unwrap());
    let speed_limiter = SpeedLimiter::new();
    let service = Arc::new(DownloadService::new(database.clone(), speed_limiter));

    // Initially downloads and queue should be empty
    let initial_downloads = service.get_downloads_ui().await;
    assert!(initial_downloads.is_empty());

    let initial_queue = service.get_queue_ui().await;
    assert!(initial_queue.is_empty());

    // Insert mock history records into SQLite
    let repo = DownloadRepository::new(database.pool().clone());
    let now = Utc::now().to_rfc3339();
    let row1 = DownloadRow {
        id: Uuid::new_v4().to_string(),
        url: "https://example.com/file1.zip".to_string(),
        filename: "file1.zip".to_string(),
        destination_path: temp_dir.join("file1.zip").to_string_lossy().to_string(),
        total_bytes: Some(10485760), // 10MB
        downloaded_bytes: 10485760,
        status: "completed".to_string(),
        supports_range: true,
        etag: Some("test_etag_1".to_string()),
        connections: 8,
        error_message: None,
        checksum: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    repo.insert_download(&row1).await.unwrap();

    let row2 = DownloadRow {
        id: Uuid::new_v4().to_string(),
        url: "https://example.com/archive.tar.gz".to_string(),
        filename: "archive.tar.gz".to_string(),
        destination_path: temp_dir.join("archive.tar.gz").to_string_lossy().to_string(),
        total_bytes: Some(5242880), // 5MB
        downloaded_bytes: 5242880,
        status: "completed".to_string(),
        supports_range: true,
        etag: Some("test_etag_2".to_string()),
        connections: 4,
        error_message: None,
        checksum: Some("abcdef1234567890".to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    repo.insert_download(&row2).await.unwrap();

    // Query history UI without filter
    let history = service.get_history_ui("").await;
    assert_eq!(history.len(), 2);

    // Query history with filter
    let filtered = service.get_history_ui("archive").await;
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].file_name, "archive.tar.gz");

    // Delete single history item
    service.delete_history_item(&row1.id).await.unwrap();
    let history_after_delete = service.get_history_ui("").await;
    assert_eq!(history_after_delete.len(), 1);

    // Clear history
    service.clear_history().await.unwrap();
    let history_empty = service.get_history_ui("").await;
    assert!(history_empty.is_empty());

    // Clean up temp directory
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_queue_management() {
    let temp_dir = std::env::temp_dir().join(format!("speedflow_test_q_{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await.unwrap();
    let db_path = temp_dir.join("test_q.db");

    let database = Arc::new(Database::init(Some(db_path)).await.unwrap());
    let speed_limiter = SpeedLimiter::new();
    let service = Arc::new(DownloadService::new(database.clone(), speed_limiter));

    // Manually push items into queue to test queue reordering & removal
    {
        let mut q = service.queue.write().await;
        q.push(speedflow::app::service::QueueItem {
            id: Uuid::new_v4(),
            url: "https://example.com/item1.bin".to_string(),
            destination_folder: temp_dir.clone(),
            filename: "item1.bin".to_string(),
            total_bytes: Some(1000),
            supports_range: true,
            etag: None,
            priority: "Normal".to_string(),
        });
        q.push(speedflow::app::service::QueueItem {
            id: Uuid::new_v4(),
            url: "https://example.com/item2.bin".to_string(),
            destination_folder: temp_dir.clone(),
            filename: "item2.bin".to_string(),
            total_bytes: Some(2000),
            supports_range: true,
            etag: None,
            priority: "High".to_string(),
        });
    }

    let q_items = service.get_queue_ui().await;
    assert_eq!(q_items.len(), 2);
    assert_eq!(q_items[0].file_name, "item1.bin");
    assert_eq!(q_items[1].file_name, "item2.bin");

    let id2 = Uuid::parse_str(q_items[1].id.as_str()).unwrap();
    // Move item2 up
    service.move_queue_up(id2).await;
    let q_items_moved = service.get_queue_ui().await;
    assert_eq!(q_items_moved[0].file_name, "item2.bin");
    assert_eq!(q_items_moved[1].file_name, "item1.bin");

    // Remove item2
    service.remove_from_queue(id2).await;
    let q_items_removed = service.get_queue_ui().await;
    assert_eq!(q_items_removed.len(), 1);
    assert_eq!(q_items_removed[0].file_name, "item1.bin");

    // Clear queue
    service.clear_queue().await;
    let q_empty = service.get_queue_ui().await;
    assert!(q_empty.is_empty());

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}

#[tokio::test]
async fn test_cancel_active_task_removes_from_ui_without_deadlock() {
    let temp_dir = std::env::temp_dir().join(format!("speedflow_test_cancel_{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&temp_dir).await.unwrap();
    let db_path = temp_dir.join("test_cancel.db");

    let database = Arc::new(Database::init(Some(db_path)).await.unwrap());
    let speed_limiter = SpeedLimiter::new();
    let service = Arc::new(DownloadService::new(database.clone(), speed_limiter));

    let task_id = Uuid::new_v4();
    let (pause_tx, _pause_rx) = tokio::sync::watch::channel(false);
    let active_task = Arc::new(speedflow::app::service::ActiveTask {
        id: task_id,
        url: "https://example.com/large.zip".to_string(),
        filename: "large.zip".to_string(),
        destination_path: temp_dir.join("large.zip"),
        total_bytes: Some(100_000_000),
        downloaded_bytes: Arc::new(std::sync::atomic::AtomicU64::new(14_800_000)),
        status: Arc::new(tokio::sync::RwLock::new(
            speedflow::core::download_task::DownloadStatus::Downloading,
        )),
        supports_range: true,
        etag: None,
        connections: 8,
        pause_tx: Arc::new(tokio::sync::RwLock::new(Some(pause_tx))),
        created_at: Utc::now(),
        checksum: Arc::new(tokio::sync::RwLock::new(None)),
        last_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        instant_speed: Arc::new(std::sync::atomic::AtomicU64::new(373_300)),
        error_message: Arc::new(tokio::sync::RwLock::new(None)),
    });

    {
        service.active_tasks.write().await.insert(task_id, active_task);
        service.task_order.write().await.push(task_id);
    }

    assert_eq!(service.get_downloads_ui().await.len(), 1);

    // Must complete promptly without deadlocking on active_tasks RwLock
    tokio::time::timeout(std::time::Duration::from_secs(2), service.cancel_task(task_id))
        .await
        .expect("cancel_task deadlocked!")
        .unwrap();

    assert!(service.get_downloads_ui().await.is_empty());

    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
