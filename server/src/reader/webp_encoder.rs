use image::DynamicImage;
use webp::Encoder;

const WEBP_QUALITY: f32 = 80.0;

/// Encode image to WebP format
pub fn encode_webp(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let encoder = Encoder::from_rgba(&rgba, width, height);
    let encoded = encoder.encode(WEBP_QUALITY);

    Ok(encoded.to_vec())
}
