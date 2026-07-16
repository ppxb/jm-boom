use super::{EndpointManager, FALLBACK_ENDPOINTS};
use crate::jm::{JmClient, JmError, JmResult};
use std::{collections::HashSet, future::Future, pin::Pin, time::Instant};

const MAX_REQUEST_ENDPOINT_ATTEMPTS: usize = 4;

impl EndpointManager {
    pub(super) async fn request_candidates(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        let mut candidates = Vec::new();

        push_unique(&mut candidates, &inner.current_endpoint);

        if inner.probe_completed {
            for probe in inner.endpoints.iter().filter(|probe| probe.available) {
                push_unique(&mut candidates, &probe.endpoint);
            }
        }

        for endpoint in FALLBACK_ENDPOINTS {
            push_unique(&mut candidates, endpoint);
        }
        candidates
    }
}

pub async fn request_with_failover<T, F>(
    jm: &JmClient,
    endpoints: &EndpointManager,
    operation: F,
) -> JmResult<(String, T)>
where
    F: for<'a> Fn(&'a JmClient, &'a str) -> Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>,
{
    let mut attempted = HashSet::new();
    let mut last_error = None;

    for _ in 0..2 {
        for endpoint in endpoints.request_candidates().await {
            if attempted.len() >= MAX_REQUEST_ENDPOINT_ATTEMPTS {
                break;
            }
            if !attempted.insert(endpoint.clone()) {
                continue;
            }

            let started = Instant::now();
            match operation(jm, &endpoint).await {
                Ok(value) => {
                    endpoints
                        .report_success(&endpoint, started.elapsed().as_millis() as u64)
                        .await;
                    return Ok((endpoint, value));
                }
                Err(error) if error.is_retryable() => {
                    tracing::warn!(endpoint, %error, "endpoint request failed, trying next candidate");
                    endpoints
                        .report_failure(&endpoint, &error.to_string())
                        .await;
                    last_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        if attempted.len() >= MAX_REQUEST_ENDPOINT_ATTEMPTS {
            break;
        }
    }

    Err(last_error.unwrap_or(JmError::Empty))
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|current| current == value) {
        values.push(value.to_string());
    }
}
