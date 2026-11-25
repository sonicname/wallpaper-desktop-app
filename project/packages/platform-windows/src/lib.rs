//! Read-only Windows queries plus opening links in the default browser.
#![cfg(windows)]

use std::{mem, ptr};

use wallpaper_core::Size;
use windows_sys::Win32::{
    Graphics::Gdi::{
        DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE,
        DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplaySettingsExW,
    },
    UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Monitor {
    /// Physical pixels, already swapped for rotated screens and independent of DPI scaling.
    pub size: Size,
    pub primary: bool,
}

/// Active monitors, primary first.
pub fn monitors() -> Vec<Monitor> {
    let mut found = vec![];
    for index in 0.. {
        // SAFETY: both structs are plain data, and their size fields are set before use.
        let mut device: DISPLAY_DEVICEW = unsafe { mem::zeroed() };
        device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;
        if unsafe { EnumDisplayDevicesW(ptr::null(), index, &mut device, 0) } == 0 {
            break;
        }
        if device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
            continue;
        }
        let mut mode: DEVMODEW = unsafe { mem::zeroed() };
        mode.dmSize = mem::size_of::<DEVMODEW>() as u16;
        let name = device.DeviceName.as_ptr();
        if unsafe { EnumDisplaySettingsExW(name, ENUM_CURRENT_SETTINGS, &mut mode, 0) } != 0 {
            found.push(Monitor {
                size: Size::new(mode.dmPelsWidth, mode.dmPelsHeight),
                primary: device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0,
            });
        }
    }
    found.sort_by_key(|m| !m.primary);
    found
}

/// Opens an https link in the default browser. Other schemes are refused.
pub fn open_link(url: &str) -> bool {
    if !url.starts_with("https://") {
        return false;
    }
    let wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (verb, target) = (wide("open"), wide(url));
    // SAFETY: both strings are null terminated and outlive the call.
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    result as isize > 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_monitors_with_real_sizes_and_the_primary_first() {
        let all = monitors();
        assert!(
            all.iter()
                .all(|m| m.size.width >= 640 && m.size.height >= 480),
            "{all:?}"
        );
        assert!(all.first().is_none_or(|m| m.primary), "{all:?}");
    }

    #[test]
    fn refuses_links_that_are_not_https() {
        assert!(!open_link("file:///C:/Windows/notepad.exe"));
        assert!(!open_link("calc.exe"));
    }
}
