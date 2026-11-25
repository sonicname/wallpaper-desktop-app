use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    /// Identifies the screen to the operating system. Not meant for display.
    pub id: String,
    /// Physical pixels, already swapped for rotated screens and independent of DPI scaling.
    pub size: Size,
    pub primary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackdropMode {
    Picture,
    Solid,
    Slideshow,
    Spotlight,
}

impl BackdropMode {
    /// Whether [`Desktop::restore`] can bring this mode back after a picture was set.
    pub fn can_be_restored(self) -> bool {
        matches!(self, Self::Picture | Self::Solid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Picture {
    pub screen: String,
    /// Empty when the screen shows no picture.
    pub path: PathBuf,
    /// A copy to fall back to if the original file disappears.
    pub kept: Option<PathBuf>,
}

impl Picture {
    /// The file to show, if it still exists.
    pub fn existing(&self) -> Option<&Path> {
        [Some(&self.path), self.kept.as_ref()]
            .into_iter()
            .flatten()
            .find(|p| p.is_file())
            .map(PathBuf::as_path)
    }
}

/// What the desktop looks like, in enough detail to bring it back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backdrop {
    pub mode: BackdropMode,
    /// How pictures are placed on the screen. Only meaningful to the operating system.
    pub placement: i32,
    pub pictures: Vec<Picture>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopError {
    /// The system accepted the call but shows something else, usually because of a policy.
    NotApplied,
    /// The picture to bring back no longer exists.
    Missing(PathBuf),
    System(String),
}

impl fmt::Display for DesktopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotApplied => write!(f, "the system kept the old wallpaper"),
            Self::Missing(path) => write!(f, "{} no longer exists", path.display()),
            Self::System(reason) => f.write_str(reason),
        }
    }
}

impl std::error::Error for DesktopError {}

/// The wallpaper of the current user. Calls may block for a short time.
pub trait Desktop: Send + Sync {
    /// Connected screens, primary first.
    fn screens(&self) -> Vec<Screen>;
    fn read(&self) -> Result<Backdrop, DesktopError>;
    /// Shows one picture per screen. Each picture must already have the size of its screen.
    fn apply(&self, pictures: &[(String, PathBuf)]) -> Result<(), DesktopError>;
    fn restore(&self, backdrop: &Backdrop) -> Result<(), DesktopError>;
}
