use super::{
    manager::DownloadManager,
    model::{DownloadStatus, DownloadTask},
    storage::DownloadStorage,
    DownloadError, DownloadResult,
};
use crate::{
    cache::{page_image_id, CachedReaderPage},
    reader::{decode_with_limits, PageImageFormat},
};
use anyhow::{Context, Result};
use flate2::{write::ZlibEncoder, Compression};
use image::{ImageDecoder, ImageReader, Limits};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, TextStr};
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

const MAX_EXPORT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PAGE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub enum ExportFormat {
    Pdf,
    Cbz,
}

impl ExportFormat {
    pub fn parse(value: Option<&str>) -> DownloadResult<Self> {
        match value {
            Some("pdf") => Ok(Self::Pdf),
            Some("cbz") => Ok(Self::Cbz),
            _ => Err(DownloadError::InvalidRequest(
                "导出格式必须是 pdf 或 cbz".into(),
            )),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Cbz => "cbz",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Cbz => "application/vnd.comicbook+zip",
        }
    }
}

pub struct DownloadExport {
    pub file: File,
    pub permit: tokio::sync::OwnedSemaphorePermit,
    pub filename: String,
    pub format: ExportFormat,
    pub size: u64,
}

struct ExportChapter {
    id: String,
    image_ids: Vec<String>,
}

struct CancelExport(Arc<AtomicBool>);

impl Drop for CancelExport {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

impl DownloadManager {
    pub async fn export(
        &self,
        task_id: &str,
        format: ExportFormat,
    ) -> DownloadResult<DownloadExport> {
        let permit = self
            .export_semaphore
            .clone()
            .try_acquire_owned()
            .map_err(|_| DownloadError::ExportBusy)?;
        let operation = self.task_operations.lock(task_id).await;
        let mut task = self
            .tasks
            .read()
            .await
            .get(task_id)
            .cloned()
            .ok_or(DownloadError::NotFound)?;
        if task.status != DownloadStatus::Completed {
            return Err(DownloadError::InvalidState {
                operation: "export",
                status: task.status.to_string(),
            });
        }
        task.chapters.sort_by_key(|chapter| chapter.order);
        let manifests: HashMap<_, _> = self
            .repository
            .export_manifests(task_id)
            .await?
            .into_iter()
            .map(|manifest| (manifest.chapter_id.clone(), manifest))
            .collect();
        let mut chapters = Vec::with_capacity(task.chapters.len());
        for chapter in &task.chapters {
            let manifest = manifests
                .get(&chapter.chapter_id)
                .filter(|manifest| {
                    manifest.task_id == task.task_id && manifest.album_id == task.album_id
                })
                .ok_or_else(|| {
                    DownloadError::InvalidRequest("下载任务的章节记录不完整，无法导出".into())
                })?;
            if manifest.images.is_empty() {
                return Err(DownloadError::InvalidRequest(
                    "下载章节没有可导出的页面".into(),
                ));
            }
            chapters.push(ExportChapter {
                id: chapter.chapter_id.clone(),
                image_ids: manifest
                    .images
                    .iter()
                    .map(|image| page_image_id(image))
                    .collect(),
            });
        }
        if chapters.is_empty() {
            return Err(DownloadError::InvalidRequest(
                "下载任务没有可导出的章节".into(),
            ));
        }
        let cancelled = CancelExport(Arc::new(AtomicBool::new(false)));
        let signal = cancelled.0.clone();
        let storage = self.storage.clone();
        let filename = export_filename(
            &task.album_id,
            &task.comic_title,
            &task.comic_author,
            format,
        );
        let (file, permit) = tokio::task::spawn_blocking(move || {
            // These guards stay with the blocking job even if the HTTP request is cancelled.
            // Deletion can proceed once the independent export file has been completed.
            let _operation = operation;
            let file = build_export(&storage, &task, &chapters, format, &signal)?;
            Ok::<_, anyhow::Error>((file, permit))
        })
        .await
        .map_err(anyhow::Error::from)??;
        let size = file.metadata().map_err(anyhow::Error::from)?.len();
        Ok(DownloadExport {
            file,
            permit,
            filename,
            format,
            size,
        })
    }
}

fn build_export(
    storage: &DownloadStorage,
    task: &DownloadTask,
    chapters: &[ExportChapter],
    format: ExportFormat,
    cancelled: &AtomicBool,
) -> Result<File> {
    let file = tempfile::tempfile().context("无法创建导出临时文件")?;
    let mut file = match format {
        ExportFormat::Cbz => write_cbz(file, storage, task, chapters, cancelled)?,
        ExportFormat::Pdf => write_pdf(file, storage, task, chapters, cancelled)?,
    };
    check_cancelled(cancelled)?;
    check_size(file.metadata()?.len())?;
    file.seek(SeekFrom::Start(0))?;
    Ok(file)
}

fn read_page(
    storage: &DownloadStorage,
    task_id: &str,
    chapter: &str,
    image_id: &str,
    page: usize,
) -> Result<CachedReaderPage> {
    for format in PageImageFormat::supported() {
        let path = storage.page_path(task_id, chapter, image_id, format.extension());
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).context("无法读取离线图片"),
        };
        anyhow::ensure!(
            file.metadata()?.len() <= MAX_PAGE_BYTES,
            "离线图片过大，无法导出"
        );
        let mut data = Vec::new();
        file.take(MAX_PAGE_BYTES + 1).read_to_end(&mut data)?;
        anyhow::ensure!(
            data.len() as u64 <= MAX_PAGE_BYTES,
            "离线图片过大，无法导出"
        );
        anyhow::ensure!(
            format.is_complete(&data),
            "章节 {chapter} 第 {} 页文件损坏，无法导出",
            page + 1
        );
        return Ok(CachedReaderPage { data, format });
    }
    anyhow::bail!("章节 {chapter} 第 {} 页文件缺失，无法导出", page + 1)
}

