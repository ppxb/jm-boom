use once_cell::sync::OnceCell;
use std::sync::RwLock;

static ENDPOINT_CONFIG: OnceCell<RwLock<EndpointConfig>> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct EndpointConfig {
    pub current: String,
    pub candidates: Vec<String>,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            current: "https://www.cdnhjk.net".to_string(),
            candidates: vec![
                "https://www.cdnhjk.net".to_string(),
                "https://www.jmapinode1.cc".to_string(),
                "https://www.jmapinodel.com".to_string(),
            ],
        }
    }
}

/// Get current endpoint
pub fn get_endpoint() -> String {
    let config = ENDPOINT_CONFIG.get_or_init(|| RwLock::new(EndpointConfig::default()));
    config
        .read()
        .map(|c| c.current.clone())
        .unwrap_or_else(|_| "https://www.cdnhjk.net".to_string())
}

/// Set current endpoint
pub fn set_endpoint(endpoint: String) -> anyhow::Result<()> {
    let config = ENDPOINT_CONFIG.get_or_init(|| RwLock::new(EndpointConfig::default()));
    let mut guard = config
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
    guard.current = endpoint;
    Ok(())
}

/// Get all candidate endpoints
pub fn get_candidates() -> Vec<String> {
    let config = ENDPOINT_CONFIG.get_or_init(|| RwLock::new(EndpointConfig::default()));
    config
        .read()
        .map(|c| c.candidates.clone())
        .unwrap_or_default()
}
