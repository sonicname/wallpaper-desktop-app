//! Windows integration: screens, the desktop wallpaper, and opening links.
#![cfg(windows)]

mod autostart;
mod desktop;
mod instance;
mod link;
mod screens;
mod system;
mod workspace;

pub use desktop::WindowsDesktop;
pub use instance::{Instance, raise};
pub use link::open_link;
pub use system::WindowsSystem;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}
