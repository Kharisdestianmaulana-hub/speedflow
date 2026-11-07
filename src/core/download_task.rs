use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Checking,
    Queued,
    Downloading,
    Paused,
    Retrying,
    Finalizing,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Checking => "checking",
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Retrying => "retrying",
            Self::Finalizing => "finalizing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: Uuid,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub connections: u32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DownloadTask {
    pub fn new(url: String, filename: String, destination_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            url,
            filename,
            destination_path,
            total_bytes: None,
            downloaded_bytes: 0,
            status: DownloadStatus::Pending,
            supports_range: false,
            etag: None,
            connections: 8,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn progress(&self) -> f32 {
        match self.total_bytes {
            Some(total) if total > 0 => (self.downloaded_bytes as f32 / total as f32).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }
}
