mod chapter;
mod client;
mod crypto;
mod error;
mod models;
mod setting;
mod signature;

pub use client::JmClient;
pub use error::{JmError, JmResult};
pub use models::*;
pub(crate) use setting::invalidate_img_host;
pub(crate) use signature::SettingRequestSignature;

// API constants
pub(crate) const API_VERSION: &str = "2.0.20";
pub(crate) const API_SECRET: &str = "185Hcomic3PAPP7R";
