use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_retries {
            return None;
        }

        let delay_ms = (self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let delay = Duration::from_millis(delay_ms).min(self.max_delay);
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_progression() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(500)));
        assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(1000)));
        assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(2000)));
        assert_eq!(policy.delay_for_attempt(5), None);
    }
}
