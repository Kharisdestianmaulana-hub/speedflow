#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod core;
pub mod engine;
pub mod platform;
pub mod services;
pub mod storage;
pub mod ui;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use tokio::sync::mpsc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use crate::app::service::format_bytes;
use crate::app::AppState;
use crate::core::download_task::DownloadStatus;
use crate::platform::{NotificationTarget, PlatformSystem, SystemTray, TrayEvent};
use crate::services::{
    BridgeServer, BrowserIntegrationService, ClipboardService, FileCategory, InterceptedDownload,
    NotificationService,
};
use crate::storage::{AppSettings, DownloadRepository};

#[cfg(windows)]
fn bring_window_to_foreground(window_title: &str) {
    use std::os::windows::ffi::OsStrExt;
    let title: Vec<u16> = std::ffi::OsStr::new(window_title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        unsafe extern "system" {
            fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
            fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
            fn SetForegroundWindow(hWnd: isize) -> i32;
            fn BringWindowToTop(hWnd: isize) -> i32;
        }
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if hwnd != 0 {
            const SW_RESTORE: i32 = 9;
            ShowWindow(hwnd, SW_RESTORE);
            BringWindowToTop(hwnd);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(windows)]
fn bring_popup_to_foreground() {
    bring_window_to_foreground("SpeedFlow - Download File Info");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting SpeedFlow Desktop Download Manager...");

    let app_state = AppState::new().await?;
    let main_window = ui::MainWindow::new()?;
    let add_popup_window = ui::AddDownloadWindow::new()?;

    // Set default downloads destination folder & load persisted AppSettings from SQLite
    let default_dest = if let Some(user_dirs) = directories::UserDirs::new() {
        user_dirs
            .download_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\Users\\riray\\Downloads".to_string())
    } else {
        "C:\\Users\\riray\\Downloads".to_string()
    };

    let service = app_state.download_service.clone();
    let repo = DownloadRepository::new(app_state.database.pool().clone());
    if let Some(saved) = repo.load_settings().await {
        let base_dest = if saved.destination_path.trim().is_empty() {
            default_dest.clone()
        } else {
            saved.destination_path.clone()
        };
        main_window.set_destination_path(base_dest.into());
        main_window.set_settings_max_connections(saved.max_connections.clamp(1, 32));
        main_window.set_settings_max_simultaneous(saved.max_simultaneous.clamp(1, 10));
        main_window.set_settings_auto_retry(saved.auto_retry);
        main_window.set_settings_auto_categorize(saved.auto_categorize);
        main_window.set_settings_speed_limit_enabled(saved.speed_limit_enabled);
        main_window.set_settings_speed_limit_kb(saved.speed_limit_kb.into());
        main_window.set_is_dark(saved.is_dark);
        main_window.set_show_onboarding(!saved.onboarding_completed);

        service
            .max_connections
            .store(saved.max_connections.clamp(1, 32) as u32, std::sync::atomic::Ordering::Relaxed);
        service
            .max_simultaneous
            .store(saved.max_simultaneous.clamp(1, 10) as usize, std::sync::atomic::Ordering::Relaxed);
        service
            .auto_categorize
            .store(saved.auto_categorize, std::sync::atomic::Ordering::Relaxed);
    } else {
        main_window.set_destination_path(default_dest.into());
        main_window.set_show_onboarding(true);
    }
    main_window.set_settings_autostart(PlatformSystem::is_autostart_enabled());

    // Initialize Browser Integration Status
    {
        let status = BrowserIntegrationService::get_status();
        main_window.set_browser_chrome_installed(status.chrome_installed);
        main_window.set_browser_chrome_integrated(status.chrome_integrated);
        main_window.set_browser_edge_installed(status.edge_installed);
        main_window.set_browser_edge_integrated(status.edge_integrated);
        main_window.set_browser_firefox_installed(status.firefox_installed);
        main_window.set_browser_firefox_integrated(status.firefox_integrated);
        main_window.set_browser_brave_installed(status.brave_installed);
        main_window.set_browser_brave_integrated(status.brave_integrated);
        main_window.set_browser_bridge_active(true);
    }

    // 0. Start Browser Bridge Server (Port 19760) for Real-Time Browser Extension Interception
    let (event_tx, mut event_rx) = mpsc::channel::<InterceptedDownload>(64);
    let bridge_server = Arc::new(BridgeServer::new(19760, event_tx));
    tokio::spawn(async move {
        let _ = bridge_server.run().await;
    });

    // Listen for browser intercepted downloads
    {
        let http_engine = service.http_engine.clone();
        let window_weak = main_window.as_weak();
        let popup_weak_outer = add_popup_window.as_weak();
        tokio::spawn(async move {
            while let Some(item) = event_rx.recv().await {
                info!("Handling intercepted download: {:?}", item.url);
                let url = item.url.clone();

                // Register browser session context (Cookie, Referer, User-Agent) for multi-connection acceleration
                http_engine.register_session(
                    &url,
                    crate::engine::http::SessionContext {
                        referrer: item.referrer.clone(),
                        cookies: item.cookies.clone(),
                        user_agent: item.user_agent.clone(),
                    },
                );

                let inspected = http_engine.inspect_url(&url).await;

                let (filename, size_str, category_str, resume_str) = match inspected {
                    Ok(info) => {
                        let name = if info.filename != "downloaded_file.bin" && !info.filename.is_empty() {
                            info.filename
                        } else {
                            item.filename
                                .filter(|f| !f.is_empty())
                                .unwrap_or(info.filename)
                        };
                        let sz = info
                            .content_length
                            .map(format_bytes)
                            .unwrap_or_else(|| "Unknown Size".to_string());
                        let cat = FileCategory::from_filename(&name).as_str().to_string();
                        let res = if info.supports_range {
                            "Supported".to_string()
                        } else {
                            "No Range".to_string()
                        };
                        (name, sz, cat, res)
                    }
                    Err(_) => {
                        let name = item
                            .filename
                            .filter(|f| !f.is_empty())
                            .unwrap_or_else(|| "downloaded_file.bin".to_string());
                        let cat = FileCategory::from_filename(&name).as_str().to_string();
                        (name, "Unknown Size".to_string(), cat, "Supported".to_string())
                    }
                };

                let w_weak = window_weak.clone();
                let popup_weak = popup_weak_outer.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let (cur_dest, auto_cat, max_conns) = if let Some(w) = w_weak.upgrade() {
                        (
                            w.get_destination_path().to_string(),
                            w.get_settings_auto_categorize(),
                            w.get_settings_max_connections(),
                        )
                    } else {
                        ("C:\\Users\\riray\\Downloads".to_string(), true, 16)
                    };

                    let resolved_dest = FileCategory::resolve_category_folder(
                        std::path::Path::new(&cur_dest),
                        &filename,
                        auto_cat,
                    );

                    // Open ONLY the standalone Download File Info popup window!
                    if let Some(popup) = popup_weak.upgrade() {
                        popup.set_destination_path(
                            resolved_dest.to_string_lossy().to_string().into(),
                        );
                        popup.set_new_url_text(url.into());
                        popup.set_modal_filename(filename.into());
                        popup.set_modal_filesize(size_str.into());
                        popup.set_modal_category(category_str.into());
                        popup.set_modal_resume_supported(resume_str.into());
                        popup.set_settings_max_connections(max_conns);
                        let _ = popup.show();

                        #[cfg(windows)]
                        bring_popup_to_foreground();
                    }
                });
            }
        });
    }

    // Wire standalone AddDownloadWindow callbacks
    {
        let popup_weak = add_popup_window.as_weak();
        add_popup_window.on_close_popup(move || {
            if let Some(p) = popup_weak.upgrade() {
                let _ = p.hide();
            }
        });
    }

    {
        let service = service.clone();
        add_popup_window.on_start_download_full(move |url, filename, dest| {
            let service = service.clone();
            let url_str = url.to_string();
            let fn_str = if filename.trim().is_empty() {
                None
            } else {
                Some(filename.to_string())
            };
            let dest_path = PathBuf::from(dest.to_string());
            tokio::spawn(async move {
                if let Err(e) = service
                    .start_download_full(url_str, fn_str, Some(dest_path))
                    .await
                {
                    error!("Failed to start full download from popup: {:?}", e);
                }
            });
        });
    }

    {
        let service = service.clone();
        add_popup_window.on_queue_download_full(move |url, filename, dest| {
            let service = service.clone();
            let url_str = url.to_string();
            let fn_str = if filename.trim().is_empty() {
                None
            } else {
                Some(filename.to_string())
            };
            let dest_path = PathBuf::from(dest.to_string());
            tokio::spawn(async move {
                if let Err(e) = service
                    .queue_download_full(url_str, fn_str, Some(dest_path))
                    .await
                {
                    error!("Failed to queue download from popup: {:?}", e);
                }
            });
        });
    }

    {
        let http_engine = service.http_engine.clone();
        let popup_weak = add_popup_window.as_weak();
        let window_weak = main_window.as_weak();
        add_popup_window.on_inspect_url_requested(move |url_str| {
            let url = url_str.to_string();
            let http_engine = http_engine.clone();
            let popup_weak = popup_weak.clone();
            let window_weak = window_weak.clone();

            tokio::spawn(async move {
                if let Ok(info) = http_engine.inspect_url(&url).await {
                    let name = info.filename;
                    let sz = info
                        .content_length
                        .map(format_bytes)
                        .unwrap_or_else(|| "Unknown Size".to_string());
                    let cat = FileCategory::from_filename(&name).as_str().to_string();
                    let res = if info.supports_range {
                        "Supported".to_string()
                    } else {
                        "No Range".to_string()
                    };

                    let _ = slint::invoke_from_event_loop(move || {
                        let auto_cat = window_weak
                            .upgrade()
                            .map(|w| w.get_settings_auto_categorize())
                            .unwrap_or(true);
                        if let Some(p) = popup_weak.upgrade() {
                            let cur_dest = p.get_destination_path().to_string();
                            let resolved_dest = FileCategory::resolve_category_folder(
                                std::path::Path::new(&cur_dest),
                                &name,
                                auto_cat,
                            );
                            p.set_destination_path(
                                resolved_dest.to_string_lossy().to_string().into(),
                            );
                            p.set_modal_filename(name.into());
                            p.set_modal_filesize(sz.into());
                            p.set_modal_category(cat.into());
                            p.set_modal_resume_supported(res.into());
                        }
                    });
                }
            });
        });
    }

    {
        let popup_weak = add_popup_window.as_weak();
        add_popup_window.on_browse_destination(move || {
            let popup_weak = popup_weak.clone();
            tokio::task::spawn_blocking(move || {
                let folder = rfd::FileDialog::new()
                    .set_title("Select Download Destination Folder")
                    .pick_folder();
                if let Some(path) = folder {
                    let path_str = path.to_string_lossy().to_string();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(p) = popup_weak.upgrade() {
                            p.set_destination_path(path_str.into());
                        }
                    });
                }
            });
        });
    }

    // 1. Callbacks: Start Download (Legacy & Full)
    {
        let service = service.clone();
        main_window.on_start_download(move |url, dest| {
            let service = service.clone();
            let url_str = url.to_string();
            let dest_path = PathBuf::from(dest.to_string());
            tokio::spawn(async move {
                if let Err(e) = service.start_download(url_str, Some(dest_path)).await {
                    error!("Failed to start download: {:?}", e);
                }
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_start_download_full(move |url, filename, dest| {
            let service = service.clone();
            let url_str = url.to_string();
            let fn_str = if filename.trim().is_empty() {
                None
            } else {
                Some(filename.to_string())
            };
            let dest_path = PathBuf::from(dest.to_string());
            tokio::spawn(async move {
                if let Err(e) = service
                    .start_download_full(url_str, fn_str, Some(dest_path))
                    .await
                {
                    error!("Failed to start full download: {:?}", e);
                }
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_queue_download_full(move |url, filename, dest| {
            let service = service.clone();
            let url_str = url.to_string();
            let fn_str = if filename.trim().is_empty() {
                None
            } else {
                Some(filename.to_string())
            };
            let dest_path = PathBuf::from(dest.to_string());
            tokio::spawn(async move {
                if let Err(e) = service
                    .queue_download_full(url_str, fn_str, Some(dest_path))
                    .await
                {
                    error!("Failed to queue download: {:?}", e);
                }
            });
        });
    }

    // Inspect URL when typed or pasted in modal
    {
        let http_engine = service.http_engine.clone();
        let window_weak = main_window.as_weak();
        main_window.on_inspect_url_requested(move |url_str| {
            let url = url_str.to_string();
            let http_engine = http_engine.clone();
            let window_weak = window_weak.clone();

            tokio::spawn(async move {
                if let Ok(info) = http_engine.inspect_url(&url).await {
                    let name = info.filename;
                    let sz = info
                        .content_length
                        .map(format_bytes)
                        .unwrap_or_else(|| "Unknown Size".to_string());
                    let cat = FileCategory::from_filename(&name).as_str().to_string();
                    let res = if info.supports_range {
                        "Supported".to_string()
                    } else {
                        "No Range".to_string()
                    };

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            let cur_dest = w.get_destination_path().to_string();
                            let auto_cat = w.get_settings_auto_categorize();
                            let resolved_dest = FileCategory::resolve_category_folder(
                                std::path::Path::new(&cur_dest),
                                &name,
                                auto_cat,
                            );
                            w.set_destination_path(
                                resolved_dest.to_string_lossy().to_string().into(),
                            );
                            w.set_modal_filename(name.into());
                            w.set_modal_filesize(sz.into());
                            w.set_modal_category(cat.into());
                            w.set_modal_resume_supported(res.into());
                        }
                    });
                }
            });
        });
    }

    // Smart Clipboard Auto-Detection when clicking '+' FAB button
    {
        let http_engine = service.http_engine.clone();
        let window_weak = main_window.as_weak();
        main_window.on_open_add_dialog_requested(move || {
            if let Some(clip_url) = ClipboardService::get_clipboard_url() {
                if let Some(w) = window_weak.upgrade() {
                    let current_url = w.get_new_url_text().to_string();
                    if current_url.trim().is_empty() || current_url.trim() != clip_url {
                        w.set_new_url_text(clip_url.clone().into());
                        let http_engine = http_engine.clone();
                        let window_weak = window_weak.clone();
                        tokio::spawn(async move {
                            if let Ok(info) = http_engine.inspect_url(&clip_url).await {
                                let name = info.filename;
                                let sz = info
                                    .content_length
                                    .map(format_bytes)
                                    .unwrap_or_else(|| "Unknown Size".to_string());
                                let cat = FileCategory::from_filename(&name).as_str().to_string();
                                let res = if info.supports_range {
                                    "Supported".to_string()
                                } else {
                                    "No Range".to_string()
                                };

                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(w) = window_weak.upgrade() {
                                        let cur_dest = w.get_destination_path().to_string();
                                        let auto_cat = w.get_settings_auto_categorize();
                                        let resolved_dest = FileCategory::resolve_category_folder(
                                            std::path::Path::new(&cur_dest),
                                            &name,
                                            auto_cat,
                                        );
                                        w.set_destination_path(
                                            resolved_dest.to_string_lossy().to_string().into(),
                                        );
                                        w.set_modal_filename(name.into());
                                        w.set_modal_filesize(sz.into());
                                        w.set_modal_category(cat.into());
                                        w.set_modal_resume_supported(res.into());
                                    }
                                });
                            }
                        });
                    }
                }
            }
        });
    }

    // 2. Callback: Pause / Resume Task
    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_pause_resume_task(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                let window_weak = window_weak.clone();
                tokio::spawn(async move {
                    let status_opt = {
                        let tasks = service.active_tasks.read().await;
                        if let Some(task) = tasks.get(&id) {
                            Some(*task.status.read().await)
                        } else {
                            None
                        }
                    };
                    if let Some(status) = status_opt {
                        if status == DownloadStatus::Downloading {
                            let _ = service.pause_task(id).await;
                        } else if status == DownloadStatus::Paused {
                            let _ = service.resume_task(id).await;
                        }
                        let downloads_ui = service.get_downloads_ui().await;
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = window_weak.upgrade() {
                                w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                            }
                        });
                    }
                });
            }
        });
    }

    // 3. Callback: Cancel Task
    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_cancel_task(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                let window_weak = window_weak.clone();
                tokio::spawn(async move {
                    let _ = service.cancel_task(id).await;
                    let downloads_ui = service.get_downloads_ui().await;
                    let queue_ui = service.get_queue_ui().await;
                    let mut active_count = 0;
                    let mut completed_count = 0;
                    for item in &downloads_ui {
                        if item.status == "downloading"
                            || item.status == "paused"
                            || item.status == "retrying"
                        {
                            active_count += 1;
                        } else if item.status == "completed" {
                            completed_count += 1;
                        }
                    }
                    let queued_count = queue_ui.len() as i32;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                            w.set_queue_items(ModelRc::new(VecModel::from(queue_ui)));
                            w.set_active_downloads_count(active_count);
                            w.set_completed_downloads_count(completed_count);
                            w.set_queued_downloads_count(queued_count);
                        }
                    });
                });
            }
        });
    }

    // 4. Callback: Open Folder / Open File / Retry for Task
    {
        let service = service.clone();
        main_window.on_open_folder_task(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                tokio::spawn(async move {
                    service.open_folder_task(id).await;
                });
            }
        });
    }

    {
        let service = service.clone();
        main_window.on_open_file_task(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                tokio::spawn(async move {
                    service.open_file_task(id).await;
                });
            }
        });
    }

    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_retry_task(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                let window_weak = window_weak.clone();
                tokio::spawn(async move {
                    let _ = service.retry_task(id).await;
                    let downloads_ui = service.get_downloads_ui().await;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                        }
                    });
                });
            }
        });
    }

    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_update_task_url(move |id_str, new_url_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                let window_weak = window_weak.clone();
                let new_url = new_url_str.to_string();
                tokio::spawn(async move {
                    let _ = service.update_task_url_and_resume(id, new_url).await;
                    let downloads_ui = service.get_downloads_ui().await;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                        }
                    });
                });
            }
        });
    }

    {
        let window_weak = main_window.as_weak();
        main_window.on_paste_update_url(move || {
            if let Some(clip_url) = ClipboardService::get_clipboard_url() {
                if let Some(w) = window_weak.upgrade() {
                    w.set_update_url_new_text(clip_url.into());
                }
            }
        });
    }

    // 5. Callback: Browse Destination Folder (Native File Dialog)
    {
        let window_weak = main_window.as_weak();
        main_window.on_browse_destination(move || {
            let window_weak = window_weak.clone();
            tokio::task::spawn_blocking(move || {
                let folder = rfd::FileDialog::new()
                    .set_title("Select Download Destination Folder")
                    .pick_folder();
                if let Some(path) = folder {
                    let path_str = path.to_string_lossy().to_string();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = window_weak.upgrade() {
                            w.set_destination_path(path_str.into());
                        }
                    });
                }
            });
        });
    }

    // 6. Queue Callbacks
    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_pause_resume_queue(move || {
            let service = service.clone();
            let window_weak = window_weak.clone();
            tokio::spawn(async move {
                service.clone().pause_resume_queue().await;
                let is_paused = service.queue_paused.load(std::sync::atomic::Ordering::Relaxed);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = window_weak.upgrade() {
                        w.set_queue_paused(is_paused);
                    }
                });
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_clear_queue(move || {
            let service = service.clone();
            tokio::spawn(async move {
                service.clear_queue().await;
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_move_queue_up(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                tokio::spawn(async move {
                    service.move_queue_up(id).await;
                });
            }
        });
    }

    {
        let service = service.clone();
        main_window.on_move_queue_down(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                tokio::spawn(async move {
                    service.move_queue_down(id).await;
                });
            }
        });
    }

    {
        let service = service.clone();
        main_window.on_remove_from_queue(move |id_str| {
            if let Ok(id) = Uuid::parse_str(id_str.as_str()) {
                let service = service.clone();
                tokio::spawn(async move {
                    service.remove_from_queue(id).await;
                });
            }
        });
    }

    // 7. History Callbacks
    {
        let service = service.clone();
        main_window.on_clear_history(move || {
            let service = service.clone();
            tokio::spawn(async move {
                let _ = service.clear_history().await;
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_open_history_file(move |id_str| {
            let service = service.clone();
            let id = id_str.to_string();
            tokio::spawn(async move {
                service.open_history_file(&id).await;
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_open_history_folder(move |id_str| {
            let service = service.clone();
            let id = id_str.to_string();
            tokio::spawn(async move {
                service.open_history_folder(&id).await;
            });
        });
    }

    {
        let service = service.clone();
        main_window.on_delete_history_item(move |id_str, delete_file| {
            let service = service.clone();
            let id = id_str.to_string();
            tokio::spawn(async move {
                let _ = service.delete_history_item(&id, delete_file).await;
            });
        });
    }

    // 8. Browser Integration Callbacks
    {
        let window_weak = main_window.as_weak();
        main_window.on_install_browser_integration(move || {
            let res = BrowserIntegrationService::install_integration();
            if let Some(w) = window_weak.upgrade() {
                match res {
                    Ok(status) => {
                        w.set_browser_chrome_integrated(status.chrome_integrated);
                        w.set_browser_edge_integrated(status.edge_integrated);
                        w.set_browser_firefox_integrated(status.firefox_integrated);
                        w.set_browser_brave_integrated(status.brave_integrated);

                        let msg = if status.edge_integrated && status.firefox_integrated {
                            "Successfully connected to Microsoft Edge and Mozilla Firefox! Please restart your browser to activate."
                        } else if status.edge_integrated {
                            "Successfully connected to Microsoft Edge! Please restart Edge to activate the extension."
                        } else if status.firefox_integrated {
                            "Successfully connected to Mozilla Firefox! Please restart Firefox to activate the extension."
                        } else if !status.edge_installed && !status.firefox_installed {
                            if status.chrome_installed || status.brave_installed {
                                "Edge and Firefox are not installed on this computer. SpeedFlow is ready and connected for Chrome, Brave, and other browsers."
                            } else {
                                "Edge and Firefox are not installed on this computer. SpeedFlow is ready and waiting to catch downloads."
                            }
                        } else {
                            "Browser connection is active. The extension is ready to use."
                        };
                        w.set_browser_integration_msg(msg.into());
                    }
                    Err(e) => {
                        w.set_browser_integration_msg(format!("Note: {e}").into());
                    }
                }
            }
        });
    }

    {
        let window_weak = main_window.as_weak();
        main_window.on_remove_browser_integration(move || {
            let res = BrowserIntegrationService::remove_integration();
            if let Some(w) = window_weak.upgrade() {
                match res {
                    Ok(status) => {
                        w.set_browser_chrome_integrated(status.chrome_integrated);
                        w.set_browser_edge_integrated(status.edge_integrated);
                        w.set_browser_firefox_integrated(status.firefox_integrated);
                        w.set_browser_brave_integrated(status.brave_integrated);
                        w.set_browser_integration_msg("Browser integration has been disconnected.".into());
                    }
                    Err(e) => {
                        w.set_browser_integration_msg(format!("Note: {e}").into());
                    }
                }
            }
        });
    }

    main_window.on_open_extension_folder(move || {
        let _ = BrowserIntegrationService::open_extension_folder();
    });

    {
        let window_weak = main_window.as_weak();
        main_window.on_copy_chrome_extensions_url(move || {
            let ok = ClipboardService::set_clipboard_text("chrome://extensions");
            if let Some(w) = window_weak.upgrade() {
                if ok {
                    w.set_browser_integration_msg(
                        "Copied 'chrome://extensions' to clipboard! Paste it into your Chrome/Brave address bar.".into(),
                    );
                } else {
                    w.set_browser_integration_msg(
                        "Please type 'chrome://extensions' in your Chrome/Brave address bar.".into(),
                    );
                }
            }
        });
    }

    // 8b. Bulk Pause/Resume, Copy SHA-256 Checksum, and Windows Auto-Start Toggle
    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_pause_all_tasks(move || {
            let service = service.clone();
            let window_weak = window_weak.clone();
            tokio::spawn(async move {
                service.pause_all_tasks().await;
                let downloads_ui = service.get_downloads_ui().await;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = window_weak.upgrade() {
                        w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                    }
                });
            });
        });
    }

    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        main_window.on_resume_all_tasks(move || {
            let service = service.clone();
            let window_weak = window_weak.clone();
            tokio::spawn(async move {
                service.resume_all_tasks().await;
                let downloads_ui = service.get_downloads_ui().await;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = window_weak.upgrade() {
                        w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                    }
                });
            });
        });
    }

    main_window.on_copy_checksum(move |checksum_str| {
        let _ = ClipboardService::set_clipboard_text(checksum_str.as_str());
    });

    main_window.on_toggle_autostart(move |enabled| {
        let _ = PlatformSystem::set_autostart(enabled);
    });

    {
        let window_weak = main_window.as_weak();
        let repo = repo.clone();
        main_window.on_finish_onboarding(move || {
            if let Some(w) = window_weak.upgrade() {
                w.set_show_onboarding(false);
                let repo = repo.clone();
                let dest = w.get_destination_path().to_string();
                let max_conns = w.get_settings_max_connections();
                let max_sim = w.get_settings_max_simultaneous();
                let auto_retry = w.get_settings_auto_retry();
                let auto_cat = w.get_settings_auto_categorize();
                let limit_enabled = w.get_settings_speed_limit_enabled();
                let limit_kb_str = w.get_settings_speed_limit_kb().to_string();
                let is_dark = w.get_is_dark();
                tokio::spawn(async move {
                    let settings = AppSettings {
                        destination_path: dest,
                        max_connections: max_conns,
                        max_simultaneous: max_sim,
                        auto_retry,
                        auto_categorize: auto_cat,
                        speed_limit_enabled: limit_enabled,
                        speed_limit_kb: limit_kb_str,
                        is_dark,
                        onboarding_completed: true,
                    };
                    let _ = repo.save_settings(&settings).await;
                });
            }
        });
    }

    // 9. Real-time Background Sync Loop (every 300ms)
    {
        let service = service.clone();
        let window_weak = main_window.as_weak();
        let search_query_cache = Arc::new(std::sync::RwLock::new(String::new()));
        let pending_settings = Arc::new(std::sync::RwLock::new(None::<AppSettings>));
        let last_saved_settings = Arc::new(std::sync::RwLock::new(None::<AppSettings>));

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(300));
            loop {
                interval.tick().await;

                // Persist settings to SQLite if changed in UI
                let maybe_to_save = {
                    let cur = pending_settings.read().unwrap().clone();
                    let last = last_saved_settings.read().unwrap().clone();
                    if cur.is_some() && cur != last {
                        cur
                    } else {
                        None
                    }
                };
                if let Some(settings_to_save) = maybe_to_save {
                    let repo = DownloadRepository::new(service.database.pool().clone());
                    if repo.save_settings(&settings_to_save).await.is_ok() {
                        *last_saved_settings.write().unwrap() = Some(settings_to_save);
                    }
                }

                // 1. Update download metrics and compute total aggregate speed
                let total_speed = service.update_metrics_and_get_total_speed().await;

                // 2. Fetch current search query from thread-safe cache
                let search_query = search_query_cache.read().unwrap().clone();

                // 3. Generate UI models asynchronously
                let downloads_ui = service.get_downloads_ui().await;
                let queue_ui = service.get_queue_ui().await;
                let history_ui = service.get_history_ui(&search_query).await;

                let mut active_count = 0;
                let mut completed_count = 0;
                for item in &downloads_ui {
                    if item.status == "downloading" || item.status == "paused" || item.status == "retrying" || item.status == "failed" {
                        active_count += 1;
                    } else if item.status == "completed" {
                        completed_count += 1;
                    }
                }
                let queued_count = queue_ui.len();

                // 4. Dispatch updates strictly onto Slint's GUI thread
                let w_weak = window_weak.clone();
                let svc = service.clone();
                let sq_cache = search_query_cache.clone();
                let pend_settings = pending_settings.clone();

                let res = slint::invoke_from_event_loop(move || {
                    if let Some(w) = w_weak.upgrade() {
                        // Read and cache search query from UI
                        *sq_cache.write().unwrap() = w.get_history_search_query().to_string();

                        // Read settings from UI and sync atomically
                        let max_conns = w.get_settings_max_connections().max(1) as u32;
                        let max_sim = w.get_settings_max_simultaneous().max(1) as usize;
                        let auto_retry = w.get_settings_auto_retry();
                        let auto_cat = w.get_settings_auto_categorize();
                        let limit_enabled = w.get_settings_speed_limit_enabled();
                        let limit_kb_str = w.get_settings_speed_limit_kb().to_string();
                        let limit_kb = limit_kb_str.trim().parse::<u64>().unwrap_or(0);
                        let is_dark = w.get_is_dark();
                        let raw_dest = w.get_destination_path().to_string();
                        let base_dest = FileCategory::strip_category_subfolder(std::path::Path::new(&raw_dest))
                            .to_string_lossy()
                            .to_string();

                        svc.max_connections.store(max_conns, std::sync::atomic::Ordering::Relaxed);
                        svc.max_simultaneous.store(max_sim, std::sync::atomic::Ordering::Relaxed);
                        svc.auto_categorize.store(auto_cat, std::sync::atomic::Ordering::Relaxed);
                        if limit_enabled && limit_kb > 0 {
                            svc.speed_limiter.set_limit(limit_kb * 1024);
                        } else {
                            svc.speed_limiter.disable();
                        }

                        *pend_settings.write().unwrap() = Some(AppSettings {
                            destination_path: base_dest,
                            max_connections: max_conns as i32,
                            max_simultaneous: max_sim as i32,
                            auto_retry,
                            auto_categorize: auto_cat,
                            speed_limit_enabled: limit_enabled,
                            speed_limit_kb: limit_kb_str,
                            is_dark,
                            onboarding_completed: !w.get_show_onboarding(),
                        });

                        // Apply model updates to UI
                        w.set_downloads(ModelRc::new(VecModel::from(downloads_ui)));
                        w.set_queue_items(ModelRc::new(VecModel::from(queue_ui)));
                        w.set_history_items(ModelRc::new(VecModel::from(history_ui)));
                        w.set_active_downloads_count(active_count);
                        w.set_completed_downloads_count(completed_count);
                        w.set_queued_downloads_count(queued_count as i32);
                        w.set_total_download_speed(SharedString::from(total_speed));
                    }
                });

                if res.is_err() {
                    break; // Event loop shut down
                }
            }
        });
    }

    // 10. Background Mode & Native Windows System Tray Integration
    {
        let notified_once = Arc::new(std::sync::atomic::AtomicBool::new(false));
        main_window.window().on_close_requested(move || {
            info!("MainWindow close requested -> hiding to System Tray (Background Mode)");
            NotificationService::set_background_mode(true);
            if !notified_once.swap(true, std::sync::atomic::Ordering::Relaxed) {
                NotificationService::show(
                    "SpeedFlow Running in Background",
                    "SpeedFlow stays active in your system tray to intercept browser downloads and finish active tasks.",
                );
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    add_popup_window.window().on_close_requested(move || {
        slint::CloseRequestResponse::HideWindow
    });

    {
        let (tray_tx, mut tray_rx) = mpsc::unbounded_channel::<TrayEvent>();
        SystemTray::spawn(tray_tx);

        let w_weak = main_window.as_weak();
        let p_weak = add_popup_window.as_weak();
        tokio::spawn(async move {
            while let Some(ev) = tray_rx.recv().await {
                match ev {
                    TrayEvent::ShowMainWindow => {
                        NotificationService::set_background_mode(false);
                        let w_weak = w_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = w_weak.upgrade() {
                                let _ = w.show();
                                #[cfg(windows)]
                                bring_window_to_foreground(
                                    "SpeedFlow - Desktop Download Manager & Accelerator",
                                );
                            }
                        });
                    }
                    TrayEvent::NotificationClicked(target) => {
                        NotificationService::set_background_mode(false);
                        let w_weak = w_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(w) = w_weak.upgrade() {
                                match target {
                                    NotificationTarget::TasksActive => {
                                        w.set_current_tab(0);
                                        w.set_active_subtab(0);
                                    }
                                    NotificationTarget::TasksCompleted => {
                                        w.set_current_tab(0);
                                        w.set_active_subtab(1);
                                    }
                                    NotificationTarget::QueueTab => {
                                        w.set_current_tab(1);
                                    }
                                }
                                let _ = w.show();
                                #[cfg(windows)]
                                bring_window_to_foreground(
                                    "SpeedFlow - Desktop Download Manager & Accelerator",
                                );
                            }
                        });
                    }
                    TrayEvent::ShowAddPopup => {
                        let p_weak = p_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(p) = p_weak.upgrade() {
                                if let Some(clip_url) = ClipboardService::get_clipboard_url() {
                                    p.set_new_url_text(clip_url.into());
                                }
                                let _ = p.show();
                                #[cfg(windows)]
                                bring_popup_to_foreground();
                            }
                        });
                    }
                    TrayEvent::Quit => {
                        info!("Quit requested from System Tray menu.");
                        let _ = slint::quit_event_loop();
                        break;
                    }
                }
            }
        });
    }

    let start_minimized = std::env::args().any(|arg| arg == "--minimized");
    NotificationService::set_background_mode(start_minimized);
    if start_minimized {
        info!("Running Slint UI Event Loop in Background Tray Mode (--minimized)...");
    } else {
        info!("Running Slint UI Event Loop (Background & System Tray enabled)...");
        main_window.show()?;
    }
    slint::run_event_loop_until_quit()?;
    info!("SpeedFlow terminated cleanly.");

    Ok(())
}
