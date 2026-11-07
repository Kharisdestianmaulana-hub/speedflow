use crate::core::download_task::DownloadStatus;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    TaskCreated(Uuid),
    TaskStatusChanged { id: Uuid, status: DownloadStatus },
    TaskProgress {
        id: Uuid,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        speed_bps: u64,
        eta_seconds: Option<u64>,
    },
    TaskCompleted(Uuid),
    TaskFailed { id: Uuid, error: String },
}
