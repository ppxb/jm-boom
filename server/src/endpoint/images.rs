use super::{health::EndpointHealth, request_with_failover, EndpointManager};
use crate::{
    jm::{download_image_bytes, invalidate_img_host, JmClient, JmError, JmResult},
    reader::PageImageFormat,
};
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};
use tokio::task::JoinSet;

const IMAGE_PROBE_PATH: &str = "/media/users/nopic-.gif?v=0";
const IMAGE_PROBE_MAX_BYTES: usize = 1024 * 1024;
const IMAGE_PROBE_CONCURRENCY: usize = 6;
const MAX_IMAGE_HOST_ATTEMPTS: usize = 4;

impl EndpointManager {
    pub(super) async fn refresh_image_hosts(&self, mut hosts: Vec<String>) {
        let mut seen = HashSet::new();
        hosts.retain(|host| seen.insert(host.clone()));
        let mut available: Vec<_> = probe_image_hosts(&self.client, hosts)
            .await
            .into_iter()
            .filter_map(|(host, latency)| latency.map(|latency| (host, latency)))
            .collect();
        available.sort_by_key(|(_, latency)| *latency);
        if available.is_empty() {
            return;
        }
        let mut inner = self.inner.write().await;
        let mut hosts: Vec<_> = available.into_iter().map(|(host, _)| host).collect();
        for host in &hosts {
            inner.image_health.remove(host);
        }
        // Keep cooling hosts so a transient failure can recover without rediscovery.
        for host in &inner.image_hosts {
            if inner.image_health.contains_key(host) && !hosts.contains(host) {
                hosts.push(host.clone());
            }
        }
        inner.image_hosts = hosts;
    }

    async fn image_candidates(&self) -> Vec<String> {
        let mut inner = self.inner.write().await;
        let now = Instant::now();
        let mut hosts: Vec<_> = inner
            .image_hosts
            .iter()
            .filter(|host| {
                !inner
                    .image_health
                    .get(*host)
                    .is_some_and(|health| health.cooling(now))
            })
            .cloned()
            .collect();
        if !hosts.is_empty() {
            let offset = inner.image_cursor % hosts.len();
            inner.image_cursor = inner.image_cursor.wrapping_add(1);
            hosts.rotate_left(offset);
        }
        hosts
    }

    pub(super) async fn recover_image_hosts(&self) {
        let now = Instant::now();
        let due: HashMap<_, _> = self
            .inner
            .read()
            .await
            .image_health
            .iter()
            .filter(|(_, health)| !health.cooling(now))
            .map(|(host, health)| (host.clone(), health.clone()))
            .collect();
        for (host, latency) in probe_image_hosts(&self.client, due.keys().cloned().collect()).await
        {
            let mut inner = self.inner.write().await;
            if inner.image_health.get(&host) != due.get(&host) {
                continue;
            }
            if latency.is_some() {
                inner.image_health.remove(&host);
            } else {
                let health = EndpointHealth::fail(inner.image_health.get(&host), Instant::now());
                inner.image_health.insert(host, health);
            }
        }
    }

    async fn download_from_host(&self, jm: &JmClient, host: &str, path: &str) -> JmResult<Vec<u8>> {
        let result = jm.download_image(&format!("{host}{path}")).await;
        match &result {
            Ok(_) => {
                self.inner.write().await.image_health.remove(host);
            }
            Err(error) if error.is_retryable() => {
                tracing::warn!(image_host = host, %error, "image request failed, cooling down host");
                let mut inner = self.inner.write().await;
                let health = EndpointHealth::fail(inner.image_health.get(host), Instant::now());
                inner.image_health.insert(host.to_string(), health);
            }
            Err(_) => {}
        }
        result
    }

