//! Windows integration: screens, the desktop wallpaper, and opening links.
#![cfg(windows)]

mod desktop;
mod link;
mod screens;

pub use desktop::WindowsDesktop;
pub use link::open_link;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}
