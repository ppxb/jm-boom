mod decoder;
mod page;

pub(crate) use page::decode_with_limits;

pub use decoder::{decode_scrambled_image, encode_webp, needs_decoding, page_name_from_image};
pub use page::{prepare_page_image, ImageDimensions, PageImageFormat};
