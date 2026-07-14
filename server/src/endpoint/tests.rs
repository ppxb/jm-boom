use super::{
    discovery::normalize_endpoint, request_with_failover, select_current_endpoint, EndpointInner,
    EndpointManager, EndpointMode, EndpointProbe, FALLBACK_ENDPOINTS,
};
use crate::jm::{JmClient, JmError, JmResult};
use sqlx::sqlite::SqlitePoolOptions;
use std::{future::Future, pin::Pin, sync::Arc};

#[test]
fn auto_mode_selects_the_fastest_available_endpoint() {
    let mut inner = test_inner(EndpointMode::Auto);
    inner.endpoints = vec![
        available("https://slow.example", 80),
        unavailable("https://down.example"),
        available("https://fast.example", 12),
    ];

    select_current_endpoint(&mut inner);

    assert_eq!(inner.current_endpoint, "https://fast.example");
}

#[test]
fn manual_selection_is_preserved_until_explicitly_unavailable() {
    let mut inner = test_inner(EndpointMode::Manual);
    inner.selected_endpoint = Some("https://manual.example".to_string());
    inner.endpoints = vec![available("https://fast.example", 5)];

    select_current_endpoint(&mut inner);
    assert_eq!(inner.current_endpoint, "https://manual.example");

    inner.endpoints.push(unavailable("https://manual.example"));
    select_current_endpoint(&mut inner);
    assert_eq!(inner.current_endpoint, "https://fast.example");
}

#[test]
fn failed_probe_set_does_not_replace_the_current_endpoint() {
    let mut inner = test_inner(EndpointMode::Auto);
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
async fn request_candidates_keep_manual_selection_first_after_probe() {
    let manager = test_manager().await;
    {
        let mut inner = manager.inner.write().await;
        inner.mode = EndpointMode::Manual;
        inner.probe_completed = true;
        inner.selected_endpoint = Some("https://manual.example".to_string());
        inner.current_endpoint = "https://fast.example".to_string();
        inner.endpoints = vec![
            available("https://fast.example", 1),
            available("https://manual.example", 20),
        ];
    }

    let candidates = manager.request_candidates().await;
    assert_eq!(candidates[0], "https://manual.example");
    assert_eq!(candidates[1], "https://fast.example");
    assert_eq!(
        &candidates[candidates.len() - 2..],
        &FALLBACK_ENDPOINTS.map(str::to_string)
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

#[tokio::test]
async fn selected_endpoint_round_trips_through_repository() {
    let db = test_db().await;
    let manager = EndpointManager::new(db.clone())
        .await
        .expect("create endpoint manager");
    {
        manager
            .inner
            .write()
            .await
            .endpoints
            .push(available("https://manual.example", 10));
    }

    manager
        .set_selected(Some("manual.example/".to_string()))
        .await
        .expect("persist selected endpoint");
    let restored = EndpointManager::new(db)
        .await
        .expect("restore endpoint manager")
        .state()
        .await;

    assert_eq!(restored.mode, EndpointMode::Manual);
    assert_eq!(
        restored.selected_endpoint.as_deref(),
        Some("https://manual.example")
    );
    assert_eq!(restored.current_endpoint, "https://manual.example");
}

#[test]
fn normalizes_only_https_origins() {
    assert_eq!(
        normalize_endpoint(" example.com/ ").expect("normalize endpoint"),
        "https://example.com"
    );
    assert!(normalize_endpoint("http://example.com").is_err());
    assert!(normalize_endpoint("https://example.com/path").is_err());
}

fn test_inner(mode: EndpointMode) -> EndpointInner {
    EndpointInner {
        mode,
        probe_completed: false,
        selected_endpoint: None,
        current_endpoint: FALLBACK_ENDPOINTS[0].to_string(),
        endpoints: Vec::new(),
    }
}

async fn test_manager() -> EndpointManager {
    EndpointManager::new(test_db().await)
        .await
        .expect("create endpoint manager")
}

async fn test_db() -> sqlx::SqlitePool {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("run migrations");
    db
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
