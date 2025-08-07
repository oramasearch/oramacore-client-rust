//! Utility functions for the Orama client.

use std::time::{Duration, Instant};
use uuid::Uuid;

/// Create a random string of specified length
pub fn create_random_string(length: usize) -> String {
    use uuid::Uuid;

    // Generate multiple UUIDs if needed to reach the desired length
    let mut result = String::new();
    while result.len() < length {
        let uuid_str = Uuid::new_v4().to_string().replace('-', "");
        result.push_str(&uuid_str);
    }

    // Truncate to exact length
    result.truncate(length);
    result
}

/// Format duration in milliseconds to human readable string
pub fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else {
        let seconds = duration_ms as f64 / 1000.0;
        if seconds.fract() == 0.0 {
            format!("{}s", seconds as u64)
        } else {
            format!("{:.1}s", seconds)
        }
    }
}

/// Get current timestamp in milliseconds
pub fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Generate a new UUID v4 as string
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Safely parse JSON with fallback
pub fn safe_json_parse<T>(data: &str) -> Result<T, serde_json::Error>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_str(data)
}

/// Throttle function execution
pub struct Throttle {
    last_called: std::sync::Mutex<Option<Instant>>,
    limit: Duration,
}

impl Throttle {
    /// Create a new throttle with the specified limit in milliseconds
    pub fn new(limit_ms: u64) -> Self {
        Self {
            last_called: std::sync::Mutex::new(None),
            limit: Duration::from_millis(limit_ms),
        }
    }

    /// Execute function if enough time has passed since last call
    pub fn execute<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce() -> R,
    {
        let mut last_called = self.last_called.lock().unwrap();
        let now = Instant::now();

        match *last_called {
            Some(last) if now.duration_since(last) < self.limit => None,
            _ => {
                *last_called = Some(now);
                Some(f())
            }
        }
    }
}

/// Debounce function execution
pub struct Debounce {
    timer: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    delay: Duration,
}

impl Debounce {
    /// Create a new debounce with the specified delay in milliseconds
    pub fn new(delay_ms: u64) -> Self {
        Self {
            timer: std::sync::Mutex::new(None),
            delay: Duration::from_millis(delay_ms),
        }
    }

    /// Execute function after delay, cancelling any previous pending execution
    pub async fn execute<F, Fut>(&self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut timer = self.timer.lock().unwrap();

        // Cancel previous timer if exists
        if let Some(handle) = timer.take() {
            handle.abort();
        }

        let delay = self.delay;
        *timer = Some(tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            f().await;
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_random_string() {
        let s1 = create_random_string(10);
        let s2 = create_random_string(10);

        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1000), "1s");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(2000), "2s");
    }

    #[test]
    fn test_generate_uuid() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID length
    }

    #[tokio::test]
    async fn test_throttle() {
        let throttle = Throttle::new(100);

        // First call should execute
        let result1 = throttle.execute(|| "first");
        assert_eq!(result1, Some("first"));

        // Immediate second call should be throttled
        let result2 = throttle.execute(|| "second");
        assert_eq!(result2, None);

        // Wait for throttle period to pass
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Third call should execute
        let result3 = throttle.execute(|| "third");
        assert_eq!(result3, Some("third"));
    }
}
