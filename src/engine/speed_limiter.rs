use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct SpeedLimiter {
    enabled: Arc<AtomicBool>,
    max_bytes_per_sec: Arc<AtomicU64>,
}

impl SpeedLimiter {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            max_bytes_per_sec: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_limit(&self, bytes_per_sec: u64) {
        if bytes_per_sec == 0 {
            self.enabled.store(false, Ordering::Relaxed);
        } else {
            self.max_bytes_per_sec.store(bytes_per_sec, Ordering::Relaxed);
            self.enabled.store(true, Ordering::Relaxed);
        }
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub async fn throttle(&self, bytes_transferred: usize, elapsed: Duration) {
        if !self.is_enabled() {
            return;
        }

        let max_bps = self.max_bytes_per_sec.load(Ordering::Relaxed);
        if max_bps == 0 {
            return;
        }

        let target_duration = Duration::from_secs_f64(bytes_transferred as f64 / max_bps as f64);
        if target_duration > elapsed {
            let sleep_time = target_duration - elapsed;
            sleep(sleep_time).await;
        }
    }
}
