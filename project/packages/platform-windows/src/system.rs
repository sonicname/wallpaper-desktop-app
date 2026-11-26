use std::path::PathBuf;

use wallpaper_core::{Autostart, LocalTime, System};
use windows::Win32::{
    System::SystemInformation::GetLocalTime,
    UI::Shell::{
        QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
        SHQueryUserNotificationState,
    },
};

use crate::{autostart::Startup, dialog};

pub struct WindowsSystem {
    startup: Startup,
}

impl WindowsSystem {
    /// `name` identifies the app in the startup list. `command` is what Windows runs.
    pub fn new(name: &str, command: String) -> Self {
        Self {
            startup: Startup::new(name, command),
        }
    }
}

impl System for WindowsSystem {
    fn now(&self) -> LocalTime {
        // SAFETY: no arguments, always succeeds.
        let now = unsafe { GetLocalTime() };
        LocalTime::new(
            now.wYear.into(),
            now.wMonth.into(),
            now.wDay.into(),
            now.wHour.into(),
            now.wMinute.into(),
        )
    }

    fn do_not_disturb(&self) -> bool {
        // SAFETY: plain query.
        let state = unsafe { SHQueryUserNotificationState() };
        state.is_ok_and(|state| {
            [
                QUNS_BUSY,
                QUNS_RUNNING_D3D_FULL_SCREEN,
                QUNS_PRESENTATION_MODE,
            ]
            .contains(&state)
        })
    }

    fn autostart(&self) -> Autostart {
        self.startup.state()
    }

    fn set_autostart(&self, on: bool) -> Result<Autostart, String> {
        self.startup.set(on)
    }

    fn pick_folder(&self) -> Option<PathBuf> {
        dialog::pick_folder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_is_plausible() {
        let now = WindowsSystem::new("unused", String::new()).now();
        assert!(now.day > LocalTime::new(2026, 11, 1, 0, 0).day, "{now:?}");
        assert!(now.minute < 24 * 60, "{now:?}");
    }
}
