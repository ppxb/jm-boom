use super::EndpointProbe;
use crate::jm::SettingRequestSignature;
use reqwest::Client;
use std::time::Instant;

pub(super) async fn probe_endpoint(client: &Client, endpoint: String) -> EndpointProbe {
    let signature = SettingRequestSignature::current();
    let started = Instant::now();
    let result = client
        .get(format!("{endpoint}/setting"))
        .header("token", signature.token)
        .header("tokenparam", signature.tokenparam)
        .query(&[("app_img_shunt", "1"), ("t", signature.ts.as_str())])
        .send()
        .await
        .and_then(reqwest::Response::error_for_status);

    match result {
        Ok(_) => EndpointProbe {
            endpoint,
            available: true,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            error: None,
        },
        Err(error) => EndpointProbe {
            endpoint,
            available: false,
            latency_ms: None,
            error: Some(error.to_string()),
        },
    }
}
