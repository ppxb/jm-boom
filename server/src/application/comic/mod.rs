mod comments;
mod home;

pub(crate) use comments::ComicComments;
pub(crate) use home::{
    HomeFeed, HomeSectionList, HomeSectionMode, HomeSectionRequest, WeekFilters, WeekItems,
};

use crate::{
    domain::{comic::ComicDetail, reader::ChapterManifest},
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

    pub async fn get_comic_detail(&self, comic_id: String) -> JmResult<ComicDetail> {
        self.with_failover(move |client, endpoint| {
            let comic_id = comic_id.clone();
            Box::pin(async move { client.get_comic_detail(endpoint, &comic_id).await })
        })
        .await
    }

    pub async fn get_chapter(&self, chapter_id: String) -> JmResult<ChapterManifest> {
        self.with_failover(move |client, endpoint| {
            let chapter_id = chapter_id.clone();
            Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
        })
        .await
    }

    async fn image_host(&self) -> Option<String> {
        self.with_failover(|client, endpoint| Box::pin(client.get_img_host(endpoint)))
            .await
            .ok()
    }

    async fn with_failover<T, F>(&self, operation: F) -> JmResult<T>
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
}
