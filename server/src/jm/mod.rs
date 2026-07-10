mod auth;
mod client;
mod crypto;
mod error;
mod models;

pub use auth::JmAuth;
pub use client::JmClient;
pub use error::{JmError, JmResult};
pub use models::*;

// API constants
pub(crate) const API_VERSION: &str = "2.0.20";
pub(crate) const API_SECRET: &str = "185Hcomic3PAPP7R";
