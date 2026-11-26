use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use wallpaper_core::{Backdrop, BackdropMode, Desktop, DesktopError, Picture, Screen};
use windows::{
    Win32::{
        Foundation::{RECT, S_OK},
        System::{
            Com::{
                CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemFree, CoUninitialize,
            },
            Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        },
        UI::Shell::{
            DESKTOP_WALLPAPER_POSITION, DSS_SLIDESHOW, DWPOS_FILL, DesktopWallpaper,
            IDesktopWallpaper,
        },
    },
    core::{HSTRING, Interface, PCWSTR, PWSTR, w},
};

use crate::{screens::screens, wide, workspace};

const PATIENCE: Duration = Duration::from_secs(3);

/// The wallpaper of the current virtual desktop. Windows has no public way to reach the others.
pub struct WindowsDesktop;

impl Desktop for WindowsDesktop {
    fn screens(&self) -> Vec<Screen> {
        screens()
    }

    fn workspace(&self) -> Option<String> {
        with_com(|| Ok(workspace::current())).ok().flatten()
    }

    fn read(&self) -> Result<Backdrop, DesktopError> {
        with_shell(|shell| {
            // Do not mix two workspaces if the user switches right now.
            for _ in 0..3 {
                let backdrop = shell.read()?;
                if workspace::current() == backdrop.workspace {
                    return Ok(backdrop);
                }
            }
            Err(DesktopError::OtherWorkspace)
        })
    }

    fn apply(
        &self,
        pictures: &[(String, PathBuf)],
        workspace: Option<&str>,
    ) -> Result<(), DesktopError> {
        with_shell(|shell| {
            require(workspace)?;
            let wanted: Vec<_> = pictures
                .iter()
                .map(|(screen, path)| (screen.as_str(), path.as_path()))
                .collect();
            shell.show(&wanted)?;
            // Pictures already have the size of their screen, so this only matters later, for
            // example after the resolution changes.
            shell.place(DWPOS_FILL)?;
            shell.settle(&wanted, workspace)
        })
    }

    fn restore(&self, backdrop: &Backdrop) -> Result<(), DesktopError> {
        // Decide everything first, so that a missing file leaves the desktop untouched.
        let connected = screens();
        let is_connected = |p: &&Picture| connected.iter().any(|s| s.id == p.screen);
        let mut wanted = vec![];
        for picture in backdrop.pictures.iter().filter(is_connected) {
            let path = if picture.path.as_os_str().is_empty() {
                Path::new("")
            } else {
                picture
                    .existing()
                    .ok_or_else(|| DesktopError::Missing(picture.path.clone()))?
            };
            wanted.push((picture.screen.as_str(), path));
        }
        with_shell(|shell| {
            let workspace = backdrop.workspace.as_deref();
            require(workspace)?;
            shell.show(&wanted)?;
            shell.place(DESKTOP_WALLPAPER_POSITION(backdrop.placement))?;
            shell.settle(&wanted, workspace)
        })
    }
}

struct Shell(IDesktopWallpaper);

impl Shell {
    fn read(&self) -> Result<Backdrop, DesktopError> {
        let workspace = workspace::current();
        let mut pictures = vec![];
        // SAFETY: plain COM calls; returned strings are freed by `take`.
        let count = unsafe { self.0.GetMonitorDevicePathCount() }.map_err(system)?;
        for index in 0..count {
            let screen = take(unsafe { self.0.GetMonitorDevicePathAt(index) }.map_err(system)?);
            if self.is_attached(&screen) {
                let path = self.get(&screen)?;
                pictures.push(Picture {
                    screen,
                    path,
                    kept: None,
                });
            }
        }
        let placement = unsafe { self.0.GetPosition() }.map_err(system)?.0;
        Ok(Backdrop {
            mode: self.mode(&pictures),
            placement,
            workspace,
            pictures,
        })
    }

    /// The safe wrapper hides `S_FALSE`, which is how detached screens are reported.
    fn is_attached(&self, screen: &str) -> bool {
        let id = wide(screen);
        let mut rect = RECT::default();
        // SAFETY: the object is alive, and both pointers are valid for the call.
        let result = unsafe {
            (self.0.vtable().GetMonitorRECT)(self.0.as_raw(), PCWSTR(id.as_ptr()), &mut rect)
        };
        result == S_OK
    }

    fn get(&self, screen: &str) -> Result<PathBuf, DesktopError> {
        // SAFETY: the returned string is freed by `take`.
        let path = unsafe { self.0.GetWallpaper(&HSTRING::from(screen)) }.map_err(system)?;
        Ok(PathBuf::from(take(path)))
    }

