use super::{decode_scrambled_image, encode_webp, needs_decoding};
use anyhow::{bail, Context, Result};
use image::{ImageFormat, ImageReader, Limits};
use std::io::Cursor;

const MAX_DECODE_WIDTH: u32 = 16_384;
const MAX_DECODE_HEIGHT: u32 = 65_535;
const MAX_DECODE_PIXELS: u64 = 32_000_000;
const MAX_DECODE_ALLOC_BYTES: u64 = 192 * 1024 * 1024;

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

    fn image_format(self) -> ImageFormat {
        match self {
            Self::Gif => ImageFormat::Gif,
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
            Self::WebP => ImageFormat::WebP,
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
        let original = decode_with_limits(data, format)?;
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

fn decode_with_limits(data: Vec<u8>, format: PageImageFormat) -> Result<image::DynamicImage> {
    let image_format = format.image_format();
    let dimensions = ImageReader::with_format(Cursor::new(data.as_slice()), image_format)
        .into_dimensions()
        .context("无法读取待解扰图片尺寸")?;
    validate_decode_dimensions(dimensions.0, dimensions.1)?;

    let mut reader = ImageReader::with_format(Cursor::new(data), image_format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODE_WIDTH);
    limits.max_image_height = Some(MAX_DECODE_HEIGHT);
    limits.max_alloc = Some(MAX_DECODE_ALLOC_BYTES);
    reader.limits(limits);
    reader.decode().context("无法解码待解扰图片")
}

fn validate_decode_dimensions(width: u32, height: u32) -> Result<()> {
    anyhow::ensure!(
        width <= MAX_DECODE_WIDTH && height <= MAX_DECODE_HEIGHT,
        "待解扰图片尺寸超过限制: {width}x{height}"
    );
    let pixels = u64::from(width)
        .checked_mul(u64::from(height))
        .context("待解扰图片像素数量溢出")?;
    anyhow::ensure!(
        pixels <= MAX_DECODE_PIXELS,
        "待解扰图片像素数量超过限制: {pixels}"
    );
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{validate_decode_dimensions, MAX_DECODE_HEIGHT, MAX_DECODE_WIDTH};

    #[test]
    fn bounds_scrambled_image_dimensions_and_pixels() {
        assert!(validate_decode_dimensions(1_500, 20_000).is_ok());
        assert!(validate_decode_dimensions(MAX_DECODE_WIDTH + 1, 1).is_err());
        assert!(validate_decode_dimensions(1, MAX_DECODE_HEIGHT + 1).is_err());
        assert!(validate_decode_dimensions(8_000, 5_000).is_err());
    }
}
