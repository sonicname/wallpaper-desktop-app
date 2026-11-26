//! Everything providers should not have to care about: HTTP, rate limits, disk cache, decoding.

mod cache;
mod gallery;
mod http;
mod image_ops;

pub use cache::DiskCache;
pub use gallery::{Gallery, write_picture, write_whole};
pub use http::HostHttp;
pub use image_ops::{ImageError, Pixels, cover, dimensions, thumbnail};
