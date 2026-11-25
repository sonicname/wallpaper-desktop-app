use std::{
    fs, io,
    path::{Path, PathBuf},
};

use image::{
    ExtendedColorType, ImageEncoder,
    codecs::{
        jpeg::JpegEncoder,
        png::{CompressionType, FilterType, PngEncoder},
    },
};
use serde::{Deserialize, Serialize};
use wallpaper_core::Backdrop;

use crate::Pixels;

/// Windows has been reported to show black for larger files, so those become JPEG.
const MAX_PNG_BYTES: usize = 20_000_000;
/// Pictures of other programs above this size are not copied.
const MAX_KEPT_BYTES: u64 = 60_000_000;
/// Steps per workspace.
const MAX_HISTORY: usize = 10;
/// Workspaces come and go, and nothing tells when one is gone for good.
const MAX_WORKSPACES: usize = 20;

/// The pictures shown on the desktop and what was there before them. Every workspace has a
/// history of its own.
pub struct Gallery {
    dir: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct History {
    /// Oldest first. The first entry of a workspace is what the user had before this app
    /// changed anything there.
    entries: Vec<Backdrop>,
    /// What each workspace shows now, least recently changed first.
    shown: Vec<Backdrop>,
}

impl Gallery {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Writes a picture that is ready to be shown. The name changes with the content, so the
    /// system never confuses it with an older picture.
    pub fn save(&self, pixels: &Pixels) -> io::Result<PathBuf> {
        write_picture(&self.dir, pixels)
    }

    /// Takes over a picture written by [`write_picture`] somewhere else.
    pub fn adopt(&self, picture: &Path) -> io::Result<PathBuf> {
        let name = picture.file_name().ok_or(io::ErrorKind::InvalidInput)?;
        let path = self.dir.join(name);
        if !path.is_file() {
            write_whole(&path, &fs::read(picture)?)?;
        }
        Ok(path)
    }

    /// Remembers `backdrop` as the state to go back to. Must succeed before anything changes.
    pub fn remember(&self, mut backdrop: Backdrop) -> io::Result<()> {
        for picture in &mut backdrop.pictures {
            if !picture.path.starts_with(&self.dir) {
                picture.kept = self.keep(&picture.path);
            }
        }
        let workspace = backdrop.workspace.clone();
        let mut history = self.history();
        history.entries.push(backdrop);
        let own: Vec<usize> = (0..history.entries.len())
            .filter(|i| history.entries[*i].workspace == workspace)
            .collect();
        if own.len() > MAX_HISTORY {
            // The original stays reachable, however often the wallpaper changes.
            history.entries.remove(own[1]);
        }
        self.store(&history)
    }

    /// The state of `workspace` before the last change to it.
    pub fn previous(&self, workspace: Option<&str>) -> Option<Backdrop> {
        let mut history = self.history();
        let last = last_of(&history, workspace)?;
        Some(history.entries.swap_remove(last))
    }

    pub fn forget_previous(&self, workspace: Option<&str>) -> io::Result<()> {
        let mut history = self.history();
        if let Some(last) = last_of(&history, workspace) {
            history.entries.remove(last);
        }
        self.store(&history)
    }

    /// Records what a workspace shows now, and deletes the pictures no workspace shows or needs
    /// to go back.
    pub fn showing(&self, shown: Backdrop) -> io::Result<()> {
        let mut history = self.history();
        history.shown.retain(|b| b.workspace != shown.workspace);
        history.shown.push(shown);
        while history.shown.len() > MAX_WORKSPACES {
            let oldest = history.shown.remove(0).workspace;
            history.entries.retain(|b| b.workspace != oldest);
        }
        self.store(&history)?;
        self.clean(&history)
    }

