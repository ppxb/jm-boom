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
        anyhow::ensure!(
            page_image.format.is_complete(&page_image.data),
            "拒绝写入不完整的下载页面"
        );
        let extension = page_image.format.extension();
        let path = self.page_path(task_id, chapter_id, page, extension);
        let temporary_path = self.page_path(task_id, chapter_id, page, &format!("{extension}.tmp"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let result = async {
            fs::write(&temporary_path, &page_image.data).await?;
            fs::rename(&temporary_path, &path).await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;

        if let Err(error) = result {
            if let Err(cleanup_error) = remove_file_if_exists(&temporary_path).await {
                tracing::warn!(
                    path = %temporary_path.display(),
                    %cleanup_error,
                    "下载页面临时文件清理失败"
                );
            }
            return Err(error);
        }
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
            match fs::read(&path).await {
                Ok(data) if format.is_complete(&data) => {
                    return Ok(Some(CachedReaderPage { data, format }));
                }
                Ok(data) => {
                    tracing::warn!(
                        task_id,
                        chapter_id,
                        page,
                        bytes = data.len(),
                        path = %path.display(),
                        "删除损坏或格式不匹配的下载页面"
                    );
                    remove_file_if_exists(&path).await?;
                }
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

async fn remove_file_if_exists(path: &std::path::Path) -> anyhow::Result<()> {
    match fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::DownloadStorage;
    use crate::{cache::CachedReaderPage, reader::PageImageFormat};
    use std::{
        io::Cursor,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    use tokio::fs;

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    #[tokio::test]
    async fn stores_page_via_temporary_file_and_ignores_uncommitted_file() {
        let root = test_root("atomic");
        let storage = DownloadStorage::new(root.clone())
            .await
            .expect("create download storage");
        let data = png_data();
        let temporary_path = storage.page_path("task", "1001", 0, "png.tmp");
        fs::create_dir_all(temporary_path.parent().expect("temporary parent"))
            .await
            .expect("create temporary parent");
        fs::write(&temporary_path, &data)
            .await
            .expect("write interrupted temporary file");

        assert!(storage
            .read_page("task", "1001", 0)
            .await
            .expect("read page before commit")
            .is_none());

        storage
            .store_page(
                "task",
                "1001",
                0,
                &CachedReaderPage {
                    data: data.clone(),
                    format: PageImageFormat::Png,
                },
            )
            .await
            .expect("store page atomically");

        assert!(!temporary_path.exists());
        let stored = storage
            .read_page("task", "1001", 0)
            .await
            .expect("read committed page")
            .expect("committed page should exist");
        assert_eq!(stored.format, PageImageFormat::Png);
        assert_eq!(stored.data, data);

        fs::remove_dir_all(root).await.expect("remove test storage");
    }

    #[tokio::test]
    async fn removes_truncated_page_instead_of_reusing_it() {
        let root = test_root("truncated");
        let storage = DownloadStorage::new(root.clone())
            .await
            .expect("create download storage");
        let path = storage.page_path("task", "1001", 0, "png");
        let mut truncated = png_data();
        truncated.truncate(truncated.len() - 12);
        fs::create_dir_all(path.parent().expect("page parent"))
            .await
            .expect("create page parent");
        fs::write(&path, truncated)
            .await
            .expect("write truncated page");

        assert!(storage
            .read_page("task", "1001", 0)
            .await
            .expect("read truncated page")
            .is_none());
        assert!(!path.exists());

        fs::remove_dir_all(root).await.expect("remove test storage");
    }

    fn png_data() -> Vec<u8> {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            1,
            1,
            image::Rgb([1, 2, 3]),
        ));
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("encode png");
        bytes.into_inner()
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "jm-boom-download-storage-{name}-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
