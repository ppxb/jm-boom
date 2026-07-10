use super::{API_SECRET, API_VERSION};
use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp signature required by anonymous JM API requests.
#[derive(Debug, Clone)]
pub struct JmRequestSignature {
    pub ts: String,
    pub token: String,
    pub tokenparam: String,
}

impl JmRequestSignature {
    pub fn new() -> Self {
        let ts = current_millis_timestamp();
        Self {
            token: md5_hex(&format!("{ts}{API_VERSION}")),
            tokenparam: format!("{ts},{API_VERSION}"),
            ts,
        }
    }
}

impl Default for JmRequestSignature {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SettingRequestSignature {
    pub ts: String,
    pub token: String,
    pub tokenparam: String,
}

impl SettingRequestSignature {
    pub fn current() -> Self {
        let ts = current_seconds_timestamp();
        Self {
            token: md5_hex(&format!("{ts}{API_SECRET}")),
            tokenparam: format!("{ts},{API_VERSION}"),
            ts,
        }
    }
}

fn current_millis_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn current_seconds_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}
