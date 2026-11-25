//! Turns a picture from a source into the wallpaper, and back. Runs on worker threads.

use std::path::PathBuf;

use wallpaper_core::{Backdrop, Desktop, DesktopError, Item, Picture, ProviderError, Screen, Size};
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

/// Where a change goes: the workspace the user was looking at when asking for it.
pub struct Place<'a> {
    pub gallery: &'a Gallery,
    pub desktop: &'a dyn Desktop,
    pub workspace: Option<&'a str>,
}

/// Crops `item` for every screen and shows it. The desktop is unchanged if this fails, except
/// for [`DesktopError::WorkspaceSwitched`].
pub fn apply(
    item: &Item,
    screens: &[Screen],
    loader: &Loader,
    place: &Place,
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
                let path = place.gallery.save(&pixels).map_err(|_| ApplyError::Disk)?;
                ready.push((screen.size, path.clone()));
                path
            }
        };
        wanted.push((screen.id.clone(), path));
    }
    show(wanted, place)
}

/// Shows pictures that are in the gallery already. The desktop is unchanged if this fails,
/// except for [`DesktopError::WorkspaceSwitched`].
pub fn show(wanted: Vec<(String, PathBuf)>, place: &Place) -> Result<(), ApplyError> {
    // The download can take a while, and the user may have moved on in the meantime.
    let before = place.desktop.read().map_err(ApplyError::Desktop)?;
    if before.workspace.as_deref() != place.workspace {
        return Err(ApplyError::Desktop(DesktopError::OtherWorkspace));
    }
    place
        .gallery
        .remember(before.clone())
        .map_err(|_| ApplyError::Disk)?;
    let result = place.desktop.apply(&wanted, place.workspace);
    match &result {
        Ok(()) => place.record(),
        // Probably shown, so the pictures and the way back have to stay.
        Err(DesktopError::WorkspaceSwitched) => {
            let _ = place.gallery.showing(with_pictures(before, wanted));
        }
        Err(_) => {
            // Some screens may already have changed. The first error is the one worth reporting.
            let _ = place.desktop.restore(&before);
            let _ = place.gallery.forget_previous(place.workspace);
            place.record();
        }
    }
    result.map_err(ApplyError::Desktop)
}

/// Brings back what this workspace showed before the last [`apply`] to it.
pub fn undo(place: &Place) -> Result<(), ApplyError> {
    let before = place
        .gallery
        .previous(place.workspace)
        .ok_or(ApplyError::NothingToUndo)?;
    place
        .desktop
        .restore(&before)
        .map_err(ApplyError::Desktop)?;
    place
        .gallery
        .forget_previous(place.workspace)
        .map_err(|_| ApplyError::Disk)?;
    place.record();
    Ok(())
}

impl Place<'_> {
    fn record(&self) {
        // Leftover files only cost space; the next change tries again.
        if let Ok(shown) = self.desktop.read()
            && shown.workspace.as_deref() == self.workspace
        {
            let _ = self.gallery.showing(shown);
        }
    }
}

fn with_pictures(mut backdrop: Backdrop, pictures: Vec<(String, PathBuf)>) -> Backdrop {
    for (screen, path) in pictures {
        backdrop.pictures.retain(|p| p.screen != screen);
        backdrop.pictures.push(Picture {
            screen,
            path,
            kept: None,
        });
    }
    backdrop
}
