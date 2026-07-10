use super::{API_SECRET, API_VERSION};
use std::time::{SystemTime, UNIX_EPOCH};

/// JM API authentication
#[derive(Debug, Clone)]
pub struct JmAuth {
    pub ts: String,
    pub token: String,
    pub tokenparam: String,
}

impl JmAuth {
    /// Create auth for regular API calls (millisecond timestamp)
    pub fn new() -> Self {
        let ts = current_millis_timestamp();
        Self {
            token: md5_hex(&format!("{ts}{API_VERSION}")),
            tokenparam: format!("{ts},{API_VERSION}"),
            ts,
        }
    }
}

impl Default for JmAuth {
    fn default() -> Self {
        Self::new()
    }
}

fn current_millis_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
