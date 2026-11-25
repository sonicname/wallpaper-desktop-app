//! Turns a picture from a source into the wallpaper, and back. Runs on worker threads.

use std::path::PathBuf;

use wallpaper_core::{Desktop, DesktopError, Item, ProviderError, Screen, Size};
use wallpaper_host::{Gallery, Pixels, cover};

use crate::{grid::BACKGROUND, loader::Loader};

#[derive(Debug)]
pub enum ApplyError {
    Download(ProviderError),
    Unreadable,
    Disk,
    Desktop(DesktopError),
    NothingToUndo,
}

/// Crops `item` for every screen and shows it. The desktop is unchanged if this fails.
pub fn apply(
    item: &Item,
    screens: &[Screen],
    loader: &Loader,
    gallery: &Gallery,
    desktop: &dyn Desktop,
) -> Result<(), ApplyError> {
    let bytes = loader.original(item).map_err(ApplyError::Download)?;
    let mut ready: Vec<(Size, PathBuf)> = vec![];
    let mut wanted = vec![];
    for screen in screens {
        let known = ready
            .iter()
            .find(|r| r.0 == screen.size)
            .map(|r| r.1.clone());
        let path = match known {
            Some(path) => path,
            None => {
                let pixels: Pixels = cover(&bytes, screen.size, 0.0, BACKGROUND)
                    .map_err(|_| ApplyError::Unreadable)?;
                let path = gallery.save(&pixels).map_err(|_| ApplyError::Disk)?;
                ready.push((screen.size, path.clone()));
                path
            }
        };
        wanted.push((screen.id.clone(), path));
    }
    let before = desktop.read().map_err(ApplyError::Desktop)?;
    gallery
        .remember(before.clone())
        .map_err(|_| ApplyError::Disk)?;
    if let Err(error) = desktop.apply(&wanted) {
        // Some screens may already have changed. The first error is the one worth reporting.
        let _ = desktop.restore(&before);
        let _ = gallery.forget_previous();
        clean(gallery, desktop);
        return Err(ApplyError::Desktop(error));
    }
    clean(gallery, desktop);
    Ok(())
}

/// Brings back what was shown before the last [`apply`].
pub fn undo(gallery: &Gallery, desktop: &dyn Desktop) -> Result<(), ApplyError> {
    let before = gallery.previous().ok_or(ApplyError::NothingToUndo)?;
    desktop.restore(&before).map_err(ApplyError::Desktop)?;
    gallery.forget_previous().map_err(|_| ApplyError::Disk)?;
    clean(gallery, desktop);
    Ok(())
}

fn clean(gallery: &Gallery, desktop: &dyn Desktop) {
    // Leftover files only cost space; the next change tries again.
    if let Ok(shown) = desktop.read() {
        let _ = gallery.clean(&shown);
    }
}
