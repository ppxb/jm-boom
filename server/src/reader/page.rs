use super::{decode_scrambled_image, encode_webp, needs_decoding};
use anyhow::{bail, Context, Result};
use image::ImageFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageImageFormat {
    Gif,
    Jpeg,
    Png,
    WebP,
}

impl PageImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Gif => "gif",
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Gif => "image/gif",
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
        }
    }

    pub fn supported() -> [Self; 4] {
        [Self::WebP, Self::Gif, Self::Jpeg, Self::Png]
    }
}

pub struct PreparedPageImage {
    pub data: Vec<u8>,
    pub format: PageImageFormat,
    pub decoded: bool,
}

pub async fn prepare_page_image(
    data: Vec<u8>,
    comic_id: u32,
    page_name: String,
) -> Result<PreparedPageImage> {
    let format = detect_page_image_format(&data)?;
    if !needs_decoding(comic_id, &page_name, format == PageImageFormat::Gif) {
        return Ok(PreparedPageImage {
            data,
            format,
            decoded: false,
        });
    }

    let data = tokio::task::spawn_blocking(move || {
        let original = image::load_from_memory(&data).context("无法解码待解扰图片")?;
        let decoded = decode_scrambled_image(original, comic_id, &page_name);
        Ok::<_, anyhow::Error>(encode_webp(&decoded))
    })
    .await
    .context("图片解扰任务异常退出")??;

    Ok(PreparedPageImage {
        data,
        format: PageImageFormat::WebP,
        decoded: true,
    })
}

fn detect_page_image_format(data: &[u8]) -> Result<PageImageFormat> {
    match image::guess_format(data).context("无法识别图片格式")? {
        ImageFormat::Gif => Ok(PageImageFormat::Gif),
        ImageFormat::Jpeg => Ok(PageImageFormat::Jpeg),
        ImageFormat::Png => Ok(PageImageFormat::Png),
        ImageFormat::WebP => Ok(PageImageFormat::WebP),
        format => bail!("暂不支持的阅读图片格式: {format:?}"),
    }
}