fn write_cbz(
    file: File,
    storage: &DownloadStorage,
    task: &DownloadTask,
    chapters: &[ExportChapter],
    cancelled: &AtomicBool,
) -> Result<File> {
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut index = 0usize;
    let mut bytes = 0u64;
    for chapter in chapters {
        for (page, image_id) in chapter.image_ids.iter().enumerate() {
            check_cancelled(cancelled)?;
            let image = read_page(storage, &task.task_id, &chapter.id, image_id, page)?;
            bytes = bytes
                .checked_add(image.data.len() as u64)
                .context("导出文件过大")?;
            check_size(bytes)?;
            index += 1;
            archive.start_file(format!("{index:08}.{}", image.format.extension()), options)?;
            archive.write_all(&image.data)?;
        }
    }
    archive.start_file("ComicInfo.xml", options)?;
    write!(archive, "<?xml version=\"1.0\" encoding=\"utf-8\"?><ComicInfo><Title>{}</Title><Writer>{}</Writer><PageCount>{index}</PageCount></ComicInfo>", xml_escape(&task.comic_title), xml_escape(&task.comic_author))?;
    Ok(archive.finish()?)
}

fn write_pdf(
    mut file: File,
    storage: &DownloadStorage,
    task: &DownloadTask,
    chapters: &[ExportChapter],
    cancelled: &AtomicBool,
) -> Result<File> {
    let count: usize = chapters.iter().map(|chapter| chapter.image_ids.len()).sum();
    anyhow::ensure!(count <= ((i32::MAX - 4) / 3) as usize, "导出页数过多");
    let mut pdf = Pdf::new();
    let catalog = Ref::new(1);
    let tree = Ref::new(2);
    pdf.catalog(catalog).pages(tree);
    pdf.document_info(Ref::new(3))
        .title(TextStr(&task.comic_title))
        .author(TextStr(&task.comic_author))
        .creator(TextStr("JM Boom"));
    let mut pages = Vec::with_capacity(count);
    for chapter in chapters {
        for (index, image_id) in chapter.image_ids.iter().enumerate() {
            check_cancelled(cancelled)?;
            let image = read_page(storage, &task.task_id, &chapter.id, image_id, index)?;
            let (width, height, encoded) = pdf_image(image)?;
            check_size(pdf.len() as u64 + encoded.len() as u64 + 4096)?;
            let page_id = Ref::new(4 + pages.len() as i32 * 3);
            let image_id = Ref::new(page_id.get() + 1);
            let content_id = Ref::new(page_id.get() + 2);
            let name = Name(b"PageImage");
            // Keep very tall strips within PDF's standard 14,400-point page boundary.
            let scale = (14_400.0 / width.max(height) as f32).min(0.75);
            let (w, h) = (width as f32 * scale, height as f32 * scale);
            let mut page = pdf.page(page_id);
            page.parent(tree)
                .media_box(Rect::new(0.0, 0.0, w, h))
                .contents(content_id);
            page.resources().x_objects().pair(name, image_id);
            page.finish();
            let mut object = pdf.image_xobject(image_id, &encoded);
            object.filter(Filter::FlateDecode);
            object
                .width(width as i32)
                .height(height as i32)
                .bits_per_component(8);
            object.color_space().device_rgb();
            object.finish();
            let mut content = Content::new();
            content
                .save_state()
                .transform([w, 0.0, 0.0, h, 0.0, 0.0])
                .x_object(name)
                .restore_state();
            pdf.stream(content_id, &content.finish());
            pages.push(page_id);
        }
    }
    pdf.pages(tree).count(pages.len() as i32).kids(pages);
    let data = pdf.finish();
    check_size(data.len() as u64)?;
    file.write_all(&data)?;
    Ok(file)
}