    fn clean(&self, history: &History) -> io::Result<()> {
        let used: Vec<&Path> = history
            .entries
            .iter()
            .chain(&history.shown)
            .flat_map(|b| &b.pictures)
            .flat_map(|p| [Some(&p.path), p.kept.as_ref()])
            .flatten()
            .map(PathBuf::as_path)
            .collect();
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path != self.history_file() && !used.contains(&path.as_path()) && path.is_file() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    /// Losing the copy only matters if the original is deleted later, so errors are ignored.
    fn keep(&self, path: &Path) -> Option<PathBuf> {
        let size = fs::metadata(path).ok().filter(|m| m.is_file())?.len();
        if size > MAX_KEPT_BYTES {
            return None;
        }
        let bytes = fs::read(path).ok()?;
        let extension = path
            .extension()
            .map_or("img".into(), |e| e.to_string_lossy());
        let kept = self
            .dir
            .join(format!("kept-{:016x}.{extension}", hash(&bytes)));
        if !kept.is_file() {
            write_whole(&kept, &bytes).ok()?;
        }
        Some(kept)
    }

    fn history_file(&self) -> PathBuf {
        self.dir.join("history.json")
    }

    fn history(&self) -> History {
        let bytes = fs::read(self.history_file()).unwrap_or_default();
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    fn store(&self, history: &History) -> io::Result<()> {
        write_whole(&self.history_file(), &serde_json::to_vec_pretty(history)?)
    }
}

fn last_of(history: &History, workspace: Option<&str>) -> Option<usize> {
    history
        .entries
        .iter()
        .rposition(|b| b.workspace.as_deref() == workspace)
}

/// Writes a picture that is ready to be shown, under a name that depends on its content.
pub fn write_picture(dir: &Path, pixels: &Pixels) -> io::Result<PathBuf> {
    let (bytes, extension) = encode(pixels)?;
    let size = pixels.size;
    let name = format!(
        "{:016x}-{}x{}.{extension}",
        hash(&bytes),
        size.width,
        size.height
    );
    let path = dir.join(name);
    if !path.is_file() {
        write_whole(&path, &bytes)?;
    }
    Ok(path)
}

/// PNG, because Windows compresses JPEG wallpapers a second time.
fn encode(pixels: &Pixels) -> io::Result<(Vec<u8>, &'static str)> {
    let (width, height) = (pixels.size.width, pixels.size.height);
    let mut png = vec![];
    PngEncoder::new_with_quality(&mut png, CompressionType::Default, FilterType::Adaptive)
        .write_image(&pixels.rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(io::Error::other)?;
    if png.len() <= MAX_PNG_BYTES {
        return Ok((png, "png"));
    }
    let mut jpeg = vec![];
    JpegEncoder::new_with_quality(&mut jpeg, 97)
        .write_image(&pixels.rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(io::Error::other)?;
    Ok((jpeg, "jpg"))
}

/// Writes to a temporary file first, so that a crash never leaves half a file behind.
pub fn write_whole(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let part = path.with_extension("part");
    fs::write(&part, bytes)?;
    fs::rename(&part, path).inspect_err(|_| drop(fs::remove_file(&part)))
}

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wallpaper_core::{BackdropMode, Picture, Size};

    const HERE: Option<&str> = Some("here");

    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("wallpaper-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn showing(workspace: &str, path: impl AsRef<Path>) -> Backdrop {
        Backdrop {
            mode: BackdropMode::Picture,
            placement: 4,
            workspace: Some(workspace.into()),
            pictures: vec![Picture {
                screen: "one".into(),
                path: path.as_ref().into(),
                kept: None,
            }],
        }
    }

    fn pixels(seed: u8) -> Pixels {
        Pixels {
            size: Size::new(8, 4),
            rgb: (0..96).map(|i| i ^ seed).collect(),
        }
    }

    fn undo_all(gallery: &Gallery, workspace: Option<&str>) -> Vec<PathBuf> {
        let mut seen = vec![];
        while let Some(previous) = gallery.previous(workspace) {
            seen.push(previous.pictures[0].path.clone());
            gallery.forget_previous(workspace).unwrap();
        }
        seen
    }

    #[test]
    fn saved_pictures_are_lossless_and_named_by_content() {
        let folder = Folder::new("save");
        let gallery = Gallery::open(&folder.0).unwrap();
        let first = gallery.save(&pixels(1)).unwrap();
        assert_eq!(first, gallery.save(&pixels(1)).unwrap());
        assert_ne!(first, gallery.save(&pixels(2)).unwrap());
        assert_eq!(first.extension().unwrap(), "png");
        let back = image::open(&first).unwrap().into_rgb8().into_raw();
        assert_eq!(back, pixels(1).rgb);
    }

    #[test]
    fn going_back_survives_a_deleted_original() {
        let folder = Folder::new("kept");
        let original = folder.0.join("mine.jpg");
        fs::write(&original, b"my picture").unwrap();
        let gallery = Gallery::open(folder.0.join("gallery")).unwrap();
        gallery.remember(showing("here", &original)).unwrap();
        fs::remove_file(&original).unwrap();
        // Reopened, as after a restart.
        let gallery = Gallery::open(folder.0.join("gallery")).unwrap();
        let previous = gallery.previous(HERE).unwrap();
        let found = previous.pictures[0].existing().unwrap();
        assert_eq!(fs::read(found).unwrap(), b"my picture");
    }

    #[test]
    fn history_is_bounded_but_never_loses_the_original() {
        let folder = Folder::new("history");
        let gallery = Gallery::open(&folder.0).unwrap();
        for i in 0..25 {
            gallery.remember(showing("here", format!("/p{i}"))).unwrap();
        }
        let seen = undo_all(&gallery, HERE);
        assert_eq!(seen.len(), MAX_HISTORY);
        assert_eq!(seen[0], Path::new("/p24"));
        assert_eq!(seen[MAX_HISTORY - 1], Path::new("/p0"));
    }

    #[test]
    fn every_workspace_has_its_own_history() {
        let folder = Folder::new("workspaces");
        let gallery = Gallery::open(&folder.0).unwrap();
        for i in 0..15 {
            gallery.remember(showing("a", format!("/a{i}"))).unwrap();
            gallery.remember(showing("b", format!("/b{i}"))).unwrap();
        }
        let a = undo_all(&gallery, Some("a"));
        assert!(
            a.iter()
                .all(|p| p.starts_with("/") && p.to_string_lossy().starts_with("/a"))
        );
        assert_eq!(
            (a.len(), a[MAX_HISTORY - 1].as_path()),
            (MAX_HISTORY, Path::new("/a0"))
        );
        // Undoing one workspace left the other alone.
        let b = undo_all(&gallery, Some("b"));
        assert_eq!(
            (b.len(), b[MAX_HISTORY - 1].as_path()),
            (MAX_HISTORY, Path::new("/b0"))
        );
        assert!(gallery.previous(None).is_none());
    }

    #[test]
    fn cleaning_keeps_what_is_shown_or_needed() {
        let folder = Folder::new("clean");
        let gallery = Gallery::open(&folder.0).unwrap();
        let [old, before, shown] = [1, 2, 3].map(|seed| gallery.save(&pixels(seed)).unwrap());
        gallery.remember(showing("here", &before)).unwrap();
        gallery.showing(showing("here", &shown)).unwrap();
        let left = [&old, &before, &shown].map(|p| p.is_file());
        assert_eq!(left, [false, true, true]);
        assert!(gallery.previous(HERE).is_some());
    }

    #[test]
    fn cleaning_keeps_what_another_workspace_shows() {
        let folder = Folder::new("others");
        let gallery = Gallery::open(&folder.0).unwrap();
        let [first, second, third] = [1, 2, 3].map(|seed| gallery.save(&pixels(seed)).unwrap());
        gallery.showing(showing("a", &first)).unwrap();
        // Saved again, because nothing used it when "a" was recorded.
        let second_again = gallery.save(&pixels(2)).unwrap();
        assert_eq!(second, second_again);
        gallery.showing(showing("b", &second)).unwrap();
        assert!(first.is_file(), "workspace a still shows it");
        gallery.save(&pixels(3)).unwrap();
        gallery.showing(showing("a", &third)).unwrap();
        assert_eq!(
            [&first, &second, &third].map(|p| p.is_file()),
            [false, true, true]
        );
    }

    #[test]
    fn forgotten_workspaces_do_not_pile_up() {
        let folder = Folder::new("many");
        let gallery = Gallery::open(&folder.0).unwrap();
        for i in 0..MAX_WORKSPACES + 3 {
            let name = format!("w{i}");
            gallery.remember(showing(&name, "/before")).unwrap();
            gallery.showing(showing(&name, "/now")).unwrap();
        }
        assert!(gallery.previous(Some("w2")).is_none());
        assert!(gallery.previous(Some("w3")).is_some());
    }

    #[test]
    fn a_damaged_history_file_is_an_empty_history() {
        let folder = Folder::new("damaged");
        let gallery = Gallery::open(&folder.0).unwrap();
        fs::write(folder.0.join("history.json"), b"{oops").unwrap();
        assert!(gallery.previous(HERE).is_none());
        gallery.remember(showing("here", "/p")).unwrap();
        assert!(gallery.previous(HERE).is_some());
    }
}
