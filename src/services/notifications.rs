use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

use crate::platform::{NotificationTarget, SystemTray};

static IN_BACKGROUND: AtomicBool = AtomicBool::new(false);

pub struct NotificationService;

impl NotificationService {
    pub fn set_background_mode(in_background: bool) {
        IN_BACKGROUND.store(in_background, Ordering::Release);
    }

    pub fn is_background_mode() -> bool {
        IN_BACKGROUND.load(Ordering::Acquire)
    }

    /// Shows a Windows Tray Notification unconditionally (e.g., when closing to tray for the first time).
    pub fn show(title: &str, body: &str) {
        info!("[Notification] {}: {}", title, body);
        SystemTray::show_balloon(title, body, NotificationTarget::TasksActive);
    }

    /// Shows a Windows Tray Notification ONLY when SpeedFlow's main window is in background mode.
    pub fn show_if_background(title: &str, body: &str, target: NotificationTarget) {
        if Self::is_background_mode() {
            info!("[Background Notification] {}: {}", title, body);
            SystemTray::show_balloon(title, body, target);
        }
    }

    pub fn notify_download_starting(filename: &str) {
        Self::show_if_background(
            "SpeedFlow — Starting Download",
            &format!("Starting download \"{}\"", filename),
            NotificationTarget::TasksActive,
        );
    }

    pub fn notify_download_completed(filename: &str) {
        Self::show_if_background(
            "SpeedFlow — Download Complete",
            &format!("Finished downloading \"{}\"", filename),
            NotificationTarget::TasksCompleted,
        );
    }

    pub fn notify_download_failed(filename: &str) {
        Self::show_if_background(
            "SpeedFlow — Download Failed",
            &format!("Failed to download \"{}\". Click to retry.", filename),
            NotificationTarget::TasksActive,
        );
    }

    pub fn notify_download_queued(filename: &str) {
        Self::show_if_background(
            "SpeedFlow — Added to Queue",
            &format!("\"{}\" has been added to the download queue.", filename),
            NotificationTarget::QueueTab,
        );
    }
}
