use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        // Get or create entry for this key
        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        entry.retain(|&time| now.duration_since(time) < self.window);

        // Check if under limit
        if entry.len() < self.max_requests {
            entry.push(now);
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let limiter = request
        .extensions()
        .get::<RateLimiter>()
        .cloned()
        .expect("RateLimiter extension not found");

    let key = addr.ip().to_string();

    if limiter.check(&key) {
        next.run(request).await
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            "Too many requests. Please try again later.",
        )
            .into_response()
    }
}

pub fn create_rate_limiter() -> RateLimiter {
    // 100 requests per minute
    RateLimiter::new(100, Duration::from_secs(60))
}