fn pdf_image(page: CachedReaderPage) -> Result<(u32, u32, Vec<u8>)> {
    let orientation = {
        let mut reader = ImageReader::new(Cursor::new(&page.data)).with_guessed_format()?;
        let mut limits = Limits::default();
        limits.max_alloc = Some(1024 * 1024);
        reader.limits(limits);
        reader.into_decoder()?.orientation()?
    };
    let mut image = decode_with_limits(page.data, page.format)
        .context("PDF 导出失败：图片无法解码或尺寸过大")?;
    image.apply_orientation(orientation);
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    let mut row = Vec::with_capacity(width as usize * 3);
    for pixels in image.rows() {
        row.clear();
        for pixel in pixels {
            let alpha = u16::from(pixel[3]);
            for channel in &pixel.0[..3] {
                row.push(((u16::from(*channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8);
            }
        }
        encoder.write_all(&row)?;
    }
    Ok((width, height, encoder.finish()?))
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<()> {
    anyhow::ensure!(!cancelled.load(Ordering::Relaxed), "导出已取消");
    Ok(())
}

fn check_size(bytes: u64) -> Result<()> {
    anyhow::ensure!(
        bytes <= MAX_EXPORT_BYTES,
        "导出文件超过 512 MiB，请减少下载任务中的章节数量后重试"
    );
    Ok(())
}

fn export_filename(album_id: &str, title: &str, author: &str, format: ExportFormat) -> String {
    format!(
        "JM{}_{}_{}.{}",
        filename_part(album_id, "unknown", 20),
        filename_part(title, "未命名", 140),
        filename_part(author, "N_A", 60),
        format.extension()
    )
}

fn filename_part(value: &str, fallback: &str, max_bytes: usize) -> String {
    let mut name = String::new();
    for ch in value.trim().chars() {
        let ch = if ch.is_control() || "<>:\"/\\|?*".contains(ch) {
            '_'
        } else {
            ch
        };
        if name.len() + ch.len_utf8() > max_bytes {
            break;
        }
        name.push(ch);
    }
    let name = name.trim_end_matches(['.', ' ']);
    if name.is_empty() {
        fallback.to_string()
    } else {
        name.to_string()
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_names_include_id_title_and_author_without_unsafe_path_characters() {
        assert_eq!(
            export_filename("123456", "漫画名称", "作者", ExportFormat::Cbz),
            "JM123456_漫画名称_作者.cbz"
        );
        assert_eq!(
            export_filename("123", " 漫画:/\\*?\"<>| ", "", ExportFormat::Pdf),
            "JM123_漫画__________N_A.pdf"
        );
        let name = export_filename(
            "123456",
            &"长标题".repeat(100),
            &"作者".repeat(100),
            ExportFormat::Cbz,
        );
        assert!(name.len() < 255);
        assert!(name.ends_with(".cbz"));
    }

    #[test]
    fn pdf_images_keep_dimensions_and_composite_transparency_on_white() {
        let image = image::RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                image::Rgba([255, 0, 0, 128])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut data = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut data, image::ImageFormat::Png)
            .unwrap();
        let (width, height, compressed) = pdf_image(CachedReaderPage {
            data: data.into_inner(),
            format: PageImageFormat::Png,
        })
        .unwrap();
        assert_eq!((width, height), (2, 1));
        let mut rgb = Vec::new();
        flate2::read::ZlibDecoder::new(compressed.as_slice())
            .read_to_end(&mut rgb)
            .unwrap();
        assert_eq!(rgb, vec![255, 127, 127, 255, 255, 255]);
    }

    #[test]
    fn format_size_and_cancellation_guards_reject_invalid_exports() {
        assert!(ExportFormat::parse(Some("exe")).is_err());
        assert!(ExportFormat::parse(None).is_err());
        assert!(check_size(MAX_EXPORT_BYTES + 1).is_err());
        let signal = Arc::new(AtomicBool::new(false));
        let guard = CancelExport(signal.clone());
        assert!(check_cancelled(&signal).is_ok());
        drop(guard);
        assert!(check_cancelled(&signal).is_err());
        assert_eq!(xml_escape("A & <B>"), "A &amp; &lt;B&gt;");
    }
}
