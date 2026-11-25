//! Domain types shared by every provider and shell. No UI, network or OS code lives here.

mod fit;
mod model;
mod provider;

pub use fit::{Fit, Ratio, fit};
pub use model::{Category, Cursor, Item, Page, Rgb, Size, Sort, Thumb, ThumbCrop};
pub use provider::{
    Capabilities, Http, ListRequest, Provider, ProviderError, ProviderInfo, RateLimit,
};
