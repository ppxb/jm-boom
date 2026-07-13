use super::{
    manager::DownloadManager, DownloadError, DownloadResult, DOWNLOAD_PAGE_CONCURRENCY_PER_TASK,
};
use crate::{
    image_work::ImageWorkPriority,
    page_materializer::{PageMaterializeError, PageMaterializeRequest},
};
use std::{sync::Arc, time::Instant};
use tokio::{sync::watch, task::JoinSet};

#[derive(Clone)]
struct WorkerContext {
    task_id: Arc<str>,
    generation: u64,
    cancelled: watch::Receiver<bool>,
}

struct PageJob {
    comic_id: u32,
    chapter_id: String,
    page: usize,
    image_path: String,
}

impl DownloadManager {
    pub(super) async fn run_task(
        &self,
        task_id: &str,
        generation: u64,
        cancelled: &watch::Receiver<bool>,
    ) -> DownloadResult<()> {
        let Some((chapters, task_snapshot)) = self.begin_generation(task_id, generation).await?
        else {
            return Ok(());
        };
        let context = WorkerContext {
            task_id: Arc::from(task_id),
            generation,
            cancelled: cancelled.clone(),
        };
        let started = Instant::now();
        let mut downloaded_bytes = 0u64;
        let mut resolved_chapters = Vec::with_capacity(chapters.len());

        for chapter_request in chapters {
            if !self.is_context_active(&context).await {
                return Ok(());
            }
            self.update_chapter(task_id, generation, &chapter_request.title)
                .await?;
            let request_chapter_id = chapter_request.chapter_id.clone();
            let (_, chapter) = self
                .jm_request(move |client, endpoint| {
                    let chapter_id = request_chapter_id.clone();
                    Box::pin(async move { client.get_chapter(endpoint, &chapter_id).await })
                })
                .await?;
            if !self.is_context_active(&context).await {
                return Ok(());
            }
            self.repository
                .persist_manifest(&task_snapshot, &chapter_request, &chapter.images)
                .await?;
            resolved_chapters.push((chapter_request, chapter));
        }

        let total_pages = resolved_chapters
            .iter()
            .map(|(_, chapter)| chapter.images.len() as u32)
            .sum();
        self.set_total_pages(task_id, generation, total_pages)
            .await?;

        for (chapter_request, chapter) in resolved_chapters {
            if !self.is_context_active(&context).await {
                return Ok(());
            }
            let comic_id = chapter_request
                .chapter_id
                .parse::<u32>()
                .map_err(anyhow::Error::from)?;
            self.update_chapter(task_id, generation, &chapter_request.title)
                .await?;
            let mut jobs = JoinSet::new();
            let mut next_page = 0usize;

            while next_page < chapter.images.len() || !jobs.is_empty() {
                while next_page < chapter.images.len()
                    && jobs.len() < DOWNLOAD_PAGE_CONCURRENCY_PER_TASK
                {
                    let manager = self.clone();
                    let context = context.clone();
                    let job = PageJob {
                        comic_id,
                        chapter_id: chapter_request.chapter_id.clone(),
                        page: next_page,
                        image_path: chapter.images[next_page].clone(),
                    };
                    jobs.spawn(async move { manager.process_page_job(&context, job).await });
                    next_page += 1;
                }

                if let Some(result) = jobs.join_next().await {
                    let result = result.map_err(anyhow::Error::from)??;
                    if let Some(size) = result {
                        downloaded_bytes = downloaded_bytes.saturating_add(size);
                        self.complete_page(
                            task_id,
                            generation,
                            downloaded_bytes,
                            started.elapsed(),
                        )
                        .await?;
                    }
                }
            }
            if !self.is_context_active(&context).await {
                return Ok(());
            }
            self.repository
                .complete_manifest(task_id, &chapter_request.chapter_id)
                .await?;
        }

        if self.is_context_active(&context).await {
            self.complete_generation(task_id, generation).await?;
        }
        Ok(())
    }

    async fn process_page_job(
        &self,
        context: &WorkerContext,
        job: PageJob,
    ) -> DownloadResult<Option<u64>> {
        if !self.is_context_active(context).await {
            return Ok(None);
        }

        let offline = self
            .storage
            .read_page(&context.task_id, &job.chapter_id, job.page)
            .await?;
        let was_offline = offline.is_some();
        let page_image = match offline {
            Some(offline) => offline,
            None => match self
                .page_materializer
                .materialize(PageMaterializeRequest {
                    chapter_id: &job.chapter_id,
                    page: job.page,
                    comic_id: job.comic_id,
                    image_path: Some(&job.image_path),
                    priority: ImageWorkPriority::Download,
                    cancelled: Some(context.cancelled.clone()),
                })
                .await
            {
                Ok(page) => page,
                Err(PageMaterializeError::Cancelled) => return Ok(None),
                Err(PageMaterializeError::Upstream(error)) => {
                    return Err(DownloadError::Upstream(error));
                }
                Err(PageMaterializeError::Internal(error)) => {
                    return Err(DownloadError::Internal(error));
                }
                Err(PageMaterializeError::PageNotFound) => {
                    return Err(DownloadError::Internal(anyhow::anyhow!(
                        "下载章节页面索引超出范围"
                    )));
                }
            },
        };

        if !self.is_context_active(context).await {
            return Ok(None);
        }
        if !was_offline {
            self.storage
                .store_page(&context.task_id, &job.chapter_id, job.page, &page_image)
                .await?;
        }
        Ok(Some(page_image.data.len() as u64))
    }

    async fn is_context_active(&self, context: &WorkerContext) -> bool {
        self.is_generation_active(&context.task_id, context.generation, &context.cancelled)
            .await
    }
}
