use image::{DynamicImage, GenericImageView, ImageBuffer};

const SCRAMBLE_NUM: u32 = 10;
const SCRAMBLE_ID_MODULO: u32 = 220980;

/// Descramble JM image based on chapter ID
pub fn descramble_image(data: &[u8], chapter_id: &str) -> Result<DynamicImage, String> {
    // Load image
    let img = image::load_from_memory(data).map_err(|e| format!("Failed to load image: {e}"))?;

    // Parse chapter ID
    let id_num: u32 = chapter_id
        .parse()
        .map_err(|e| format!("Invalid chapter ID: {e}"))?;

    // If ID is below threshold, no descrambling needed
    if id_num < SCRAMBLE_ID_MODULO {
        return Ok(img);
    }

    let (width, height) = img.dimensions();
    let remainder = height % SCRAMBLE_NUM;
    let block_height = height / SCRAMBLE_NUM;

    // Create output buffer
    let mut output = ImageBuffer::new(width, height);

    // Copy each scrambled block to its correct position
    for i in 0..SCRAMBLE_NUM {
        let src_y = height - block_height * (i + 1) - remainder;
        let dst_y = block_height * i + remainder;

        for y in 0..block_height {
            for x in 0..width {
                let pixel = img.get_pixel(x, src_y + y);
                output.put_pixel(x, dst_y + y, pixel);
            }
        }
    }

    // Copy remainder at top
    if remainder > 0 {
        for y in 0..remainder {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                output.put_pixel(x, y, pixel);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}
