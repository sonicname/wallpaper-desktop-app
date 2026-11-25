use std::ffi::c_void;

use wallpaper_core::Autostart;
use windows::{
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Registry::{
            HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_BINARY, RRF_RT_REG_SZ, RegDeleteKeyValueW,
            RegGetValueW, RegSetKeyValueW,
        },
    },
    core::HSTRING,
};

use crate::wide;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Where the Task Manager and the Settings app record what the user turned off.
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

/// An entry in the list of programs Windows starts for this user. Needs no administrator.
pub struct Startup {
    run: HSTRING,
    approved: HSTRING,
    name: HSTRING,
    command: String,
}

impl Startup {
    pub fn new(name: &str, command: String) -> Self {
        Self::at(RUN, APPROVED, name, command)
    }

    fn at(run: &str, approved: &str, name: &str, command: String) -> Self {
        Self {
            run: run.into(),
            approved: approved.into(),
            name: name.into(),
            command,
        }
    }

    pub fn state(&self) -> Autostart {
        if !self.exists() {
            Autostart::Off
        } else if self.is_blocked() {
            Autostart::Blocked
        } else {
            Autostart::On
        }
    }

    /// Never touches the choice the user made in the Task Manager.
    pub fn set(&self, on: bool) -> Result<Autostart, String> {
        let result = if on {
            let command = wide(&self.command);
            // SAFETY: the pointer and the length describe the same null terminated string.
            unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    &self.run,
                    &self.name,
                    REG_SZ.0,
                    Some(command.as_ptr().cast::<c_void>()),
                    size_of_val(&command[..]) as u32,
                )
            }
        } else {
            // SAFETY: both names outlive the call.
            unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, &self.run, &self.name) }
        };
        if result.is_ok() || (!on && result == ERROR_FILE_NOT_FOUND) {
            Ok(self.state())
        } else {
            Err(result.to_hresult().message())
        }
    }

    fn exists(&self) -> bool {
        // SAFETY: only asks whether the value is there.
        unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                &self.run,
                &self.name,
                RRF_RT_REG_SZ,
                None,
                None,
                None,
            )
        }
        .is_ok()
    }

    /// Not documented: an odd first byte means turned off.
    fn is_blocked(&self) -> bool {
        let mut value = [0_u8; 12];
        let mut bytes = value.len() as u32;
        // SAFETY: the buffer and its size match.
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                &self.approved,
                &self.name,
                RRF_RT_REG_BINARY,
                None,
                Some(value.as_mut_ptr().cast()),
                Some(&mut bytes),
            )
        };
        result.is_ok() && value[0] & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Registry::{REG_BINARY, RegDeleteTreeW};

    /// A key of its own, so that the real startup list is never touched.
    const ROOT: &str = r"Software\WallpaperDesktopTest";

    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &HSTRING::from(ROOT)) };
        }
    }

    #[test]
    fn can_be_turned_on_and_off_and_respects_the_task_manager() {
        let _cleanup = Cleanup;
        let (run, approved) = (format!(r"{ROOT}\Run"), format!(r"{ROOT}\Approved"));
        let startup = Startup::at(
            &run,
            &approved,
            "Test",
            r#""C:\a b\app.exe" --background"#.into(),
        );
        assert_eq!(startup.state(), Autostart::Off);
        assert_eq!(startup.set(false), Ok(Autostart::Off));
        assert_eq!(startup.set(true), Ok(Autostart::On));
        assert_eq!(startup.set(true), Ok(Autostart::On));
        // As the Task Manager writes it.
        let blocked = [3_u8, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];
        let result = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                &HSTRING::from(&approved),
                &HSTRING::from("Test"),
                REG_BINARY.0,
                Some(blocked.as_ptr().cast()),
                blocked.len() as u32,
            )
        };
        assert!(result.is_ok());
        assert_eq!(startup.state(), Autostart::Blocked);
        assert_eq!(startup.set(true), Ok(Autostart::Blocked));
        assert_eq!(startup.set(false), Ok(Autostart::Off));
    }
}
