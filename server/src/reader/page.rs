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

    pub fn detect(data: &[u8]) -> Result<Self> {
        match image::guess_format(data).context("无法识别图片格式")? {
            ImageFormat::Gif => Ok(Self::Gif),
            ImageFormat::Jpeg => Ok(Self::Jpeg),
            ImageFormat::Png => Ok(Self::Png),
            ImageFormat::WebP => Ok(Self::WebP),
            format => bail!("暂不支持的阅读图片格式: {format:?}"),
        }
    }

    pub fn is_complete(self, data: &[u8]) -> bool {
        if Self::detect(data).ok() != Some(self) {
            return false;
        }

        match self {
            Self::Gif => data.last() == Some(&0x3b),
            Self::Jpeg => data.ends_with(&[0xff, 0xd9]),
            Self::Png => data.ends_with(&[
                0x00, 0x00, 0x00, 0x00, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82,
            ]),
            Self::WebP => webp_size_matches(data),
        }
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
    let format = PageImageFormat::detect(&data)?;
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

fn webp_size_matches(data: &[u8]) -> bool {
    let Some(size_bytes) = data.get(4..8).and_then(|bytes| bytes.try_into().ok()) else {
        return false;
    };
    let declared_size = u32::from_le_bytes(size_bytes) as usize;

    declared_size
        .checked_add(8)
        .is_some_and(|expected_size| expected_size == data.len())
}
