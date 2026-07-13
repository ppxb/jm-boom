use crate::{cache::CachedReaderPage, reader::PageImageFormat};
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
pub(super) struct DownloadStorage {
    root: PathBuf,
}

pub(super) struct StagedTaskDeletion {
    original: PathBuf,
    staged: PathBuf,
}

impl DownloadStorage {
    pub async fn new(root: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&root).await?;
        let deleting_root = root.join(".deleting");
        match fs::remove_dir_all(&deleting_root).await {
            Ok(()) => tracing::info!("已清理下载删除暂存目录"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(%error, "清理下载删除暂存目录失败"),
        }
        Ok(Self { root })
    }

    pub async fn store_page(
        &self,
        task_id: &str,
        chapter_id: &str,
        page: usize,
        page_image: &CachedReaderPage,
    ) -> anyhow::Result<()> {
        let path = self.page_path(task_id, chapter_id, page, page_image.format.extension());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, &page_image.data).await?;
        Ok(())
    }

    pub async fn read_page(
        &self,
        task_id: &str,
        chapter_id: &str,
        page: usize,
    ) -> anyhow::Result<Option<CachedReaderPage>> {
        for format in PageImageFormat::supported() {
            let path = self.page_path(task_id, chapter_id, page, format.extension());
            match fs::read(path).await {
                Ok(data) => return Ok(Some(CachedReaderPage { data, format })),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(None)
    }

    pub async fn stage_task_deletion(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Option<StagedTaskDeletion>> {
        let original = self.root.join(task_id);
        match fs::metadata(&original).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }

        let deleting_root = self.root.join(".deleting");
        fs::create_dir_all(&deleting_root).await?;
        let staged = deleting_root.join(format!(
            "{task_id}-{}",
            chrono::Utc::now().timestamp_millis()
        ));
        fs::rename(&original, &staged).await?;
        Ok(Some(StagedTaskDeletion { original, staged }))
    }

    pub async fn restore_staged_deletion(&self, staged: &StagedTaskDeletion) -> anyhow::Result<()> {
        fs::rename(&staged.staged, &staged.original).await?;
        Ok(())
    }

    pub async fn finish_staged_deletion(
        &self,
        staged: Option<StagedTaskDeletion>,
    ) -> anyhow::Result<()> {
        let Some(staged) = staged else {
            return Ok(());
        };
        match fs::remove_dir_all(staged.staged).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn page_path(&self, task_id: &str, chapter_id: &str, page: usize, extension: &str) -> PathBuf {
        self.root
            .join(task_id)
            .join(chapter_id)
            .join(format!("{page}.{extension}"))
    }
}
