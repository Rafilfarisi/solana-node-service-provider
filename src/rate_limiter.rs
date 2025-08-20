use std::time::{Duration, Instant};
use dashmap::DashMap;
use uuid::Uuid;

pub struct RateLimiter {
    max_requests_per_second: u32,
    requests: DashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests_per_second: u32) -> Self {
        Self {
            max_requests_per_second,
            requests: DashMap::new(),
        }
    }
    
    pub async fn check_rate_limit(&self) -> bool {
        let client_id = Uuid::new_v4().to_string();
        let now = Instant::now();
        let window_start = now - Duration::from_secs(1);
        
        // Get or create request history for this client
        let mut requests = self.requests
            .entry(client_id.clone())
            .or_insert_with(Vec::new);
        
        // Remove old requests outside the 1-second window
        requests.retain(|&timestamp| timestamp >= window_start);
        
        // Check if we're under the limit
        if requests.len() < self.max_requests_per_second as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
    
    pub fn cleanup_old_entries(&self) {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(1);
        
        self.requests.retain(|_, requests| {
            requests.retain(|&timestamp| timestamp >= window_start);
            !requests.is_empty()
        });
    }
}
