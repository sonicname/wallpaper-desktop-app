//! The Slint shell. Everything that does not draw lives in the packages.

mod apply;
mod controller;
mod grid;
mod layout;
mod loader;
mod text;

slint::include_modules!();

pub use controller::{App, Config};