    pub(crate) async fn download_image(&self, jm: &JmClient, path: &str) -> JmResult<Vec<u8>> {
        let mut initial_endpoint = None;
        if self.inner.read().await.image_hosts.is_empty() {
            let (endpoint, host) = request_with_failover(jm, self, |client, endpoint| {
                Box::pin(client.get_img_host(endpoint))
            })
            .await?;
            initial_endpoint = Some(endpoint);
            let mut inner = self.inner.write().await;
            if !inner.image_hosts.contains(&host) {
                inner.image_hosts.push(host);
            }
        }
        let mut last_error = None;
        let mut attempted = HashSet::new();
        for host in self
            .image_candidates()
            .await
            .into_iter()
            .take(MAX_IMAGE_HOST_ATTEMPTS)
        {
            attempted.insert(host.clone());
            match self.download_from_host(jm, &host, path).await {
                Ok(data) => return Ok(data),
                Err(error) if error.is_retryable() => {
                    last_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        // The first setting may have advertised a stale image host before background probes finish.
        if let Some(endpoint) =
            initial_endpoint.filter(|_| attempted.len() < MAX_IMAGE_HOST_ATTEMPTS)
        {
            invalidate_img_host(&endpoint).await;
            let (_, host) = request_with_failover(jm, self, |client, endpoint| {
                Box::pin(client.get_img_host(endpoint))
            })
            .await?;
            if attempted.contains(&host) {
                return Err(last_error.unwrap_or(JmError::Empty));
            }
            {
                let mut inner = self.inner.write().await;
                if !inner.image_hosts.contains(&host) {
                    inner.image_hosts.push(host.clone());
                }
            }
            return self.download_from_host(jm, &host, path).await;
        }
        Err(last_error.unwrap_or(JmError::Empty))
    }
}

async fn probe_image_hosts(
    client: &reqwest::Client,
    hosts: Vec<String>,
) -> Vec<(String, Option<u128>)> {
    let mut hosts = hosts.into_iter();
    let mut tasks = JoinSet::new();
    let mut results = Vec::new();
    loop {
        while tasks.len() < IMAGE_PROBE_CONCURRENCY {
            let Some(host) = hosts.next() else { break };
            let client = client.clone();
            tasks.spawn(async move {
                let result = probe_image_host(&client, host.clone()).await;
                (host, result.map(|(_, latency)| latency))
            });
        }
        match tasks.join_next().await {
            Some(Ok(result)) => results.push(result),
            Some(Err(_)) => {}
            None => break,
        }
    }
    results
}

async fn probe_image_host(client: &reqwest::Client, host: String) -> Option<(String, u128)> {
    let started = Instant::now();
    let url = format!("{host}{IMAGE_PROBE_PATH}");
    let data = match download_image_bytes(client, &url, IMAGE_PROBE_MAX_BYTES).await {
        Ok(data) => data,
        Err(error) => {
            tracing::debug!(image_host = %host, %error, "image host probe failed");
            return None;
        }
    };
    if !PageImageFormat::detect(&data).is_ok_and(|format| format.is_complete(&data)) {
        tracing::debug!(image_host = %host, "image host probe returned an invalid image");
        return None;
    }
    Some((host, started.elapsed().as_millis()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::{tests::test_manager, EndpointMode};
    use axum::{
        http::{StatusCode, Uri},
        routing::get,
        Router,
    };
    use std::{
        io::Cursor,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    fn image() -> Vec<u8> {
        let mut data = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut data, image::ImageFormat::Png)
            .unwrap();
        data.into_inner()
    }

    fn jm() -> JmClient {
        let mut jm = JmClient::new().unwrap();
        jm.client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        jm
    }

    async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (host, server)
    }

    async fn image_server(
        status: StatusCode,
        body: Vec<u8>,
    ) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
        let requests = Arc::new(AtomicUsize::new(0));
        let (host, server) = serve(Router::new().fallback({
            let requests = requests.clone();
            move |uri: Uri| {
                assert!(uri.path().starts_with("/media/"));
                requests.fetch_add(1, Ordering::SeqCst);
                let body = body.clone();
                async move { (status, body) }
            }
        }))
        .await;
        (host, requests, server)
    }

    #[tokio::test]
    async fn probes_require_complete_images_and_keep_the_pool_on_total_failure() {
        let mut manager = test_manager().await;
        manager.client = jm().client;
        let (good, good_requests, good_server) = image_server(StatusCode::OK, image()).await;
        let mut truncated = image();
        truncated.truncate(truncated.len() - 4);
        for body in [
            b"<html>blocked</html>".to_vec(),
            Vec::new(),
            truncated,
            vec![0; IMAGE_PROBE_MAX_BYTES + 1],
        ] {
            let (bad, _, bad_server) = image_server(StatusCode::OK, body).await;
            manager
                .refresh_image_hosts(vec![bad.clone(), good.clone()])
                .await;
            assert_eq!(manager.inner.read().await.image_hosts, vec![good.clone()]);
            manager.refresh_image_hosts(vec![bad]).await;
            assert_eq!(manager.inner.read().await.image_hosts, vec![good.clone()]);
            bad_server.abort();
        }
        assert_eq!(good_requests.load(Ordering::SeqCst), 4);
        good_server.abort();
    }

    #[tokio::test]
    async fn distributes_page_and_cover_requests_across_healthy_hosts() {
        let manager = test_manager().await;
        let (first, first_requests, first_server) = image_server(StatusCode::OK, image()).await;
        let (second, second_requests, second_server) = image_server(StatusCode::OK, image()).await;
        manager.inner.write().await.image_hosts = vec![first, second];
        let jm = jm();
        for _ in 0..3 {
            manager
                .download_image(&jm, "/media/photos/123/001.png")
                .await
                .unwrap();
            manager
                .download_image(&jm, "/media/albums/123_3x4.jpg")
                .await
                .unwrap();
        }
        assert_eq!(first_requests.load(Ordering::SeqCst), 3);
        assert_eq!(second_requests.load(Ordering::SeqCst), 3);
        first_server.abort();
        second_server.abort();
    }

    #[tokio::test]
    async fn shared_hosts_from_different_api_endpoints_are_probed_and_scheduled_once() {
        let mut manager = test_manager().await;
        manager.client = jm().client;
        let (host, requests, server) = image_server(StatusCode::OK, image()).await;
        manager
            .refresh_image_hosts(vec![host.clone(), host.clone()])
            .await;
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        assert_eq!(manager.image_candidates().await, vec![host]);
        server.abort();
    }

    #[tokio::test]
    async fn expired_image_cooldown_is_reprobed_and_recovers_without_api_changes() {
        let mut manager = test_manager().await;
        manager.client = jm().client;
        let (host, requests, server) = image_server(StatusCode::OK, image()).await;
        {
            let mut inner = manager.inner.write().await;
            inner.image_hosts = vec![host.clone()];
            inner
                .image_health
                .insert(host.clone(), EndpointHealth::fail(None, Instant::now()));
        }
        assert!(manager.image_candidates().await.is_empty());
        manager.recover_cooling_endpoints().await;
        assert_eq!(requests.load(Ordering::SeqCst), 0);
        let before = manager.state().await;
        manager.inner.write().await.image_health.insert(
            host.clone(),
            EndpointHealth::fail(None, Instant::now() - Duration::from_secs(31)),
        );
        manager.recover_cooling_endpoints().await;
        assert!(manager.inner.read().await.image_health.is_empty());
        assert_eq!(manager.image_candidates().await, vec![host]);
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        assert_eq!(
            manager.state().await.current_endpoint,
            before.current_endpoint
        );
        server.abort();
    }

    #[tokio::test]
    async fn image_failover_is_shared_by_paths_and_does_not_change_the_api_selection() {
        let manager = test_manager().await;
        let (bad, bad_requests, bad_server) =
            image_server(StatusCode::SERVICE_UNAVAILABLE, Vec::new()).await;
        let (good, good_requests, good_server) = image_server(StatusCode::OK, image()).await;
        {
            let mut inner = manager.inner.write().await;
            inner.mode = EndpointMode::Manual;
            inner.selected_endpoint = Some(inner.current_endpoint.clone());
            inner.image_hosts = vec![bad.clone(), good.clone()];
        }
        let before = manager.state().await;
        let jm = jm();
        for path in ["/media/photos/123/001.png", "/media/albums/123_3x4.jpg"] {
            assert_eq!(manager.download_image(&jm, path).await.unwrap(), image());
        }
        assert_eq!(bad_requests.load(Ordering::SeqCst), 1);
        assert_eq!(good_requests.load(Ordering::SeqCst), 2);
        assert_eq!(
            manager.inner.read().await.image_hosts,
            vec![bad.clone(), good]
        );
        assert!(manager.inner.read().await.image_health[&bad].cooling(Instant::now()));
        let after = manager.state().await;
        assert_eq!(before.current_endpoint, after.current_endpoint);
        assert_eq!(before.selected_endpoint, after.selected_endpoint);
        assert_eq!(after.mode, EndpointMode::Manual);
        bad_server.abort();
        good_server.abort();
    }

    #[tokio::test]
    async fn permanent_http_errors_do_not_evict_hosts_or_try_another_host() {
        let manager = test_manager().await;
        let jm = jm();
        let (good, good_requests, good_server) = image_server(StatusCode::OK, image()).await;
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::NOT_FOUND,
        ] {
            let (host, requests, server) = image_server(status, Vec::new()).await;
            let hosts = vec![host, good.clone()];
            manager.inner.write().await.image_hosts = hosts.clone();
            manager.inner.write().await.image_cursor = 0;
            let error = manager
                .download_image(&jm, "/media/photos/123/001.png")
                .await
                .unwrap_err();
            assert!(matches!(error, JmError::ImageHttp { status: actual, .. } if actual == status));
            assert!(!error.is_retryable());
            assert_eq!(manager.inner.read().await.image_hosts, hosts);
            assert_eq!(requests.load(Ordering::SeqCst), 1);
            server.abort();
        }
        assert_eq!(good_requests.load(Ordering::SeqCst), 0);
        good_server.abort();
    }

    #[tokio::test]
    async fn image_failover_stops_at_the_attempt_limit() {
        let manager = test_manager().await;
        let mut servers = Vec::new();
        let mut hosts = Vec::new();
        let mut counts = Vec::new();
        for _ in 0..MAX_IMAGE_HOST_ATTEMPTS + 1 {
            let (host, count, server) = image_server(StatusCode::BAD_GATEWAY, Vec::new()).await;
            hosts.push(host);
            counts.push(count);
            servers.push(server);
        }
        manager.inner.write().await.image_hosts = hosts.clone();
        assert!(manager
            .download_image(&jm(), "/media/photos/123/001.png")
            .await
            .is_err());
        for (index, count) in counts.iter().enumerate() {
            assert_eq!(
                count.load(Ordering::SeqCst),
                usize::from(index < MAX_IMAGE_HOST_ATTEMPTS)
            );
        }
        assert_eq!(manager.inner.read().await.image_hosts, hosts);
        assert_eq!(
            manager.inner.read().await.image_health.len(),
            MAX_IMAGE_HOST_ATTEMPTS
        );
        for server in servers {
            server.abort();
        }
    }

    #[tokio::test]
    async fn cancellation_does_not_start_failover_or_evict_the_host() {
        let manager = test_manager().await;
        let started = Arc::new(tokio::sync::Notify::new());
        let (slow, slow_server) = serve(Router::new().fallback({
            let started = started.clone();
            move || {
                let started = started.clone();
                async move {
                    started.notify_one();
                    std::future::pending::<String>().await
                }
            }
        }))
        .await;
        let (good, good_requests, good_server) = image_server(StatusCode::OK, image()).await;
        let hosts = vec![slow, good];
        manager.inner.write().await.image_hosts = hosts.clone();
        let task = tokio::spawn({
            let manager = manager.clone();
            async move {
                manager
                    .download_image(&jm(), "/media/photos/123/001.png")
                    .await
            }
        });
        tokio::time::timeout(Duration::from_secs(2), started.notified())
            .await
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(manager.inner.read().await.image_hosts, hosts);
        assert_eq!(good_requests.load(Ordering::SeqCst), 0);
        slow_server.abort();
        good_server.abort();
    }

    #[tokio::test]
    async fn requests_before_probing_can_refresh_a_changed_image_host() {
        let manager = test_manager().await;
        let (bad, bad_requests, bad_server) =
            image_server(StatusCode::SERVICE_UNAVAILABLE, Vec::new()).await;
        let (good, good_requests, good_server) = image_server(StatusCode::OK, image()).await;
        let setting_requests = Arc::new(AtomicUsize::new(0));
        let (api, api_server) = serve(Router::new().route(
            "/setting",
            get({
                let setting_requests = setting_requests.clone();
                move || {
                    let host = if setting_requests.fetch_add(1, Ordering::SeqCst) == 0 {
                        bad.clone()
                    } else {
                        good.clone()
                    };
                    async move {
                        axum::Json(serde_json::json!({ "code": 200, "data": { "img_host": host } }))
                    }
                }
            }),
        ))
        .await;
        manager.inner.write().await.current_endpoint = api;
        assert_eq!(
            manager
                .download_image(&jm(), "/media/photos/123/001.png")
                .await
                .unwrap(),
            image()
        );
        assert_eq!(setting_requests.load(Ordering::SeqCst), 2);
        assert_eq!(bad_requests.load(Ordering::SeqCst), 1);
        assert_eq!(good_requests.load(Ordering::SeqCst), 1);
        api_server.abort();
        bad_server.abort();
        good_server.abort();
    }

    #[tokio::test]
    async fn startup_discovery_race_still_attempts_at_most_four_image_hosts() {
        let manager = test_manager().await;
        let mut hosts = Vec::new();
        let mut counts = Vec::new();
        let mut servers = Vec::new();
        for _ in 0..5 {
            let (host, count, server) = image_server(StatusCode::BAD_GATEWAY, Vec::new()).await;
            hosts.push(host);
            counts.push(count);
            servers.push(server);
        }
        let started = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let setting_requests = Arc::new(AtomicUsize::new(0));
        let (api, api_server) = serve(Router::new().route(
            "/setting",
            get({
                let started = started.clone();
                let release = release.clone();
                let setting_requests = setting_requests.clone();
                let host = hosts[4].clone();
                move || {
                    let started = started.clone();
                    let release = release.clone();
                    let setting_requests = setting_requests.clone();
                    let host = host.clone();
                    async move {
                        if setting_requests.fetch_add(1, Ordering::SeqCst) == 0 {
                            started.notify_one();
                            release.acquire().await.unwrap().forget();
                        }
                        axum::Json(serde_json::json!({"code":200,"data":{"img_host":host}}))
                    }
                }
            }),
        ))
        .await;
        manager.inner.write().await.current_endpoint = api;
        let request = tokio::spawn({
            let manager = manager.clone();
            async move {
                manager
                    .download_image(&jm(), "/media/photos/123/001.png")
                    .await
            }
        });
        tokio::time::timeout(Duration::from_secs(2), started.notified())
            .await
            .unwrap();
        manager.inner.write().await.image_hosts = hosts[..4].to_vec();
        release.add_permits(1);
        assert!(request.await.unwrap().is_err());
        assert_eq!(setting_requests.load(Ordering::SeqCst), 1);
        assert_eq!(
            counts
                .iter()
                .map(|count| count.load(Ordering::SeqCst))
                .sum::<usize>(),
            4
        );
        assert_eq!(counts[4].load(Ordering::SeqCst), 0);
        api_server.abort();
        for server in servers {
            server.abort();
        }
    }
}
