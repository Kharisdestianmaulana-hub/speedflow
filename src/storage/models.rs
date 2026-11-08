use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DownloadRow {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub total_bytes: Option<i64>,
    pub downloaded_bytes: i64,
    pub status: String,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub connections: i32,
    pub error_message: Option<String>,
    pub checksum: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SegmentRow {
    pub task_id: String,
    pub segment_index: i32,
    pub start_byte: i64,
    pub end_byte: i64,
    pub downloaded_bytes: i64,
    pub is_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub destination_path: String,
    pub max_connections: i32,
    pub max_simultaneous: i32,
    pub auto_retry: bool,
    #[serde(default = "default_true")]
    pub auto_categorize: bool,
    pub speed_limit_enabled: bool,
    pub speed_limit_kb: String,
    #[serde(default = "default_true")]
    pub is_dark: bool,
    #[serde(default)]
    pub onboarding_completed: bool,
}

fn default_true() -> bool {
    true
}