    /// One picture for every screen is set as such. Setting it screen by screen looks the same,
    /// but moves Windows to its per-screen bookkeeping, and going back would not be exact.
    fn show(&self, pictures: &[(&str, &Path)]) -> Result<(), DesktopError> {
        let connected: Vec<String> = screens().into_iter().map(|s| s.id).collect();
        if let Some(path) = common_picture(pictures, &connected) {
            return self.set(None, path);
        }
        pictures
            .iter()
            .try_for_each(|(screen, path)| self.set(Some(screen), path))
    }

    fn set(&self, screen: Option<&str>, path: &Path) -> Result<(), DesktopError> {
        let path = HSTRING::from(path.as_os_str());
        let screen = screen.map(HSTRING::from);
        let id = screen
            .as_ref()
            .map_or(PCWSTR::null(), |s| PCWSTR(s.as_ptr()));
        // SAFETY: both strings outlive the call, and null means every screen.
        unsafe { self.0.SetWallpaper(id, &path) }.map_err(system)
    }

    /// Waits until the pictures can be read back. Explorer takes about half a second, and until
    /// then reading returns the old state, which must not end up in the undo history.
    fn settle(
        &self,
        pictures: &[(&str, &Path)],
        workspace: Option<&str>,
    ) -> Result<(), DesktopError> {
        let shown = |(screen, path): &(&str, &Path)| {
            path.as_os_str().is_empty() || self.get(screen).is_ok_and(|now| same_file(&now, path))
        };
        let start = Instant::now();
        loop {
            // Right after a switch no window may be in front yet, which means nothing.
            let now = workspace::current();
            if workspace.is_some() && now.is_some() && now.as_deref() != workspace {
                return Err(DesktopError::WorkspaceSwitched);
            }
            if pictures.iter().all(shown) {
                return Ok(());
            }
            if start.elapsed() > PATIENCE {
                return Err(DesktopError::NotApplied);
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn place(&self, position: DESKTOP_WALLPAPER_POSITION) -> Result<(), DesktopError> {
        // SAFETY: plain COM calls.
        if unsafe { self.0.GetPosition() }.map_err(system)? == position {
            return Ok(());
        }
        unsafe { self.0.SetPosition(position) }.map_err(system)
    }

    fn mode(&self, pictures: &[Picture]) -> BackdropMode {
        // SAFETY: plain COM call.
        let status = unsafe { self.0.GetStatus() }.unwrap_or_default();
        match background_type() {
            // Not documented. 3 is used since Windows 11 for Spotlight.
            Some(3) => BackdropMode::Spotlight,
            _ if status.0 & DSS_SLIDESHOW.0 != 0 => BackdropMode::Slideshow,
            _ if pictures.iter().all(|p| p.path.as_os_str().is_empty()) => BackdropMode::Solid,
            _ => BackdropMode::Picture,
        }
    }
}

/// The picture shared by all connected screens, if `pictures` covers them with a single one.
fn common_picture<'a>(pictures: &[(&str, &'a Path)], connected: &[String]) -> Option<&'a Path> {
    let (_, first) = pictures.first()?;
    let covered = connected
        .iter()
        .all(|id| pictures.iter().any(|(screen, _)| screen == id));
    let same = pictures.iter().all(|(_, path)| same_file(path, first));
    (covered && same && !connected.is_empty()).then_some(*first)
}

fn background_type() -> Option<u32> {
    let (mut value, mut bytes) = (0_u32, 4_u32);
    // SAFETY: the buffer and its size describe the same four bytes.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Wallpapers"),
            w!("BackgroundType"),
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut value).cast()),
            Some(&mut bytes),
        )
    };
    result.is_ok().then_some(value)
}

fn require(workspace: Option<&str>) -> Result<(), DesktopError> {
    if workspace::current().as_deref() == workspace {
        Ok(())
    } else {
        Err(DesktopError::OtherWorkspace)
    }
}

/// Creating the object takes about a millisecond, and a fresh one never outlives an Explorer
/// restart.
fn with_shell<T: Send>(
    work: impl FnOnce(&Shell) -> Result<T, DesktopError> + Send,
) -> Result<T, DesktopError> {
    with_com(|| {
        // SAFETY: COM is initialized on this thread.
        let shell = unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL) };
        work(&Shell(shell.map_err(system)?))
    })
}

/// Runs `work` on a thread of its own, because the caller's thread may already have chosen
/// another COM apartment.
pub(crate) fn with_com<T: Send>(
    work: impl FnOnce() -> Result<T, DesktopError> + Send,
) -> Result<T, DesktopError> {
    let run = || {
        // SAFETY: the matching `CoUninitialize` runs after `work` released its objects.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(system)?;
        let result = work();
        unsafe { CoUninitialize() };
        result
    };
    thread::scope(|scope| scope.spawn(run).join())
        .unwrap_or_else(|_| Err(DesktopError::System("the wallpaper thread stopped".into())))
}

