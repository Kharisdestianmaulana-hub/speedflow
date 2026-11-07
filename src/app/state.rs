use std::sync::Arc;

use crate::app::service::DownloadService;
use crate::core::DownloadManager;
use crate::engine::SpeedLimiter;
use crate::storage::Database;

#[derive(Clone)]
pub struct AppState {
    pub download_manager: Arc<DownloadManager>,
    pub download_service: Arc<DownloadService>,
    pub database: Arc<Database>,
    pub speed_limiter: SpeedLimiter,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database = Arc::new(Database::init(None).await?);
        let download_manager = Arc::new(DownloadManager::new(3));
        let speed_limiter = SpeedLimiter::new();
        let download_service = Arc::new(DownloadService::new(
            database.clone(),
            speed_limiter.clone(),
        ));
        download_service.restore_from_database().await;

        Ok(Self {
            download_manager,
            download_service,
            database,
            speed_limiter,
        })
    }
}
