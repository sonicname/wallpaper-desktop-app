//! Domain types shared by every provider and shell. No UI, network or OS code lives here.

mod desktop;
mod fit;
mod model;
mod provider;
mod system;

pub use desktop::{Backdrop, BackdropMode, Desktop, DesktopError, Picture, Screen};
pub use fit::{Fit, Ratio, fit};
pub use model::{Category, Cursor, Item, Page, Rgb, Size, Sort, Thumb, ThumbCrop, local_path};
pub use provider::{
    Capabilities, Http, ListRequest, Provider, ProviderError, ProviderInfo, RateLimit,
};
pub use system::{Autostart, LocalTime, System};
