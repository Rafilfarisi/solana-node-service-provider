use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    max_requests_per_second: u32,
    window: Duration,
    timestamps: Mutex<VecDeque<Instant>>, // global timestamps within window
}

impl RateLimiter {
    pub fn new(max_requests_per_second: u32) -> Self {
        Self {
            max_requests_per_second,
            window: Duration::from_secs(1),
            timestamps: Mutex::new(VecDeque::new()),
        }
    }
    
    pub async fn check_rate_limit(&self) -> bool {
        let now = Instant::now();
        let mut q = self.timestamps.lock().expect("rate limiter mutex poisoned");

        // Evict timestamps older than window
        while let Some(&front) = q.front() {
            if now.duration_since(front) >= self.window {
                q.pop_front();
            } else {
                break;
            }
        }

        if q.len() < self.max_requests_per_second as usize {
            q.push_back(now);
            true
        } else {
            false
        }
    }
}
