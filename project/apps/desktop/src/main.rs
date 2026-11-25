#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, error::Error, path::PathBuf, sync::Arc};

use wallpaper_desktop::{App, Config};
use wallpaper_provider_wallhaven::Wallhaven;

fn main() -> Result<(), Box<dyn Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    let screen = wallpaper_platform_windows::monitors()
        .first()
        .map(|m| m.size);
    let cache_dir = env::var_os("LOCALAPPDATA")
        .map(|dir| PathBuf::from(dir).join("WallpaperDesktop").join("thumbs"));
    let app = App::new(Config {
        provider: Arc::new(Wallhaven),
        screen,
        cache_dir,
    })?;
    app.run()?;
    Ok(())
}
