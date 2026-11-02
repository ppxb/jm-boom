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
async fn cooling_current_and_builtin_endpoints_are_not_retried_by_new_requests() {
    let manager = test_manager().await;
    let failed = FALLBACK_ENDPOINTS[0];
    manager.report_failure(failed, "timeout").await;
    assert!(!manager
        .request_candidates()
        .await
        .contains(&failed.to_string()));
    manager.report_success(failed, 10).await;
    assert!(manager
        .request_candidates()
        .await
        .contains(&failed.to_string()));
}

#[tokio::test]
async fn recovered_manual_endpoint_regains_priority_after_a_valid_setting_probe() {
    use std::time::{Duration, Instant};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let host = "https://images.example";
    let app = axum::Router::new().route(
        "/setting",
        axum::routing::get(|| async {
            axum::Json(
                serde_json::json!({"code": 200, "data": {"img_host": "https://images.example"}}),
            )
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let mut manager = test_manager().await;
    manager.client = reqwest::Client::builder().no_proxy().build().unwrap();
    {
        let mut inner = manager.inner.write().await;
        inner.mode = EndpointMode::Manual;
        inner.selected_endpoint = Some(endpoint.clone());
        inner.current_endpoint = "https://fallback.example".into();
        inner.endpoints = vec![
            unavailable(&endpoint),
            available("https://fallback.example", 1),
        ];
        inner.image_hosts = vec![host.to_string()];
        inner.api_health.insert(
            endpoint.clone(),
            super::health::EndpointHealth::fail(None, Instant::now() - Duration::from_secs(31)),
        );
    }
    manager.recover_cooling_endpoints().await;
    let state = manager.state().await;
    assert_eq!(state.current_endpoint, endpoint);
    assert_eq!(state.selected_endpoint.as_deref(), Some(endpoint.as_str()));
    assert!(state.endpoints[0].available);
    assert!(manager.inner.read().await.api_health.is_empty());
    // A late response from the temporary fallback must not override the manual choice.
    manager.report_success("https://fallback.example", 1).await;
    assert_eq!(manager.state().await.current_endpoint, endpoint);
    server.abort();
}

#[tokio::test]
async fn http_responses_follow_the_expected_failover_policy() {
    let (fallback, fallback_server) = mock_endpoint(200, r#"{"code":200,"data":{"id":123}}"#).await;
    let mut jm = JmClient::new().expect("create JM client");
    jm.client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    for (status, body, should_retry) in [
        (502, "<html>Bad Gateway</html>", true),
        (503, r#"{"code":503,"errorMsg":"unavailable"}"#, true),
        (408, "", true),
        (429, r#"{"errorMsg":"rate limited"}"#, true),
        (400, r#"{"errorMsg":"invalid request"}"#, false),
        (401, "", false),
        (200, r#"{"code":400,"errorMsg":"invalid password"}"#, false),
    ] {
        let (first, first_server) = mock_endpoint(status, body).await;
        let manager = test_manager().await;
        {
            let mut inner = manager.inner.write().await;
            inner.probe_completed = true;
            inner.current_endpoint = first.clone();
            inner.endpoints = vec![available(&first, 1), available(&fallback, 2)];
        }

        let result = request_with_failover(&jm, &manager, |client, endpoint| {
            Box::pin(async move { client.get::<serde_json::Value>(endpoint, "test", &[]).await })
        })
        .await;
        first_server.abort();

        if should_retry {
            let (endpoint, payload) = result.expect("fail over after transient HTTP failure");
            assert_eq!(endpoint, fallback);
            assert_eq!(payload, serde_json::json!({"id": 123}));
            assert!(!manager.state().await.endpoints[0].available);
        } else {
            let error = result.expect_err("return permanent error without failover");
            assert!(matches!(error, JmError::Api(_)), "{status}: {error:?}");
            assert!(!error.is_retryable());
            assert_eq!(manager.state().await.current_endpoint, first);
            assert!(manager.state().await.endpoints[0].available);
        }
    }
    fallback_server.abort();
}

async fn mock_endpoint(status: u16, body: &'static str) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let status = axum::http::StatusCode::from_u16(status).unwrap();
    let app = axum::Router::new().route(
        "/test",
        axum::routing::get(move || async move { (status, body) }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (endpoint, server)
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
        image_hosts: Vec::new(),
        api_health: Default::default(),
        image_health: Default::default(),
        image_cursor: 0,
    }
}

pub(super) async fn test_manager() -> EndpointManager {
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
