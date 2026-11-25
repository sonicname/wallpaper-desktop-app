#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, error::Error, path::PathBuf, sync::Arc};

use wallpaper_desktop::{App, Config, open_window};
use wallpaper_platform_windows::{Instance, WindowsDesktop, WindowsSystem};
use wallpaper_provider_wallhaven::Wallhaven;

/// Identifies the app to Windows. Changing it loses the startup entry.
const NAME: &str = "WallpaperDesktop";
/// Starts without a window, for the startup entry.
const BACKGROUND: &str = "--background";

fn main() -> Result<(), Box<dyn Error>> {
    let Some(instance) = Instance::acquire(NAME) else {
        return Ok(());
    };
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    let data_dir = env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join(NAME));
    let app = App::new(Config {
        provider: Arc::new(Wallhaven),
        desktop: Arc::new(WindowsDesktop),
        system: Arc::new(WindowsSystem::new(
            NAME,
            format!("\"{}\" {BACKGROUND}", env::current_exe()?.display()),
        )),
        data_dir,
    })?;
    instance.on_start_again(|| drop(slint::invoke_from_event_loop(open_window)));
    app.run(env::args().any(|arg| arg == BACKGROUND))?;
    Ok(())
}
