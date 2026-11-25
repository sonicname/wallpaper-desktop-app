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
const MAX_HISTORY: usize = 10;

/// The pictures shown on the desktop and what was there before them.
pub struct Gallery {
    dir: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct History {
    /// Oldest first. The first entry is what the user had before this app changed anything.
    entries: Vec<Backdrop>,
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
        let (bytes, extension) = encode(pixels)?;
        let size = pixels.size;
        let name = format!(
            "{:016x}-{}x{}.{extension}",
            hash(&bytes),
            size.width,
            size.height
        );
        let path = self.dir.join(name);
        if !path.is_file() {
            write_whole(&path, &bytes)?;
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
        let mut history = self.history();
        history.entries.push(backdrop);
        if history.entries.len() > MAX_HISTORY {
            // The original stays reachable, however often the wallpaper changes.
            history.entries.remove(1);
        }
        self.store(&history)
    }

    /// The state before the last change.
    pub fn previous(&self) -> Option<Backdrop> {
        self.history().entries.pop()
    }

    pub fn forget_previous(&self) -> io::Result<()> {
        let mut history = self.history();
        history.entries.pop();
        self.store(&history)
    }

    /// Deletes pictures that are neither shown nor needed to go back.
    pub fn clean(&self, shown: &Backdrop) -> io::Result<()> {
        let history = self.history();
        let all = history.entries.iter().chain([shown]);
        let used: Vec<&Path> = all
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

fn write_whole(path: &Path, bytes: &[u8]) -> io::Result<()> {
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

    fn showing(path: &Path) -> Backdrop {
        Backdrop {
            mode: BackdropMode::Picture,
            placement: 4,
            pictures: vec![Picture {
                screen: "one".into(),
                path: path.into(),
                kept: None,
            }],
        }
    }

    fn pixels(seed: u8) -> Pixels {
        let size = Size::new(8, 4);
        Pixels {
            size,
            rgb: (0..96).map(|i| i ^ seed).collect(),
        }
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
        gallery.remember(showing(&original)).unwrap();
        fs::remove_file(&original).unwrap();
        // Reopened, as after a restart.
        let previous = Gallery::open(folder.0.join("gallery"))
            .unwrap()
            .previous()
            .unwrap();
        let found = previous.pictures[0].existing().unwrap();
        assert_eq!(fs::read(found).unwrap(), b"my picture");
    }

    #[test]
    fn history_is_bounded_but_never_loses_the_original() {
        let folder = Folder::new("history");
        let gallery = Gallery::open(&folder.0).unwrap();
        for i in 0..25 {
            gallery
                .remember(showing(Path::new(&format!("/p{i}"))))
                .unwrap();
        }
        let mut seen = vec![];
        while let Some(previous) = gallery.previous() {
            seen.push(previous.pictures[0].path.clone());
            gallery.forget_previous().unwrap();
        }
        assert_eq!(seen.len(), MAX_HISTORY);
        assert_eq!(seen[0], Path::new("/p24"));
        assert_eq!(seen[MAX_HISTORY - 1], Path::new("/p0"));
    }

    #[test]
    fn cleaning_keeps_what_is_shown_or_needed() {
        let folder = Folder::new("clean");
        let gallery = Gallery::open(&folder.0).unwrap();
        let [old, before, shown] = [1, 2, 3].map(|seed| gallery.save(&pixels(seed)).unwrap());
        gallery.remember(showing(&before)).unwrap();
        gallery.clean(&showing(&shown)).unwrap();
        assert_eq!(
            [old.is_file(), before.is_file(), shown.is_file()],
            [false, true, true]
        );
        assert!(gallery.previous().is_some());
    }

    #[test]
    fn a_damaged_history_file_is_an_empty_history() {
        let folder = Folder::new("damaged");
        let gallery = Gallery::open(&folder.0).unwrap();
        fs::write(folder.0.join("history.json"), b"{oops").unwrap();
        assert!(gallery.previous().is_none());
        gallery.remember(showing(Path::new("/p"))).unwrap();
        assert!(gallery.previous().is_some());
    }
}
