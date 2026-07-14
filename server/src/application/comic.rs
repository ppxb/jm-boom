use crate::{
    endpoint::{request_with_failover, EndpointManager},
    jm::{JmClient, JmResult},
};
use std::{future::Future, pin::Pin, sync::Arc};

#[derive(Clone)]
pub struct ComicService {
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
}

impl ComicService {
    pub fn new(jm: Arc<JmClient>, endpoints: Arc<EndpointManager>) -> Self {
        Self { jm, endpoints }
    }

    pub async fn request<T, F>(&self, operation: F) -> JmResult<T>
    where
        F: for<'a> Fn(
            &'a JmClient,
            &'a str,
        ) -> Pin<Box<dyn Future<Output = JmResult<T>> + Send + 'a>>,
    {
        request_with_failover(&self.jm, &self.endpoints, operation)
            .await
            .map(|(_, value)| value)
    }

    pub async fn img_host(&self) -> Option<String> {
        self.request(|client, endpoint| Box::pin(client.get_img_host(endpoint)))
            .await
            .ok()
    }
}
