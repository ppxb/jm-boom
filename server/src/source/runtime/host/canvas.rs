use image::{imageops, imageops::FilterType, RgbaImage};
use wasmer::FunctionEnvMut;

use super::HostState;
use crate::source::runtime::store::DescriptorValue;

const INVALID_CONTEXT: i32 = -1;
const INVALID_IMAGE: i32 = -2;
const INVALID_BOUNDS: i32 = -6;
const MAX_PIXELS: u64 = 40_000_000;

pub(crate) type Canvas = RgbaImage;
pub(crate) type ImageData = RgbaImage;

pub(super) fn new_context(mut env: FunctionEnvMut<HostState>, width: f32, height: f32) -> i32 {
    let Some((width, height)) = dimensions(width, height) else {
        return INVALID_BOUNDS;
    };
    let canvas = RgbaImage::new(width, height);
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Canvas(canvas))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn copy_image(
    mut env: FunctionEnvMut<HostState>,
    context: i32,
    image: i32,
    src_x: f32,
    src_y: f32,
    src_width: f32,
    src_height: f32,
    dst_x: f32,
    dst_y: f32,
    dst_width: f32,
    dst_height: f32,
) -> i32 {
    let Some(source) = env
        .data()
        .descriptors
        .get(image)
        .and_then(DescriptorValue::as_image)
        .cloned()
    else {
        return INVALID_IMAGE;
    };
    let Some((src_x, src_y, src_width, src_height)) = rect(
        src_x,
        src_y,
        src_width,
        src_height,
        source.width(),
        source.height(),
    ) else {
        return INVALID_BOUNDS;
    };
    let Some((_, _, dst_width, dst_height)) =
        rect(dst_x, dst_y, dst_width, dst_height, u32::MAX, u32::MAX)
    else {
        return INVALID_BOUNDS;
    };
    let Some(canvas) = env
        .data_mut()
        .descriptors
        .get_mut(context)
        .and_then(DescriptorValue::as_canvas_mut)
    else {
        return INVALID_CONTEXT;
    };
    let cropped = imageops::crop_imm(&source, src_x, src_y, src_width, src_height).to_image();
    let resized = if cropped.width() == dst_width && cropped.height() == dst_height {
        cropped
    } else {
        imageops::resize(&cropped, dst_width, dst_height, FilterType::Triangle)
    };
    imageops::overlay(canvas, &resized, dst_x as i64, dst_y as i64);
    0
}

pub(super) fn get_image(mut env: FunctionEnvMut<HostState>, context: i32) -> i32 {
    let Some(image) = env
        .data()
        .descriptors
        .get(context)
        .and_then(DescriptorValue::as_canvas)
        .cloned()
    else {
        return INVALID_CONTEXT;
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Image(image))
}

pub(super) fn get_image_width(env: FunctionEnvMut<HostState>, image: i32) -> f32 {
    env.data()
        .descriptors
        .get(image)
        .and_then(DescriptorValue::as_image)
        .map(|value| value.width() as f32)
        .unwrap_or(INVALID_IMAGE as f32)
}

pub(super) fn get_image_height(env: FunctionEnvMut<HostState>, image: i32) -> f32 {
    env.data()
        .descriptors
        .get(image)
        .and_then(DescriptorValue::as_image)
        .map(|value| value.height() as f32)
        .unwrap_or(INVALID_IMAGE as f32)
}

fn dimensions(width: f32, height: f32) -> Option<(u32, u32)> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let width = width.ceil() as u32;
    let height = height.ceil() as u32;
    (u64::from(width) * u64::from(height) <= MAX_PIXELS).then_some((width, height))
}

fn rect(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    max_width: u32,
    max_height: u32,
) -> Option<(u32, u32, u32, u32)> {
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || x < 0.0
        || y < 0.0
        || width <= 0.0
        || height <= 0.0
    {
        return None;
    }
    let x = x.floor() as u32;
    let y = y.floor() as u32;
    let width = width.ceil() as u32;
    let height = height.ceil() as u32;
    if x >= max_width || y >= max_height || width > max_width - x || height > max_height - y {
        return None;
    }
    Some((x, y, width, height))
}

#[cfg(test)]
mod tests {
    use super::{dimensions, rect};

    #[test]
    fn rejects_oversized_canvas_and_out_of_bounds_copy() {
        assert_eq!(dimensions(1200.0, 2000.0), Some((1200, 2000)));
        assert_eq!(dimensions(10_000.0, 10_000.0), None);
        assert_eq!(
            rect(0.0, 0.0, 100.0, 100.0, 100, 100),
            Some((0, 0, 100, 100))
        );
        assert_eq!(rect(50.0, 50.0, 51.0, 50.0, 100, 100), None);
    }
}
