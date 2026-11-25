#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, error::Error, path::PathBuf, sync::Arc};

use wallpaper_desktop::{App, Config};
use wallpaper_platform_windows::WindowsDesktop;
use wallpaper_provider_wallhaven::Wallhaven;

fn main() -> Result<(), Box<dyn Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    let data_dir =
        env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("WallpaperDesktop"));
    let app = App::new(Config {
        provider: Arc::new(Wallhaven),
        desktop: Arc::new(WindowsDesktop),
        data_dir,
    })?;
    app.run()?;
    Ok(())
}
