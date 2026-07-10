use image::{DynamicImage, RgbImage};

const JM_SCRAMBLE_ID: u32 = 220_980;
const SCRAMBLED_WEBP_QUALITY: f32 = 75.0;

/// Decode scrambled JM image
pub fn decode_scrambled_image(original: DynamicImage, comic_id: u32, page_name: &str) -> RgbImage {
    let rgb = original.to_rgb8();
    let seed = segmentation_count(comic_id, page_name);

    if seed <= 1 {
        return rgb;
    }

    reorder_scrambled_rows(&rgb, seed)
}

/// Check if image needs decoding
pub fn needs_decoding(comic_id: u32, page_name: &str, is_gif: bool) -> bool {
    !is_gif && segmentation_count(comic_id, page_name) > 1
}

/// Encode decoded image to WebP
pub fn encode_webp(decoded: &RgbImage) -> Vec<u8> {
    let (width, height) = decoded.dimensions();
    let encoder = webp::Encoder::from_rgb(decoded, width, height);
    encoder.encode(SCRAMBLED_WEBP_QUALITY).to_vec()
}

// Internal helpers

fn reorder_scrambled_rows(source: &RgbImage, seed: u32) -> RgbImage {
    let (width, height) = source.dimensions();
    let row_bytes = width as usize * 3;
    let source_bytes = source.as_raw();
    let mut decoded = RgbImage::new(width, height);
    let decoded_bytes = decoded.as_mut();
    let remainder = height % seed;

    for index in 0..seed {
        let mut block_height = height / seed;
        let mut dy = block_height * index;
        let sy = height - block_height * (index + 1) - remainder;

        if index == 0 {
            block_height += remainder;
        } else {
            dy += remainder;
        }

        for row in 0..block_height {
            let source_offset = (sy + row) as usize * row_bytes;
            let target_offset = (dy + row) as usize * row_bytes;
            let source_row = &source_bytes[source_offset..source_offset + row_bytes];
            let target_row = &mut decoded_bytes[target_offset..target_offset + row_bytes];
            target_row.copy_from_slice(source_row);
        }
    }

    decoded
}

fn segmentation_count(comic_id: u32, page_name: &str) -> u32 {
    if comic_id < JM_SCRAMBLE_ID {
        return 0;
    }

    if comic_id < 268_850 {
        return 10;
    }

    let key = format!("{comic_id}{page_name}");
    let key_md5 = format!("{:x}", md5::compute(key));
    let last_char = key_md5
        .as_bytes()
        .last()
        .copied()
        .map(u32::from)
        .unwrap_or_default();

    if comic_id > 421_926 {
        return (last_char % 8) * 2 + 2;
    }

    (last_char % 10) * 2 + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmentation_count() {
        assert_eq!(segmentation_count(100_000, "001.jpg"), 0);
        assert_eq!(segmentation_count(250_000, "001.jpg"), 10);
        assert!(segmentation_count(500_000, "001.jpg") >= 2);
    }
}
