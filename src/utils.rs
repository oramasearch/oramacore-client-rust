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
        format!("{duration_ms}ms")
    } else {
        let seconds = duration_ms as f64 / 1000.0;
        if seconds.fract() == 0.0 {
            format!("{}s", seconds as u64)
        } else {
            format!("{seconds:.1}s")
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

/// Safely parse JSON with LLM response fixing
pub fn safe_json_parse<T>(data: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    // First try direct parsing
    match serde_json::from_str::<T>(data) {
        Ok(parsed) => Ok(parsed),
        Err(_) => {
            // If direct parsing fails, try to fix the JSON with llm_json
            let fixed_json = llm_json::repair_json(data, &Default::default())
                .map_err(|e| format!("Failed to fix malformed JSON: {e}"))?;

            // Try parsing the fixed JSON
            serde_json::from_str::<T>(&fixed_json)
                .map_err(|e| format!("Failed to parse even after JSON fixing: {e}").into())
        }
    }
}

/// Parse potentially malformed JSON from AI responses
pub fn parse_ai_response<T>(data: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    safe_json_parse(data)
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
