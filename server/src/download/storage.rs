use super::model::OfflineChapterManifest;
use crate::{
    cache::{page_image_id, CachedReaderPage},
    keyed_lock::KeyedLock,
    reader::PageImageFormat,
};
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
pub(super) struct DownloadStorage {
    root: PathBuf,
    page_writes: KeyedLock,
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
        Ok(Self {
            root,
            page_writes: KeyedLock::new(),
        })
    }

    // One-time, restart-safe migration. Normal reads never fall back to page-number files.
    pub async fn migrate_manifest(&self, manifest: &OfflineChapterManifest) -> anyhow::Result<()> {
        let directory = self.root.join(&manifest.task_id).join(&manifest.chapter_id);
        for (index, image) in manifest.images.iter().enumerate() {
            let identity = page_image_id(image);
            for format in PageImageFormat::supported() {
                let old = directory.join(format!("{index}.{}", format.extension()));
                if !fs::try_exists(&old).await? {
                    continue;
                }
                let destination = self.page_path(
                    &manifest.task_id,
                    &manifest.chapter_id,
                    &identity,
                    format.extension(),
                );
                if fs::try_exists(&destination).await? {
                    let legacy = directory.join(".legacy");
                    fs::create_dir_all(&legacy).await?;
                    fs::rename(&old, legacy.join(format!("{index}.{}", format.extension())))
                        .await?;
                } else {
                    fs::rename(&old, destination).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn store_page(
        &self,
        task_id: &str,
        chapter_id: &str,
        image_id: &str,
        page_image: &CachedReaderPage,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            page_image.format.is_complete(&page_image.data),
            "拒绝写入不完整的下载页面"
        );
        let _write = self
            .page_writes
            .lock(&format!("{task_id}/{chapter_id}/{image_id}"))
            .await;
        let extension = page_image.format.extension();
        let path = self.page_path(task_id, chapter_id, image_id, extension);
        let temporary_path =
            self.page_path(task_id, chapter_id, image_id, &format!("{extension}.tmp"));
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
        image_id: &str,
    ) -> anyhow::Result<Option<CachedReaderPage>> {
        let _read = self
            .page_writes
            .lock(&format!("{task_id}/{chapter_id}/{image_id}"))
            .await;
        for format in PageImageFormat::supported() {
            let path = self.page_path(task_id, chapter_id, image_id, format.extension());
            match fs::read(&path).await {
                Ok(data) if format.is_complete(&data) => {
                    return Ok(Some(CachedReaderPage { data, format }));
                }
                Ok(data) => {
                    tracing::warn!(
                        task_id,
                        chapter_id,
                        image_id,
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

    pub(super) fn page_path(
        &self,
        task_id: &str,
        chapter_id: &str,
        image_id: &str,
        extension: &str,
    ) -> PathBuf {
        self.root
            .join(task_id)
            .join(chapter_id)
            .join(format!("{image_id}.{extension}"))
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
    async fn legacy_migration_resumes_after_partial_renames_and_preserves_missing_pages() {
        use crate::{cache::page_image_id, download::model::OfflineChapterManifest};
        let root = test_root("identity-migration");
        let storage = DownloadStorage::new(root.clone()).await.unwrap();
        let directory = root.join("task/1001");
        fs::create_dir_all(&directory).await.unwrap();
        let data = png_data();
        // Page A was already renamed before a previous startup stopped.
        fs::write(
            storage.page_path("task", "1001", &page_image_id("A.png"), "png"),
            &data,
        )
        .await
        .unwrap();
        fs::write(directory.join("1.png"), &data).await.unwrap();
        fs::write(directory.join("3.png"), &data).await.unwrap();
        let manifest = OfflineChapterManifest {
            task_id: "task".into(),
            album_id: "100".into(),
            chapter_id: "1001".into(),
            title: "章".into(),
            images: vec![
                "A.png".into(),
                "B.png".into(),
                "C.png".into(),
                "A.png".into(),
            ],
            updated_at: 1,
        };
        storage.migrate_manifest(&manifest).await.unwrap();
        storage.migrate_manifest(&manifest).await.unwrap();
        for image in ["A.png", "B.png"] {
            assert_eq!(
                storage
                    .read_page("task", "1001", &page_image_id(image))
                    .await
                    .unwrap()
                    .unwrap()
                    .data,
                data
            );
        }
        assert!(storage
            .read_page("task", "1001", &page_image_id("C.png"))
            .await
            .unwrap()
            .is_none());
        assert!(!directory.join("1.png").exists());
        assert_eq!(
            fs::read(directory.join(".legacy/3.png")).await.unwrap(),
            data
        );
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn concurrent_copies_of_the_same_image_commit_without_temp_file_collisions() {
        let root = test_root("identity-concurrent");
        let storage = DownloadStorage::new(root.clone()).await.unwrap();
        let mut jobs = tokio::task::JoinSet::new();
        for _ in 0..5 {
            let storage = storage.clone();
            jobs.spawn(async move {
                storage
                    .store_page(
                        "task",
                        "1001",
                        &crate::cache::page_image_id("A.png"),
                        &CachedReaderPage {
                            data: png_data(),
                            format: PageImageFormat::Png,
                        },
                    )
                    .await
            });
        }
        while let Some(result) = jobs.join_next().await {
            result.unwrap().unwrap();
        }
        assert_eq!(
            storage
                .read_page("task", "1001", &crate::cache::page_image_id("A.png"))
                .await
                .unwrap()
                .unwrap()
                .data,
            png_data()
        );
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn a_bad_read_cannot_remove_a_concurrent_valid_replacement() {
        let root = test_root("identity-read-write");
        let storage = DownloadStorage::new(root.clone()).await.unwrap();
        let id = crate::cache::page_image_id("A.png");
        let path = storage.page_path("task", "1001", &id, "png");
        fs::create_dir_all(path.parent().unwrap()).await.unwrap();
        fs::write(&path, b"broken").await.unwrap();
        let writing = storage.page_writes.lock(&format!("task/1001/{id}")).await;
        let mut reading = tokio::spawn({
            let storage = storage.clone();
            let id = id.clone();
            async move { storage.read_page("task", "1001", &id).await }
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), &mut reading)
                .await
                .is_err()
        );
        fs::write(&path, png_data()).await.unwrap();
        drop(writing);
        assert_eq!(reading.await.unwrap().unwrap().unwrap().data, png_data());
        assert!(path.exists());
        fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn stores_page_via_temporary_file_and_ignores_uncommitted_file() {
        let root = test_root("atomic");
        let storage = DownloadStorage::new(root.clone())
            .await
            .expect("create download storage");
        let data = png_data();
        let temporary_path = storage.page_path(
            "task",
            "1001",
            &crate::cache::page_image_id("001.png"),
            "png.tmp",
        );
        fs::create_dir_all(temporary_path.parent().expect("temporary parent"))
            .await
            .expect("create temporary parent");
        fs::write(&temporary_path, &data)
            .await
            .expect("write interrupted temporary file");

        assert!(storage
            .read_page("task", "1001", &crate::cache::page_image_id("001.png"))
            .await
            .expect("read page before commit")
            .is_none());

        storage
            .store_page(
                "task",
                "1001",
                &crate::cache::page_image_id("001.png"),
                &CachedReaderPage {
                    data: data.clone(),
                    format: PageImageFormat::Png,
                },
            )
            .await
            .expect("store page atomically");

        assert!(!temporary_path.exists());
        let stored = storage
            .read_page("task", "1001", &crate::cache::page_image_id("001.png"))
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
        let path = storage.page_path(
            "task",
            "1001",
            &crate::cache::page_image_id("001.png"),
            "png",
        );
        let mut truncated = png_data();
        truncated.truncate(truncated.len() - 12);
        fs::create_dir_all(path.parent().expect("page parent"))
            .await
            .expect("create page parent");
        fs::write(&path, truncated)
            .await
            .expect("write truncated page");

        assert!(storage
            .read_page("task", "1001", &crate::cache::page_image_id("001.png"))
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
