use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use slint::SharedString;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{watch, Mutex, RwLock};
use tracing::{error, info};
use uuid::Uuid;

use crate::core::download_task::DownloadStatus;
use crate::engine::http::HttpEngine;
use crate::engine::segments::calculate_segments;
use crate::engine::SpeedLimiter;
use crate::platform::PlatformSystem;
use crate::services::{ChecksumVerifier, FileCategory, NotificationService};
use crate::storage::models::DownloadRow;
use crate::storage::repository::DownloadRepository;
use crate::storage::Database;
use crate::ui::{DownloadItemData, HistoryItemData, QueueItemData};

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.1} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_speed(bps: u64) -> String {
    if bps == 0 {
        "0 B/s".to_string()
    } else {
        format!("{}/s", format_bytes(bps))
    }
}

pub fn format_eta(seconds: u64) -> String {
    if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

pub fn get_unique_filepath(dir: &Path, filename: &str) -> PathBuf {
    let mut candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_name = format!("{} ({}){}", stem, counter, ext);
        candidate = dir.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub struct ActiveTask {
    pub id: Uuid,
    pub url: Arc<RwLock<String>>,
    pub filename: String,
    pub destination_path: PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: Arc<AtomicU64>,
    pub status: Arc<RwLock<DownloadStatus>>,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub connections: u32,
    pub pause_tx: Arc<RwLock<Option<watch::Sender<bool>>>>,
    pub job_lock: Arc<Mutex<()>>,
    pub created_at: DateTime<Utc>,
    pub checksum: Arc<RwLock<Option<String>>>,
    pub last_bytes: Arc<AtomicU64>,
    pub instant_speed: Arc<AtomicU64>,
    pub error_message: Arc<RwLock<Option<String>>>,
}

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub id: Uuid,
    pub url: String,
    pub destination_folder: PathBuf,
    pub filename: String,
    pub total_bytes: Option<u64>,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub priority: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynSegStatus {
    Pending,
    Active,
    Completed,
}

struct DynSegment {
    index: u32,
    start_byte: u64,
    current_len: Arc<AtomicU64>,
    end_byte: Arc<AtomicU64>,
    status: DynSegStatus,
}

pub struct DownloadService {
    pub database: Arc<Database>,
    pub speed_limiter: SpeedLimiter,
    pub http_engine: Arc<HttpEngine>,
    pub active_tasks: Arc<RwLock<HashMap<Uuid, Arc<ActiveTask>>>>,
    pub task_order: Arc<RwLock<Vec<Uuid>>>,
    pub queue: Arc<RwLock<Vec<QueueItem>>>,
    pub queue_paused: Arc<AtomicBool>,
    pub max_simultaneous: Arc<AtomicUsize>,
    pub max_connections: Arc<AtomicU32>,
    pub auto_categorize: Arc<AtomicBool>,
    pub dynamic_layouts: Arc<RwLock<HashMap<Uuid, Vec<(u32, u64, u64)>>>>,
    pub active_streams: Arc<RwLock<HashMap<Uuid, Arc<AtomicUsize>>>>,
    segment_pools: Arc<RwLock<HashMap<Uuid, Arc<Mutex<Vec<DynSegment>>>>>>,
}

impl DownloadService {
    pub fn new(database: Arc<Database>, speed_limiter: SpeedLimiter) -> Self {
        Self {
            database,
            speed_limiter,
            http_engine: Arc::new(HttpEngine::new()),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_order: Arc::new(RwLock::new(Vec::new())),
            queue: Arc::new(RwLock::new(Vec::new())),
            queue_paused: Arc::new(AtomicBool::new(false)),
            max_simultaneous: Arc::new(AtomicUsize::new(3)),
            max_connections: Arc::new(AtomicU32::new(16)),
            auto_categorize: Arc::new(AtomicBool::new(true)),
            dynamic_layouts: Arc::new(RwLock::new(HashMap::new())),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            segment_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn restore_from_database(&self) {
        let repo = DownloadRepository::new(self.database.pool().clone());
        let Ok(rows) = repo.get_all_downloads().await else {
            return;
        };

        // get_all_downloads is ordered DESC, so reverse to restore chronological order
        for row in rows.into_iter().rev() {
            let Ok(task_id) = Uuid::parse_str(&row.id) else {
                continue;
            };

            let dest_path = PathBuf::from(&row.destination_path);
            let created_at = DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            if row.status == "queued" {
                let dest_folder = dest_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."));
                self.queue.write().await.push(QueueItem {
                    id: task_id,
                    url: row.url,
                    destination_folder: dest_folder,
                    filename: row.filename,
                    total_bytes: row.total_bytes.map(|b| b as u64),
                    supports_range: row.supports_range,
                    etag: row.etag,
                    priority: "Normal".to_string(),
                });
                continue;
            }

            if row.status == "completed" {
                if !dest_path.exists() {
                    continue;
                }
                let dl_bytes = row.downloaded_bytes.max(0) as u64;
                let active_task = Arc::new(ActiveTask {
                    id: task_id,
                    url: Arc::new(RwLock::new(row.url)),
                    filename: row.filename,
                    destination_path: dest_path,
                    total_bytes: row.total_bytes.map(|b| b as u64),
                    downloaded_bytes: Arc::new(AtomicU64::new(dl_bytes)),
                    status: Arc::new(RwLock::new(DownloadStatus::Completed)),
                    supports_range: row.supports_range,
                    etag: row.etag,
                    connections: row.connections.max(1) as u32,
                    pause_tx: Arc::new(RwLock::new(None)),
                    job_lock: Arc::new(Mutex::new(())),
                    created_at,
                    checksum: Arc::new(RwLock::new(row.checksum)),
                    last_bytes: Arc::new(AtomicU64::new(dl_bytes)),
                    instant_speed: Arc::new(AtomicU64::new(0)),
                    error_message: Arc::new(RwLock::new(None)),
                });
                self.active_tasks.write().await.insert(task_id, active_task);
                self.task_order.write().await.push(task_id);
                continue;
            }

            if row.status == "downloading" || row.status == "paused" || row.status == "failed" {
                // Any interrupted "downloading" task becomes Paused on startup so user can Resume
                let restored_status = if row.status == "failed" {
                    DownloadStatus::Failed
                } else {
                    if row.status == "downloading" {
                        let _ = repo
                            .update_status(&row.id, "paused", &Utc::now().to_rfc3339())
                            .await;
                    }
                    DownloadStatus::Paused
                };

                // Load saved dynamic segments if present
                let saved_segs = repo.load_dynamic_segments(&row.id).await.unwrap_or_default();
                let mut disk_bytes = 0u64;
                if !saved_segs.is_empty() {
                    let mut restored_dyn_segs = Vec::with_capacity(saved_segs.len());
                    for &(idx, start_b, end_b) in &saved_segs {
                        let part_path =
                            PathBuf::from(format!("{}.sf_part{}", dest_path.display(), idx));
                        let max_len = (end_b - start_b) + 1;
                        let len = fs::metadata(&part_path)
                            .await
                            .map(|m| m.len())
                            .unwrap_or(0)
                            .min(max_len);
                        disk_bytes += len;
                        let st = if len >= max_len {
                            DynSegStatus::Completed
                        } else {
                            DynSegStatus::Pending
                        };
                        restored_dyn_segs.push(DynSegment {
                            index: idx,
                            start_byte: start_b,
                            current_len: Arc::new(AtomicU64::new(len)),
                            end_byte: Arc::new(AtomicU64::new(end_b)),
                            status: st,
                        });
                    }
                    self.dynamic_layouts.write().await.insert(task_id, saved_segs);
                    self.segment_pools
                        .write()
                        .await
                        .insert(task_id, Arc::new(Mutex::new(restored_dyn_segs)));
                } else {
                    for i in 0..32 {
                        let part_path =
                            PathBuf::from(format!("{}.sf_part{}", dest_path.display(), i));
                        if let Ok(meta) = fs::metadata(&part_path).await {
                            disk_bytes += meta.len();
                        }
                    }
                }

                let dl_bytes = if disk_bytes > 0 {
                    disk_bytes
                } else {
                    row.downloaded_bytes.max(0) as u64
                };

                let active_task = Arc::new(ActiveTask {
                    id: task_id,
                    url: Arc::new(RwLock::new(row.url)),
                    filename: row.filename,
                    destination_path: dest_path,
                    total_bytes: row.total_bytes.map(|b| b as u64),
                    downloaded_bytes: Arc::new(AtomicU64::new(dl_bytes)),
                    status: Arc::new(RwLock::new(restored_status)),
                    supports_range: row.supports_range,
                    etag: row.etag,
                    connections: row.connections.max(1) as u32,
                    pause_tx: Arc::new(RwLock::new(None)),
                    job_lock: Arc::new(Mutex::new(())),
                    created_at,
                    checksum: Arc::new(RwLock::new(row.checksum)),
                    last_bytes: Arc::new(AtomicU64::new(dl_bytes)),
                    instant_speed: Arc::new(AtomicU64::new(0)),
                    error_message: Arc::new(RwLock::new(row.error_message)),
                });

                self.active_tasks.write().await.insert(task_id, active_task);
                self.task_order.write().await.push(task_id);
            }
        }
    }

    pub async fn start_download(
        self: &Arc<Self>,
        url: String,
        destination_folder: Option<PathBuf>,
    ) -> anyhow::Result<Uuid> {
        self.start_download_full(url, None, destination_folder).await
    }

    pub async fn start_download_full(
        self: &Arc<Self>,
        url: String,
        custom_filename: Option<String>,
        destination_folder: Option<PathBuf>,
    ) -> anyhow::Result<Uuid> {
        let trimmed_url = url.trim().to_string();
        if trimmed_url.is_empty() {
            anyhow::bail!("Download URL cannot be empty");
        }

        info!("Inspecting URL: {}", trimmed_url);
        let info = self.http_engine.inspect_url(&trimmed_url).await?;
        let download_url = info.url.clone();

        let dest_dir = match destination_folder {
            Some(dir) => dir,
            None => {
                if let Some(user_dirs) = directories::UserDirs::new() {
                    user_dirs
                        .download_dir()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."))
                } else {
                    PathBuf::from(".")
                }
            }
        };

        let base_filename = match custom_filename {
            Some(ref name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => info.filename.clone(),
        };

        let dest_dir = FileCategory::resolve_category_folder(
            &dest_dir,
            &base_filename,
            self.auto_categorize.load(Ordering::Relaxed),
        );
        fs::create_dir_all(&dest_dir).await?;

        let target_file_path = get_unique_filepath(&dest_dir, &base_filename);
        let final_filename = target_file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&base_filename)
            .to_string();

        let task_id = Uuid::new_v4();
        let now = Utc::now();
        let max_conns = self.max_connections.load(Ordering::Relaxed);
        let max_sim = self.max_simultaneous.load(Ordering::Relaxed);
        let active_count = self.get_active_downloading_count().await;

        if active_count >= max_sim {
            info!("Max simultaneous downloads ({}) reached. Adding to Queue.", max_sim);

            let queue_item = QueueItem {
                id: task_id,
                url: download_url.clone(),
                destination_folder: dest_dir,
                filename: final_filename.clone(),
                total_bytes: info.content_length,
                supports_range: info.supports_range,
                etag: info.etag.clone(),
                priority: "Normal".to_string(),
            };

            let row = DownloadRow {
                id: task_id.to_string(),
                url: download_url,
                filename: final_filename,
                destination_path: target_file_path.to_string_lossy().to_string(),
                total_bytes: info.content_length.map(|b| b as i64),
                downloaded_bytes: 0,
                status: "queued".to_string(),
                supports_range: info.supports_range,
                etag: info.etag,
                connections: max_conns as i32,
                error_message: None,
                checksum: None,
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            };

            let repo = DownloadRepository::new(self.database.pool().clone());
            let _ = repo.insert_download(&row).await;

            NotificationService::notify_download_queued(&row.filename);

            let mut q = self.queue.write().await;
            q.push(queue_item);

            return Ok(task_id);
        }

        let (pause_tx, pause_rx) = watch::channel(false);
        let active_task = Arc::new(ActiveTask {
            id: task_id,
            url: Arc::new(RwLock::new(download_url.clone())),
            filename: final_filename.clone(),
            destination_path: target_file_path.clone(),
            total_bytes: info.content_length,
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
            status: Arc::new(RwLock::new(DownloadStatus::Downloading)),
            supports_range: info.supports_range,
            etag: info.etag.clone(),
            connections: max_conns,
            pause_tx: Arc::new(RwLock::new(Some(pause_tx))),
            job_lock: Arc::new(Mutex::new(())),
            created_at: now,
            checksum: Arc::new(RwLock::new(None)),
            last_bytes: Arc::new(AtomicU64::new(0)),
            instant_speed: Arc::new(AtomicU64::new(0)),
            error_message: Arc::new(RwLock::new(None)),
        });

        let row = DownloadRow {
            id: task_id.to_string(),
            url: download_url,
            filename: final_filename,
            destination_path: target_file_path.to_string_lossy().to_string(),
            total_bytes: info.content_length.map(|b| b as i64),
            downloaded_bytes: 0,
            status: "downloading".to_string(),
            supports_range: info.supports_range,
            etag: info.etag,
            connections: max_conns as i32,
            error_message: None,
            checksum: None,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        };

        let repo = DownloadRepository::new(self.database.pool().clone());
        let _ = repo.insert_download(&row).await;

        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(task_id, active_task.clone());
            let mut order = self.task_order.write().await;
            order.push(task_id);
        }

        NotificationService::notify_download_starting(&active_task.filename);

        let svc = self.clone();
        tokio::spawn(async move {
            svc.run_download_job(active_task, pause_rx).await;
        });

        Ok(task_id)
    }

    pub async fn queue_download_full(
        self: &Arc<Self>,
        url: String,
        custom_filename: Option<String>,
        destination_folder: Option<PathBuf>,
    ) -> anyhow::Result<Uuid> {
        let trimmed_url = url.trim().to_string();
        if trimmed_url.is_empty() {
            anyhow::bail!("Download URL cannot be empty");
        }

        info!("Inspecting URL for Queue: {}", trimmed_url);
        let info = self.http_engine.inspect_url(&trimmed_url).await?;

        let dest_dir = match destination_folder {
            Some(dir) => dir,
            None => {
                if let Some(user_dirs) = directories::UserDirs::new() {
                    user_dirs
                        .download_dir()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."))
                } else {
                    PathBuf::from(".")
                }
            }
        };

        let base_filename = match custom_filename {
            Some(ref name) if !name.trim().is_empty() => name.trim().to_string(),
            _ => info.filename.clone(),
        };

        let dest_dir = FileCategory::resolve_category_folder(
            &dest_dir,
            &base_filename,
            self.auto_categorize.load(Ordering::Relaxed),
        );
        fs::create_dir_all(&dest_dir).await?;

        let target_file_path = get_unique_filepath(&dest_dir, &base_filename);
        let final_filename = target_file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&base_filename)
            .to_string();

        let task_id = Uuid::new_v4();
        let now = Utc::now();
        let max_conns = self.max_connections.load(Ordering::Relaxed);

        let queue_item = QueueItem {
            id: task_id,
            url: trimmed_url.clone(),
            destination_folder: dest_dir,
            filename: final_filename.clone(),
            total_bytes: info.content_length,
            supports_range: info.supports_range,
            etag: info.etag.clone(),
            priority: "Normal".to_string(),
        };

        let row = DownloadRow {
            id: task_id.to_string(),
            url: trimmed_url,
            filename: final_filename,
            destination_path: target_file_path.to_string_lossy().to_string(),
            total_bytes: info.content_length.map(|b| b as i64),
            downloaded_bytes: 0,
            status: "queued".to_string(),
            supports_range: info.supports_range,
            etag: info.etag,
            connections: max_conns as i32,
            error_message: None,
            checksum: None,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
        };

        let repo = DownloadRepository::new(self.database.pool().clone());
        let _ = repo.insert_download(&row).await;

        NotificationService::notify_download_queued(&row.filename);

        let mut q = self.queue.write().await;
        q.push(queue_item);

        Ok(task_id)
    }

    async fn get_active_downloading_count(&self) -> usize {
        let tasks: Vec<Arc<ActiveTask>> = {
            let guard = self.active_tasks.read().await;
            guard.values().cloned().collect()
        };
        let mut count = 0;
        for task in tasks {
            if *task.status.read().await == DownloadStatus::Downloading {
                count += 1;
            }
        }
        count
    }

    async fn cleanup_temp_files(destination_path: &Path) {
        for i in 0..128 {
            let part_path = format!("{}.sf_part{}", destination_path.display(), i);
            let _ = fs::remove_file(part_path).await;
        }
        let single_part = format!("{}.sf_download", destination_path.display());
        let _ = fs::remove_file(single_part).await;
    }

    async fn run_download_job(
        self: Arc<Self>,
        task: Arc<ActiveTask>,
        pause_rx: watch::Receiver<bool>,
    ) {
        let _job_guard = task.job_lock.lock().await;
        if *pause_rx.borrow() || *task.status.read().await == DownloadStatus::Cancelled {
            return;
        }

        info!("Starting download worker for: {}", task.filename);

        let job_res = self.run_download_job_inner(task.clone(), pause_rx).await;
        self.active_streams.write().await.remove(&task.id);

        // If the task was cancelled while running, clean up partial files after handles close and exit
        if *task.status.read().await == DownloadStatus::Cancelled {
            info!("Task {} cancelled by user.", task.filename);
            self.dynamic_layouts.write().await.remove(&task.id);
            self.segment_pools.write().await.remove(&task.id);
            Self::cleanup_temp_files(&task.destination_path).await;
            return;
        }

        match job_res {
            Ok(is_paused) => {
                if is_paused {
                    info!("Task {} paused by user.", task.filename);
                    let total_dl = task.downloaded_bytes.load(Ordering::Relaxed);
                    let now_str = Utc::now().to_rfc3339();
                    let repo = DownloadRepository::new(self.database.pool().clone());

                    // Only set status to Paused if the user hasn't already clicked Resume in the meantime
                    if task.pause_tx.read().await.is_none() {
                        *task.status.write().await = DownloadStatus::Paused;
                        let _ = repo
                            .update_status(&task.id.to_string(), "paused", &now_str)
                            .await;
                    }
                    let _ = repo
                        .update_progress(&task.id.to_string(), total_dl as i64, &now_str)
                        .await;
                    if let Some(segs) = self.dynamic_layouts.read().await.get(&task.id).cloned() {
                        let _ = repo
                            .save_dynamic_segments(&task.id.to_string(), &segs)
                            .await;
                    }
                } else {
                    info!("Download completed successfully: {}", task.filename);
                    self.dynamic_layouts.write().await.remove(&task.id);
                    self.segment_pools.write().await.remove(&task.id);
                    *task.status.write().await = DownloadStatus::Completed;

                    let sha = ChecksumVerifier::compute_sha256(&task.destination_path)
                        .await
                        .ok();
                    *task.checksum.write().await = sha.clone();

                    let total_dl = task.downloaded_bytes.load(Ordering::Relaxed);
                    let repo = DownloadRepository::new(self.database.pool().clone());
                    let now_str = Utc::now().to_rfc3339();
                    let _ = repo
                        .update_completed(
                            &task.id.to_string(),
                            total_dl as i64,
                            sha.as_deref(),
                            &now_str,
                        )
                        .await;
                    let _ = repo.save_dynamic_segments(&task.id.to_string(), &[]).await;

                    NotificationService::notify_download_completed(&task.filename);

                    let svc = self.clone();
                    tokio::spawn(async move {
                        svc.check_queue_and_dispatch().await;
                    });
                }
            }
            Err(e) => {
                error!("Download failed for {}: {:?}", task.filename, e);
                *task.error_message.write().await = Some(e.to_string());
                *task.status.write().await = DownloadStatus::Failed;

                NotificationService::notify_download_failed(&task.filename);

                let total_dl = task.downloaded_bytes.load(Ordering::Relaxed);
                let repo = DownloadRepository::new(self.database.pool().clone());
                let now_str = Utc::now().to_rfc3339();
                let _ = repo
                    .update_status(&task.id.to_string(), "failed", &now_str)
                    .await;
                let _ = repo
                    .update_progress(&task.id.to_string(), total_dl as i64, &now_str)
                    .await;
                if let Some(segs) = self.dynamic_layouts.read().await.get(&task.id).cloned() {
                    let _ = repo
                        .save_dynamic_segments(&task.id.to_string(), &segs)
                        .await;
                }

                let svc = self.clone();
                tokio::spawn(async move {
                    svc.check_queue_and_dispatch().await;
                });
            }
        }
    }

    async fn run_download_job_inner(
        &self,
        task: Arc<ActiveTask>,
        pause_rx: watch::Receiver<bool>,
    ) -> anyhow::Result<bool> {
        let total_bytes = task.total_bytes;
        let supports_range = task.supports_range;
        let connections = task.connections;

        if supports_range && total_bytes.is_some() && total_bytes.unwrap() > 0 {
            let total = total_bytes.unwrap();
            let num_workers = if total < 512 * 1024 {
                1
            } else {
                connections.clamp(1, 32)
            };

            // Load existing dynamic segment layout (if resuming after Pause) or create initial segments
            let initial_tuples: Vec<(u32, u64, u64)> = {
                let layouts = self.dynamic_layouts.read().await;
                if let Some(existing) = layouts.get(&task.id) {
                    existing.clone()
                } else {
                    drop(layouts);
                    let calc = calculate_segments(total, num_workers);
                    let tuples: Vec<(u32, u64, u64)> = calc
                        .into_iter()
                        .map(|s| (s.index, s.start_byte, s.end_byte))
                        .collect();
                    self.dynamic_layouts
                        .write()
                        .await
                        .insert(task.id, tuples.clone());
                    tuples
                }
            };

            let mut dyn_segments = Vec::with_capacity(initial_tuples.len());
            let mut resumed_bytes_sum = 0u64;

            for (idx, start_b, end_b) in initial_tuples {
                let part_path = PathBuf::from(format!(
                    "{}.sf_part{}",
                    task.destination_path.display(),
                    idx
                ));
                let seg_max_len = (end_b - start_b) + 1;
                let existing_len = fs::metadata(&part_path)
                    .await
                    .map(|m| m.len())
                    .unwrap_or(0)
                    .min(seg_max_len);
                resumed_bytes_sum += existing_len;

                let status = if existing_len >= seg_max_len {
                    DynSegStatus::Completed
                } else {
                    DynSegStatus::Pending
                };

                dyn_segments.push(DynSegment {
                    index: idx,
                    start_byte: start_b,
                    current_len: Arc::new(AtomicU64::new(existing_len)),
                    end_byte: Arc::new(AtomicU64::new(end_b)),
                    status,
                });
            }

            task.downloaded_bytes
                .store(resumed_bytes_sum, Ordering::Release);
            task.last_bytes
                .store(resumed_bytes_sum, Ordering::Release);

            info!(
                "SpeedFlow Engine: Downloading {} with up to {} dynamic multi-socket connections (Total: {} bytes)",
                task.filename, num_workers, total
            );

            let pool = Arc::new(Mutex::new(dyn_segments));
            self.segment_pools
                .write()
                .await
                .insert(task.id, pool.clone());
            let active_streams = Arc::new(AtomicUsize::new(0));
            self.active_streams
                .write()
                .await
                .insert(task.id, active_streams.clone());
            let mut worker_handles = Vec::new();
            const MIN_SPLIT_BYTES: u64 = 1024 * 1024; // 1 MB minimum remaining size to split in half
            let task_url_str = task.url.read().await.clone();

            for worker_id in 0..num_workers {
                let pool_ref = pool.clone();
                let active_streams_ref = active_streams.clone();
                let http_engine = self.http_engine.clone();
                let url = task_url_str.clone();
                let dest_base = task.destination_path.clone();
                let downloaded_atomic = task.downloaded_bytes.clone();
                let speed_limiter = self.speed_limiter.clone();
                let mut p_rx = pause_rx.clone();

                let handle = tokio::spawn(async move {
                    // Smooth connection ramp-up (350ms stagger per connection) so CDNs don't trigger 503 burst protection
                    if worker_id > 0 {
                        let ramp_delay = Duration::from_millis((worker_id as u64) * 350);
                        tokio::select! {
                            _ = tokio::time::sleep(ramp_delay) => {}
                            _ = p_rx.changed() => {
                                if *p_rx.borrow() {
                                    return Ok::<(), anyhow::Error>(());
                                }
                            }
                        }
                    }

                    let mut consecutive_errors = 0u32;

                    loop {
                        if *p_rx.borrow() {
                            return Ok(());
                        }

                        // Step 1: Claim a Pending segment OR dynamically split the largest Active segment in half!
                        let claimed: Option<(u32, u64, Arc<AtomicU64>, Arc<AtomicU64>)> = {
                            let mut guard = pool_ref.lock().await;

                            // 1a. Refresh any Completed statuses first
                            for s in guard.iter_mut() {
                                let cur = s.current_len.load(Ordering::Acquire);
                                let end = s.end_byte.load(Ordering::Acquire);
                                if s.start_byte + cur > end {
                                    s.status = DynSegStatus::Completed;
                                }
                            }

                            // 1b. Look for a Pending segment
                            if let Some(seg) = guard
                                .iter_mut()
                                .find(|s| s.status == DynSegStatus::Pending)
                            {
                                seg.status = DynSegStatus::Active;
                                Some((
                                    seg.index,
                                    seg.start_byte,
                                    seg.current_len.clone(),
                                    seg.end_byte.clone(),
                                ))
                            } else if guard.iter().all(|s| s.status == DynSegStatus::Completed) {
                                // All segments finished!
                                None
                            } else {
                                // 1c. In-Flight Dynamic Segment Splitting (Work Stealing):
                                // Find the Active segment with the largest remaining bytes
                                let mut best_idx: Option<usize> = None;
                                let mut best_rem: u64 = 0;

                                for (i, s) in guard.iter().enumerate() {
                                    if s.status == DynSegStatus::Active {
                                        let pos =
                                            s.start_byte + s.current_len.load(Ordering::Acquire);
                                        let end = s.end_byte.load(Ordering::Acquire);
                                        if end >= pos {
                                            let rem = (end - pos) + 1;
                                            if rem > best_rem {
                                                best_rem = rem;
                                                best_idx = Some(i);
                                            }
                                        }
                                    }
                                }

                                if let Some(target_i) = best_idx {
                                    if best_rem >= MIN_SPLIT_BYTES * 2 && guard.len() < 120 {
                                        let pos = guard[target_i].start_byte
                                            + guard[target_i]
                                                .current_len
                                                .load(Ordering::Acquire);
                                        let old_end =
                                            guard[target_i].end_byte.load(Ordering::Acquire);
                                        let split_start = pos + (best_rem / 2);

                                        if split_start > pos && split_start <= old_end {
                                            // Shrink the active segment's end_byte atomically
                                            guard[target_i]
                                                .end_byte
                                                .store(split_start - 1, Ordering::Release);

                                            let new_seg_idx = guard.len() as u32;
                                            let new_cur = Arc::new(AtomicU64::new(0));
                                            let new_end = Arc::new(AtomicU64::new(old_end));

                                            guard.push(DynSegment {
                                                index: new_seg_idx,
                                                start_byte: split_start,
                                                current_len: new_cur.clone(),
                                                end_byte: new_end.clone(),
                                                status: DynSegStatus::Active,
                                            });

                                            Some((new_seg_idx, split_start, new_cur, new_end))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        };

                        let Some((seg_idx, seg_start, cur_len_atomic, end_byte_atomic)) = claimed
                        else {
                            // Check if all segments are Completed or if we should wait briefly for active segments
                            let all_done = {
                                let guard = pool_ref.lock().await;
                                guard.iter().all(|s| s.status == DynSegStatus::Completed)
                            };
                            if all_done {
                                return Ok(());
                            }
                            tokio::select! {
                                _ = tokio::time::sleep(Duration::from_millis(200)) => continue,
                                _ = p_rx.changed() => return Ok(()),
                            }
                        };

                        let part_path =
                            PathBuf::from(format!("{}.sf_part{}", dest_base.display(), seg_idx));
                        let existing_len =
                            fs::metadata(&part_path).await.map(|m| m.len()).unwrap_or(0);
                        cur_len_atomic.store(existing_len, Ordering::Release);

                        let req_end = end_byte_atomic.load(Ordering::Acquire);
                        if seg_start + existing_len > req_end {
                            let mut guard = pool_ref.lock().await;
                            if let Some(s) = guard.iter_mut().find(|s| s.index == seg_idx) {
                                s.status = DynSegStatus::Completed;
                            }
                            continue;
                        }

                        let range_start = seg_start + existing_len;
                        let range_header = format!("bytes={}-{}", range_start, req_end);

                        let send_res = tokio::select! {
                            res = http_engine.build_get(&url).header("Range", range_header).send() => res,
                            _ = p_rx.changed() => {
                                let mut guard = pool_ref.lock().await;
                                if let Some(s) = guard.iter_mut().find(|s| s.index == seg_idx) {
                                    s.status = DynSegStatus::Pending;
                                }
                                return Ok(());
                            }
                        };

                        let response = match send_res {
                            Ok(resp) => resp,
                            Err(e) => {
                                {
                                    let mut guard = pool_ref.lock().await;
                                    if let Some(s) = guard.iter_mut().find(|s| s.index == seg_idx)
                                    {
                                        s.status = DynSegStatus::Pending;
                                    }
                                }
                                consecutive_errors += 1;
                                if consecutive_errors > 10 {
                                    return Err(e.into());
                                }
                                let backoff = Duration::from_millis(
                                    800 * (consecutive_errors.min(6) as u64),
                                );
                                tokio::select! {
                                    _ = tokio::time::sleep(backoff) => continue,
                                    _ = p_rx.changed() => return Ok(()),
                                }
                            }
                        };

                        let status = response.status();
                        if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
                            {
                                let mut guard = pool_ref.lock().await;
                                if let Some(s) = guard.iter_mut().find(|s| s.index == seg_idx) {
                                    s.status = DynSegStatus::Pending;
                                }
                            }

                            // If server caps concurrent connections (503 / 429):
                            // If other streams are already active, exit this excess worker immediately
                            // so we don't hammer the server's rate-limiter and slow down the active streams!
                            if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                                || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                            {
                                let active_now = active_streams_ref.load(Ordering::Relaxed);
                                if active_now > 0 {
                                    info!(
                                        "Server enforced max connection limit ({} active streams, HTTP {}). Parking excess worker #{}.",
                                        active_now,
                                        status.as_u16(),
                                        worker_id
                                    );
                                    return Ok(());
                                }
                                consecutive_errors += 1;
                                let backoff = Duration::from_millis(
                                    1500 * (consecutive_errors.min(8) as u64),
                                );
                                tokio::select! {
                                    _ = tokio::time::sleep(backoff) => continue,
                                    _ = p_rx.changed() => return Ok(()),
                                }
                            }

                            consecutive_errors += 1;
                            if consecutive_errors > 8 {
                                anyhow::bail!("Segment request returned status: {}", status);
                            }
                            tokio::select! {
                                _ = tokio::time::sleep(Duration::from_millis(1000)) => continue,
                                _ = p_rx.changed() => return Ok(()),
                            }
                        }

                        consecutive_errors = 0;
                        let new_active = active_streams_ref.fetch_add(1, Ordering::Relaxed) + 1;
                        info!(
                            "Segment #{} connected (Worker #{}, Active parallel streams: {})",
                            seg_idx, worker_id, new_active
                        );

                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&part_path)
                            .await?;

                        // 512 KB RAM write buffer per segment so disk I/O never stalls TCP socket recv
                        let mut writer = BufWriter::with_capacity(512 * 1024, file);
                        let mut stream = response.bytes_stream();
                        let mut was_paused = false;
                        let mut stream_failed = false;

                        loop {
                            if *p_rx.borrow() {
                                was_paused = true;
                                break;
                            }

                            let live_end = end_byte_atomic.load(Ordering::Acquire);
                            let max_seg_len = (live_end - seg_start) + 1;
                            let cur_len = cur_len_atomic.load(Ordering::Acquire);

                            if cur_len >= max_seg_len {
                                // Segment reached its (possibly dynamically split) end boundary!
                                break;
                            }

                            let next_item = tokio::select! {
                                item = stream.next() => item,
                                _ = p_rx.changed() => {
                                    if *p_rx.borrow() {
                                        was_paused = true;
                                        break;
                                    }
                                    continue;
                                }
                            };

                            let Some(chunk_res) = next_item else {
                                break;
                            };

                            match chunk_res {
                                Ok(chunk) => {
                                    let live_end = end_byte_atomic.load(Ordering::Acquire);
                                    let max_seg_len = (live_end - seg_start) + 1;
                                    let cur_len = cur_len_atomic.load(Ordering::Acquire);
                                    if cur_len >= max_seg_len {
                                        break;
                                    }

                                    let remaining_allowed = (max_seg_len - cur_len) as usize;
                                    let write_len = chunk.len().min(remaining_allowed);

                                    if write_len > 0 {
                                        writer.write_all(&chunk[..write_len]).await?;
                                        cur_len_atomic
                                            .fetch_add(write_len as u64, Ordering::Release);
                                        downloaded_atomic
                                            .fetch_add(write_len as u64, Ordering::Relaxed);

                                        if speed_limiter.is_enabled() {
                                            speed_limiter
                                                .throttle(write_len, Duration::from_millis(10))
                                                .await;
                                        }
                                    }

                                    if cur_len + (write_len as u64) >= max_seg_len {
                                        break;
                                    }
                                }
                                Err(_) => {
                                    stream_failed = true;
                                    break;
                                }
                            }
                        }

                        let _ = writer.flush().await;
                        drop(writer);
                        active_streams_ref.fetch_sub(1, Ordering::Relaxed);

                        // Update segment status in pool
                        {
                            let mut guard = pool_ref.lock().await;
                            if let Some(s) = guard.iter_mut().find(|s| s.index == seg_idx) {
                                let cur = s.current_len.load(Ordering::Acquire);
                                let end = s.end_byte.load(Ordering::Acquire);
                                if s.start_byte + cur > end {
                                    s.status = DynSegStatus::Completed;
                                } else {
                                    s.status = DynSegStatus::Pending;
                                }
                            }
                        }

                        if was_paused {
                            return Ok(());
                        }

                        if stream_failed {
                            tokio::select! {
                                _ = tokio::time::sleep(Duration::from_millis(600)) => continue,
                                _ = p_rx.changed() => return Ok(()),
                            }
                        }
                    }
                });

                worker_handles.push(handle);
            }

            let mut last_err: Option<anyhow::Error> = None;
            for h in worker_handles {
                match h.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        error!("Segment worker error: {:?}", e);
                        last_err = Some(e);
                    }
                    Err(e) => {
                        error!("Task join error: {:?}", e);
                        last_err = Some(e.into());
                    }
                }
            }

            // Persist current dynamic segment layout so Pause & Resume preserves any split segments
            let (all_completed, mut final_segments) = {
                let guard = pool.lock().await;
                let mut tuples: Vec<(u32, u64, u64)> = guard
                    .iter()
                    .map(|s| (s.index, s.start_byte, s.end_byte.load(Ordering::Acquire)))
                    .collect();
                tuples.sort_by_key(|t| t.1);
                let done = guard.iter().all(|s| {
                    let cur = s.current_len.load(Ordering::Acquire);
                    let end = s.end_byte.load(Ordering::Acquire);
                    s.start_byte + cur > end
                });
                (done, tuples)
            };

            self.dynamic_layouts
                .write()
                .await
                .insert(task.id, final_segments.clone());

            if *pause_rx.borrow() {
                return Ok(true);
            }

            if all_completed {
                info!(
                    "Merging {} dynamic segments into final file: {:?}",
                    final_segments.len(),
                    task.destination_path
                );
                *task.status.write().await = DownloadStatus::Finalizing;

                let final_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&task.destination_path)
                    .await?;
                let mut final_writer = BufWriter::with_capacity(4 * 1024 * 1024, final_file);

                final_segments.sort_by_key(|s| s.1);
                for (seg_idx, seg_start, seg_end) in final_segments {
                    let expected_len = (seg_end - seg_start) + 1;
                    let part_path = PathBuf::from(format!(
                        "{}.sf_part{}",
                        task.destination_path.display(),
                        seg_idx
                    ));
                    if let Ok(part_file) = File::open(&part_path).await {
                        let mut reader =
                            BufReader::with_capacity(4 * 1024 * 1024, part_file).take(expected_len);
                        tokio::io::copy(&mut reader, &mut final_writer).await?;
                    }
                    let _ = fs::remove_file(&part_path).await;
                }
                final_writer.flush().await?;
                Ok(false)
            } else if let Some(e) = last_err {
                Err(e)
            } else {
                anyhow::bail!("One or more segments failed to finish completely");
            }
        } else {
            // Fallback: Single-stream download with browser session headers & 512 KB BufWriter
            info!("Single-stream download for {}", task.filename);
            let active_streams = Arc::new(AtomicUsize::new(1));
            self.active_streams
                .write()
                .await
                .insert(task.id, active_streams);
            let temp_dest =
                PathBuf::from(format!("{}.sf_download", task.destination_path.display()));
            let mut p_rx = pause_rx.clone();

            let task_url_str = task.url.read().await.clone();
            let response = tokio::select! {
                res = self.http_engine.build_get(&task_url_str).send() => res?,
                _ = p_rx.changed() => {
                    return Ok(true);
                }
            };
            if !response.status().is_success() {
                anyhow::bail!("Download failed with HTTP status: {}", response.status());
            }

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_dest)
                .await?;
            let mut writer = BufWriter::with_capacity(512 * 1024, file);

            let mut stream = response.bytes_stream();
            let mut was_paused = false;

            loop {
                if *p_rx.borrow() {
                    was_paused = true;
                    break;
                }

                let next_item = tokio::select! {
                    item = stream.next() => item,
                    _ = p_rx.changed() => {
                        if *p_rx.borrow() {
                            was_paused = true;
                            break;
                        }
                        continue;
                    }
                };

                let Some(chunk_res) = next_item else {
                    break;
                };

                let chunk = chunk_res?;
                let len = chunk.len();
                writer.write_all(&chunk).await?;
                task.downloaded_bytes
                    .fetch_add(len as u64, Ordering::Relaxed);

                if self.speed_limiter.is_enabled() {
                    self.speed_limiter
                        .throttle(len, Duration::from_millis(10))
                        .await;
                }
            }

            writer.flush().await?;
            drop(writer);

            if was_paused {
                return Ok(true);
            }

            let _ = fs::rename(&temp_dest, &task.destination_path).await;
            Ok(false)
        }
    }

    pub async fn pause_task(&self, id: Uuid) -> anyhow::Result<()> {
        let task = {
            let tasks = self.active_tasks.read().await;
            tasks.get(&id).cloned()
        };
        if let Some(task) = task {
            info!("Pausing task: {}", task.filename);
            *task.status.write().await = DownloadStatus::Paused;
            task.instant_speed.store(0, Ordering::Relaxed);
            self.active_streams.write().await.remove(&id);
            if let Some(tx) = task.pause_tx.write().await.take() {
                let _ = tx.send(true);
            }
            let pool = self.database.pool().clone();
            let task_id_str = task.id.to_string();
            tokio::spawn(async move {
                let repo = DownloadRepository::new(pool);
                let _ = repo
                    .update_status(&task_id_str, "paused", &Utc::now().to_rfc3339())
                    .await;
            });
        }
        Ok(())
    }

    pub async fn resume_task(self: &Arc<Self>, id: Uuid) -> anyhow::Result<()> {
        let task = {
            let tasks = self.active_tasks.read().await;
            tasks.get(&id).cloned()
        };
        if let Some(task) = task {
            if *task.status.read().await != DownloadStatus::Paused {
                return Ok(());
            }

            info!("Resuming task: {}", task.filename);
            let (tx, rx) = watch::channel(false);
            *task.pause_tx.write().await = Some(tx);
            *task.status.write().await = DownloadStatus::Downloading;

            let pool = self.database.pool().clone();
            let task_id_str = task.id.to_string();
            tokio::spawn(async move {
                let repo = DownloadRepository::new(pool);
                let _ = repo
                    .update_status(&task_id_str, "downloading", &Utc::now().to_rfc3339())
                    .await;
            });

            let svc = self.clone();
            let task_clone = task.clone();
            tokio::spawn(async move {
                svc.run_download_job(task_clone, rx).await;
            });
        }
        Ok(())
    }

    pub async fn pause_all_tasks(&self) {
        let ids: Vec<Uuid> = {
            let order = self.task_order.read().await;
            order.clone()
        };
        for id in ids {
            let is_downloading = {
                let tasks = self.active_tasks.read().await;
                if let Some(t) = tasks.get(&id) {
                    *t.status.read().await == DownloadStatus::Downloading
                } else {
                    false
                }
            };
            if is_downloading {
                let _ = self.pause_task(id).await;
            }
        }
    }

    pub async fn resume_all_tasks(self: &Arc<Self>) {
        let ids: Vec<Uuid> = {
            let order = self.task_order.read().await;
            order.clone()
        };
        for id in ids {
            let is_paused = {
                let tasks = self.active_tasks.read().await;
                if let Some(t) = tasks.get(&id) {
                    *t.status.read().await == DownloadStatus::Paused
                } else {
                    false
                }
            };
            if is_paused {
                let _ = self.resume_task(id).await;
            }
        }
    }

    pub async fn retry_task(self: &Arc<Self>, id: Uuid) -> anyhow::Result<()> {
        let task = {
            let tasks = self.active_tasks.read().await;
            tasks.get(&id).cloned()
        };
        if let Some(task) = task {
            let status = *task.status.read().await;
            if status != DownloadStatus::Failed && status != DownloadStatus::Paused {
                return Ok(());
            }

            info!("Retrying task: {}", task.filename);
            let (tx, rx) = watch::channel(false);
            *task.pause_tx.write().await = Some(tx);
            *task.error_message.write().await = None;
            *task.status.write().await = DownloadStatus::Downloading;

            let pool = self.database.pool().clone();
            let task_id_str = task.id.to_string();
            tokio::spawn(async move {
                let repo = DownloadRepository::new(pool);
                let _ = repo
                    .update_status(&task_id_str, "downloading", &Utc::now().to_rfc3339())
                    .await;
            });

            let svc = self.clone();
            let task_clone = task.clone();
            tokio::spawn(async move {
                svc.run_download_job(task_clone, rx).await;
            });
        }
        Ok(())
    }

    pub async fn update_task_url_and_resume(
        self: &Arc<Self>,
        id: Uuid,
        new_url: String,
    ) -> anyhow::Result<()> {
        let trimmed = new_url.trim().to_string();
        if trimmed.is_empty() {
            return Ok(());
        }

        let task = {
            let tasks = self.active_tasks.read().await;
            tasks.get(&id).cloned()
        };

        if let Some(task) = task {
            info!("Updating URL for task {} to {}", task.filename, trimmed);
            *task.url.write().await = trimmed.clone();

            let pool = self.database.pool().clone();
            let task_id_str = task.id.to_string();
            let url_for_db = trimmed;
            tokio::spawn(async move {
                let repo = DownloadRepository::new(pool);
                let _ = repo
                    .update_url(&task_id_str, &url_for_db, &Utc::now().to_rfc3339())
                    .await;
            });

            self.retry_task(id).await?;
        }
        Ok(())
    }

    pub async fn cancel_task(self: &Arc<Self>, id: Uuid) -> anyhow::Result<()> {
        // Remove from active_tasks and task_order FIRST and drop both write locks immediately
        // so check_queue_and_dispatch() and the UI refresh loop never deadlock!
        let removed_task = {
            let mut tasks = self.active_tasks.write().await;
            tasks.remove(&id)
        };

        {
            let mut order = self.task_order.write().await;
            order.retain(|&x| x != id);
        }

        if let Some(task) = removed_task {
            let was_completed = *task.status.read().await == DownloadStatus::Completed;
            self.dynamic_layouts.write().await.remove(&id);
            self.segment_pools.write().await.remove(&id);
            self.active_streams.write().await.remove(&id);

            if was_completed {
                info!("Removing completed task from Tasks view: {}", task.filename);
            } else {
                info!("Cancelling task: {}", task.filename);
                *task.status.write().await = DownloadStatus::Cancelled;
                if let Some(tx) = task.pause_tx.write().await.take() {
                    let _ = tx.send(true);
                }

                let pool = self.database.pool().clone();
                let task_clone = task.clone();
                tokio::spawn(async move {
                    let repo = DownloadRepository::new(pool);
                    let _ = repo.delete_download(&task_clone.id.to_string()).await;
                    let _guard = task_clone.job_lock.lock().await;
                    Self::cleanup_temp_files(&task_clone.destination_path).await;
                });
            }
        } else {
            self.remove_from_queue(id).await;
        }

        let svc = self.clone();
        tokio::spawn(async move {
            svc.check_queue_and_dispatch().await;
        });
        Ok(())
    }

    pub async fn open_folder_task(&self, id: Uuid) {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&id) {
            let _ = PlatformSystem::reveal_in_folder(&task.destination_path);
        }
    }

    pub async fn open_file_task(&self, id: Uuid) {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&id) {
            let _ = PlatformSystem::open_file(&task.destination_path);
        }
    }

    pub fn check_queue_and_dispatch(
        self: Arc<Self>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            if self.queue_paused.load(Ordering::Relaxed) {
                return;
            }

            let max_sim = self.max_simultaneous.load(Ordering::Relaxed);
            let active_count = self.get_active_downloading_count().await;

            if active_count >= max_sim {
                return;
            }

            let mut q = self.queue.write().await;
            if q.is_empty() {
                return;
            }

            let next_item = q.remove(0);
            drop(q);

            info!("Dispatching queued download: {}", next_item.filename);

            let target_file_path = next_item.destination_folder.join(&next_item.filename);
            let max_conns = self.max_connections.load(Ordering::Relaxed);
            let now = Utc::now();

            let (pause_tx, pause_rx) = watch::channel(false);
            let active_task = Arc::new(ActiveTask {
                id: next_item.id,
                url: Arc::new(RwLock::new(next_item.url.clone())),
                filename: next_item.filename.clone(),
                destination_path: target_file_path,
                total_bytes: next_item.total_bytes,
                downloaded_bytes: Arc::new(AtomicU64::new(0)),
                status: Arc::new(RwLock::new(DownloadStatus::Downloading)),
                supports_range: next_item.supports_range,
                etag: next_item.etag.clone(),
                connections: max_conns,
                pause_tx: Arc::new(RwLock::new(Some(pause_tx))),
                job_lock: Arc::new(Mutex::new(())),
                created_at: now,
                checksum: Arc::new(RwLock::new(None)),
                last_bytes: Arc::new(AtomicU64::new(0)),
                instant_speed: Arc::new(AtomicU64::new(0)),
                error_message: Arc::new(RwLock::new(None)),
            });

            {
                let mut tasks = self.active_tasks.write().await;
                tasks.insert(next_item.id, active_task.clone());
                let mut order = self.task_order.write().await;
                order.push(next_item.id);
            }

            let repo = DownloadRepository::new(self.database.pool().clone());
            let _ = repo
                .update_status(&next_item.id.to_string(), "downloading", &now.to_rfc3339())
                .await;

            NotificationService::notify_download_starting(&active_task.filename);

            let svc = self.clone();
            tokio::spawn(async move {
                svc.run_download_job(active_task, pause_rx).await;
            });
        })
    }

    // Queue methods
    pub async fn pause_resume_queue(self: Arc<Self>) {
        let was_paused = self.queue_paused.fetch_xor(true, Ordering::Relaxed);
        info!("Queue pause toggled. Now paused: {}", !was_paused);
        if was_paused {
            self.clone().check_queue_and_dispatch().await;
        }
    }

    pub async fn clear_queue(&self) {
        let mut q = self.queue.write().await;
        info!("Clearing {} items from queue", q.len());
        q.clear();
    }

    pub async fn move_queue_up(&self, id: Uuid) {
        let mut q = self.queue.write().await;
        if let Some(pos) = q.iter().position(|it| it.id == id) {
            if pos > 0 {
                q.swap(pos, pos - 1);
            }
        }
    }

    pub async fn move_queue_down(&self, id: Uuid) {
        let mut q = self.queue.write().await;
        if let Some(pos) = q.iter().position(|it| it.id == id) {
            if pos + 1 < q.len() {
                q.swap(pos, pos + 1);
            }
        }
    }

    pub async fn remove_from_queue(&self, id: Uuid) {
        let mut q = self.queue.write().await;
        q.retain(|it| it.id != id);
        let repo = DownloadRepository::new(self.database.pool().clone());
        let _ = repo.delete_download(&id.to_string()).await;
    }

    // History methods
    pub async fn clear_history(&self) -> anyhow::Result<()> {
        info!("Clearing download history");
        let repo = DownloadRepository::new(self.database.pool().clone());
        repo.clear_history().await
    }

    pub async fn delete_history_item(&self, id: &str, delete_file: bool) -> anyhow::Result<()> {
        let repo = DownloadRepository::new(self.database.pool().clone());
        if delete_file {
            if let Ok(downloads) = repo.get_history().await {
                if let Some(row) = downloads.iter().find(|r| r.id == id) {
                    info!("Deleting downloaded file from disk: {}", row.destination_path);
                    let _ = fs::remove_file(&row.destination_path).await;
                }
            }
            if let Ok(uuid) = Uuid::parse_str(id) {
                self.active_tasks.write().await.remove(&uuid);
                self.task_order.write().await.retain(|&x| x != uuid);
            }
        }
        repo.delete_download(id).await
    }

    pub async fn open_history_file(&self, id: &str) {
        let repo = DownloadRepository::new(self.database.pool().clone());
        if let Ok(downloads) = repo.get_history().await {
            if let Some(row) = downloads.iter().find(|r| r.id == id) {
                let _ = PlatformSystem::open_file(&row.destination_path);
            }
        }
    }

    pub async fn open_history_folder(&self, id: &str) {
        let repo = DownloadRepository::new(self.database.pool().clone());
        if let Ok(downloads) = repo.get_history().await {
            if let Some(row) = downloads.iter().find(|r| r.id == id) {
                let _ = PlatformSystem::reveal_in_folder(&row.destination_path);
            }
        }
    }

    // UI Model generators
    pub async fn get_downloads_ui(&self) -> Vec<DownloadItemData> {
        let tasks = self.active_tasks.read().await;
        let order = self.task_order.read().await;
        let streams_map = self.active_streams.read().await;
        let seg_pools = self.segment_pools.read().await;
        let mut items = Vec::new();

        for id in order.iter() {
            if let Some(task) = tasks.get(id) {
                let downloaded = task.downloaded_bytes.load(Ordering::Relaxed);
                let total = task.total_bytes;
                let status = *task.status.read().await;
                let speed_bps = task.instant_speed.load(Ordering::Relaxed);
                let active_conns = streams_map
                    .get(id)
                    .map(|a| a.load(Ordering::Relaxed) as i32)
                    .unwrap_or(0);

                let progress = match total {
                    Some(tot) if tot > 0 => (downloaded as f32 / tot as f32).clamp(0.0, 1.0),
                    _ if status == DownloadStatus::Completed => 1.0,
                    _ => 0.0,
                };

                let mut seg_buckets = [0.0f32; 8];
                let mut seg_count = 1i32;
                if status != DownloadStatus::Completed {
                    if let Some(pool) = seg_pools.get(id) {
                        let guard = pool.lock().await;
                        let mut raw_segs: Vec<(u64, u64, u64)> = guard
                            .iter()
                            .map(|s| {
                                (
                                    s.start_byte,
                                    s.end_byte.load(Ordering::Relaxed),
                                    s.current_len.load(Ordering::Relaxed),
                                )
                            })
                            .collect();
                        drop(guard);
                        raw_segs.sort_by_key(|&(start_b, _, _)| start_b);

                        if raw_segs.len() > 1 {
                            seg_count = raw_segs.len() as i32;
                            if let Some(tot) = total.filter(|&t| t > 0) {
                                let num_buckets = 8usize;
                                for b_idx in 0..num_buckets {
                                    let b_start = (b_idx as u64 * tot) / num_buckets as u64;
                                    let b_end = if b_idx + 1 == num_buckets {
                                        tot.saturating_sub(1)
                                    } else {
                                        ((b_idx as u64 + 1) * tot) / num_buckets as u64 - 1
                                    };
                                    let b_span = (b_end.saturating_sub(b_start) + 1).max(1);
                                    let mut b_dl = 0u64;
                                    for &(s_start, _s_end, s_cur) in &raw_segs {
                                        if s_cur == 0 {
                                            continue;
                                        }
                                        let dl_end = s_start.saturating_add(s_cur).saturating_sub(1);
                                        let ov_start = b_start.max(s_start);
                                        let ov_end = b_end.min(dl_end);
                                        if ov_end >= ov_start {
                                            b_dl += (ov_end - ov_start) + 1;
                                        }
                                    }
                                    seg_buckets[b_idx] =
                                        (b_dl as f32 / b_span as f32).clamp(0.0, 1.0);
                                }
                            } else {
                                for (idx, (start_b, end_b, cur_len)) in
                                    raw_segs.into_iter().take(8).enumerate()
                                {
                                    let span = (end_b.saturating_sub(start_b) + 1).max(1);
                                    seg_buckets[idx] =
                                        (cur_len as f32 / span as f32).clamp(0.0, 1.0);
                                }
                            }
                        }
                    }
                }

                let speed_str = if status == DownloadStatus::Downloading {
                    format_speed(speed_bps)
                } else {
                    String::new()
                };

                let eta_str = if status == DownloadStatus::Downloading {
                    if let Some(tot) = total {
                        if speed_bps > 0 {
                            let rem = tot.saturating_sub(downloaded);
                            format_eta(rem / speed_bps)
                        } else {
                            "--:--".to_string()
                        }
                    } else {
                        "--:--".to_string()
                    }
                } else {
                    String::new()
                };

                let cat_str = FileCategory::from_filename(&task.filename).as_str().to_string();
                let full_sha = task.checksum.read().await.clone().unwrap_or_default();
                let short_sha = if full_sha.len() >= 12 {
                    format!("{}...{}", &full_sha[..6], &full_sha[full_sha.len() - 6..])
                } else {
                    full_sha.clone()
                };
                let task_url_str = task.url.read().await.clone();

                items.push(DownloadItemData {
                    id: SharedString::from(task.id.to_string()),
                    filename: SharedString::from(task.filename.clone()),
                    url: SharedString::from(task_url_str),
                    status: SharedString::from(status.as_str()),
                    downloaded_bytes: SharedString::from(format_bytes(downloaded)),
                    total_bytes: SharedString::from(
                        total.map(format_bytes).unwrap_or_else(|| "Unknown".to_string()),
                    ),
                    progress,
                    speed: SharedString::from(speed_str),
                    eta: SharedString::from(eta_str),
                    active_connections: active_conns,
                    category: SharedString::from(cat_str),
                    checksum: SharedString::from(full_sha),
                    checksum_short: SharedString::from(short_sha),
                    segment_count: seg_count,
                    seg0: seg_buckets[0],
                    seg1: seg_buckets[1],
                    seg2: seg_buckets[2],
                    seg3: seg_buckets[3],
                    seg4: seg_buckets[4],
                    seg5: seg_buckets[5],
                    seg6: seg_buckets[6],
                    seg7: seg_buckets[7],
                });
            }
        }

        items
    }

    pub async fn get_queue_ui(&self) -> Vec<QueueItemData> {
        let q = self.queue.read().await;
        let mut items = Vec::new();

        for (idx, it) in q.iter().enumerate() {
            items.push(QueueItemData {
                id: SharedString::from(it.id.to_string()),
                position: (idx + 1) as i32,
                file_name: SharedString::from(it.filename.clone()),
                file_size_display: SharedString::from(
                    it.total_bytes
                        .map(format_bytes)
                        .unwrap_or_else(|| "Unknown size".to_string()),
                ),
                url: SharedString::from(it.url.clone()),
                priority: SharedString::from(it.priority.clone()),
            });
        }

        items
    }

    pub async fn get_history_ui(&self, filter: &str) -> Vec<HistoryItemData> {
        let repo = DownloadRepository::new(self.database.pool().clone());
        let rows = repo.get_history().await.unwrap_or_default();
        let filter_lower = filter.trim().to_lowercase();
        let mut items = Vec::new();

        for row in rows {
            if !filter_lower.is_empty()
                && !row.filename.to_lowercase().contains(&filter_lower)
                && !row.destination_path.to_lowercase().contains(&filter_lower)
            {
                continue;
            }

            let size_display = row
                .total_bytes
                .map(|b| format_bytes(b as u64))
                .unwrap_or_else(|| format_bytes(row.downloaded_bytes as u64));

            let date_display = DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|_| row.updated_at.clone());

            let status_display = if row.status == "completed" {
                "Completed"
            } else {
                "Failed"
            };

            items.push(HistoryItemData {
                id: SharedString::from(row.id),
                file_name: SharedString::from(row.filename),
                file_size_display: SharedString::from(size_display),
                completed_at: SharedString::from(date_display),
                file_path: SharedString::from(row.destination_path),
                checksum: SharedString::from(row.checksum.unwrap_or_default()),
                status: SharedString::from(status_display),
            });
        }

        items
    }

    pub async fn update_metrics_and_get_total_speed(&self) -> String {
        let tasks = self.active_tasks.read().await;
        let mut total_bps = 0u64;

        for task in tasks.values() {
            let current = task.downloaded_bytes.load(Ordering::Relaxed);
            let prev = task.last_bytes.swap(current, Ordering::Relaxed);
            let delta = current.saturating_sub(prev);

            // Ticker runs every 300ms, so speed = delta / 0.3
            let speed = (delta as f64 / 0.3) as u64;
            task.instant_speed.store(speed, Ordering::Relaxed);

            if *task.status.read().await == DownloadStatus::Downloading {
                total_bps += speed;
            }
        }

        format_speed(total_bps)
    }
}
