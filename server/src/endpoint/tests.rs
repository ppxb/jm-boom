use super::{
    request_with_failover, select_current_endpoint, EndpointInner, EndpointManager, EndpointProbe,
    FALLBACK_ENDPOINTS,
};
use crate::jm::{JmClient, JmError, JmResult};
use std::{future::Future, pin::Pin, sync::Arc};

#[test]
fn auto_mode_selects_the_fastest_available_endpoint() {
    let mut inner = test_inner();
    inner.endpoints = vec![
        available("https://slow.example", 80),
        unavailable("https://down.example"),
        available("https://fast.example", 12),
    ];

    select_current_endpoint(&mut inner);

    assert_eq!(inner.current_endpoint, "https://fast.example");
}

#[test]
fn failed_probe_set_does_not_replace_the_current_endpoint() {
    let mut inner = test_inner();
    inner.current_endpoint = "https://working.example".to_string();
    inner.endpoints = vec![
        unavailable("https://first.example"),
        unavailable("https://second.example"),
    ];

    select_current_endpoint(&mut inner);

    assert_eq!(inner.current_endpoint, "https://working.example");
}

#[tokio::test]
async fn request_candidates_use_only_current_and_fallbacks_before_probe_completes() {
    let manager = test_manager().await;
    {
        let mut inner = manager.inner.write().await;
        inner.current_endpoint = FALLBACK_ENDPOINTS[1].to_string();
        inner
            .endpoints
            .push(available("https://discovered.example", 1));
    }

    assert_eq!(
        manager.request_candidates().await,
        vec![
            FALLBACK_ENDPOINTS[1].to_string(),
            FALLBACK_ENDPOINTS[0].to_string()
        ]
    );
}

#[tokio::test]
async fn failover_retries_the_next_available_candidate() {
    let manager = test_manager().await;
    {
        let mut inner = manager.inner.write().await;
        inner.probe_completed = true;
        inner.current_endpoint = "https://first.example".to_string();
        inner.endpoints = vec![
            available("https://first.example", 1),
            available("https://second.example", 2),
        ];
    }
    let jm = JmClient::new().expect("create JM client");
    let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));

    let result = request_with_failover(&jm, &manager, {
        let attempts = attempts.clone();
        move |_jm, endpoint| {
            let endpoint = endpoint.to_string();
            let attempts = attempts.clone();
            Box::pin(async move {
                attempts
                    .lock()
                    .expect("attempt log poisoned")
                    .push(endpoint.clone());
                if endpoint == "https://first.example" {
                    Err(JmError::Network("test failure".to_string()))
                } else {
                    Ok("success")
                }
            }) as Pin<Box<dyn Future<Output = JmResult<&'static str>> + Send>>
        }
    })
    .await
    .expect("fail over to second endpoint");

    assert_eq!(result, ("https://second.example".to_string(), "success"));
    assert_eq!(
        *attempts.lock().expect("attempt log poisoned"),
        vec![
            "https://first.example".to_string(),
            "https://second.example".to_string()
        ]
    );
}

fn test_inner() -> EndpointInner {
    EndpointInner {
        probe_completed: false,
        current_endpoint: FALLBACK_ENDPOINTS[0].to_string(),
        endpoints: Vec::new(),
    }
}

async fn test_manager() -> EndpointManager {
    EndpointManager::new()
        .await
        .expect("create endpoint manager")
}

fn available(endpoint: &str, latency_ms: u64) -> EndpointProbe {
    EndpointProbe {
        endpoint: endpoint.to_string(),
        available: true,
        latency_ms: Some(latency_ms),
        error: None,
    }
}

fn unavailable(endpoint: &str) -> EndpointProbe {
    EndpointProbe {
        endpoint: endpoint.to_string(),
        available: false,
        latency_ms: None,
        error: Some("unavailable".to_string()),
    }
}