fn take(text: PWSTR) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: COM returned a null terminated string that this function now owns.
    unsafe {
        let copy = text.to_string().unwrap_or_default();
        CoTaskMemFree(Some(text.0.cast()));
        copy
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    let key = |path: &Path| path.to_string_lossy().replace('/', r"\").to_lowercase();
    key(a) == key(b)
}

fn system(error: windows::core::Error) -> DesktopError {
    DesktopError::System(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Changes the real wallpaper for a moment. Run it with
    /// `cargo test -- --ignored round_trip` and `WALLPAPER_TEST_PICTURE` set to a file.
    #[test]
    #[ignore = "changes the wallpaper"]
    fn round_trip_is_exact_and_immediately_readable() {
        let picture = PathBuf::from(std::env::var("WALLPAPER_TEST_PICTURE").unwrap());
        let before = WindowsDesktop.read().unwrap();
        let wanted: Vec<_> = before
            .pictures
            .iter()
            .map(|p| (p.screen.clone(), picture.clone()))
            .collect();
        for _ in 0..2 {
            let applied = WindowsDesktop.apply(&wanted, before.workspace.as_deref());
            let during = WindowsDesktop.read().unwrap();
            let restored = WindowsDesktop.restore(&before);
            assert_eq!((applied, restored), (Ok(()), Ok(())));
            assert!(during.pictures.iter().all(|p| same_file(&p.path, &picture)));
            assert_eq!(WindowsDesktop.read().unwrap(), before);
        }
    }

    #[test]
    fn reads_the_wallpaper_of_every_connected_screen() {
        let backdrop = WindowsDesktop.read().unwrap();
        let mut read: Vec<_> = backdrop.pictures.iter().map(|p| p.screen.clone()).collect();
        let mut known: Vec<_> = screens().into_iter().map(|s| s.id).collect();
        read.sort();
        known.sort();
        assert_eq!(
            read, known,
            "both sources must name the screens the same way"
        );
    }

    #[test]
    fn a_missing_picture_is_refused_before_anything_changes() {
        let before = WindowsDesktop.read().unwrap();
        let mut broken = before.clone();
        let Some(picture) = broken.pictures.first_mut() else {
            return;
        };
        picture.path = PathBuf::from(r"C:\does\not\exist.png");
        let result = WindowsDesktop.restore(&broken);
        assert!(
            matches!(result, Err(DesktopError::Missing(_))),
            "{result:?}"
        );
        assert_eq!(WindowsDesktop.read().unwrap(), before);
    }

    /// The fix for changing the wrong virtual desktop: both calls must refuse, and change nothing.
    #[test]
    fn another_workspace_is_never_touched() {
        let before = WindowsDesktop.read().unwrap();
        let elsewhere = Some("00000000-0000-0000-0000-000000000001".to_owned());
        assert_ne!(before.workspace, elsewhere);
        // Asks for what is already shown, so that even a broken check could do no harm.
        let pictures: Vec<_> = before
            .pictures
            .iter()
            .map(|p| (p.screen.clone(), p.path.clone()))
            .collect();
        let applied = WindowsDesktop.apply(&pictures, elsewhere.as_deref());
        let foreign = Backdrop {
            workspace: elsewhere,
            ..before.clone()
        };
        let restored = WindowsDesktop.restore(&foreign);
        assert_eq!(applied, Err(DesktopError::OtherWorkspace));
        assert_eq!(restored, Err(DesktopError::OtherWorkspace));
        assert_eq!(WindowsDesktop.read().unwrap(), before);
    }

    #[test]
    fn a_shared_picture_must_cover_every_connected_screen() {
        let (a, b) = (Path::new(r"C:\a.png"), Path::new(r"C:\b.png"));
        let both = ["one".to_owned(), "two".to_owned()];
        assert_eq!(common_picture(&[("one", a)], &both[..1]), Some(a));
        assert_eq!(common_picture(&[("one", a), ("two", a)], &both), Some(a));
        assert_eq!(common_picture(&[("one", a), ("two", b)], &both), None);
        assert_eq!(common_picture(&[("one", a)], &both), None);
        assert_eq!(common_picture(&[], &both), None);
    }

    #[test]
    fn compares_paths_like_windows_does() {
        assert!(same_file(Path::new(r"C:\A\b.PNG"), Path::new("c:/a/B.png")));
        assert!(!same_file(Path::new(r"C:\a.png"), Path::new(r"C:\b.png")));
    }
}
