use std::mem;

use wallpaper_core::{Screen, Size};
use windows::{
    Win32::{
        Graphics::Gdi::{
            DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE,
            DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplaySettingsExW,
        },
        UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
    core::PCWSTR,
};

/// Connected screens, primary first. The ids are the device paths `IDesktopWallpaper` uses.
///
/// Display modes are in physical pixels whatever the DPI awareness of the process is, unlike
/// `IDesktopWallpaper::GetMonitorRECT`.
pub fn screens() -> Vec<Screen> {
    let mut found = vec![];
    for adapter in (0..).map_while(|index| device(PCWSTR::null(), index, 0)) {
        let has = |flag: windows::Win32::Graphics::Gdi::DISPLAY_DEVICE_STATE_FLAGS| {
            adapter.StateFlags.0 & flag.0 != 0
        };
        if !has(DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) {
            continue;
        }
        let name = PCWSTR(adapter.DeviceName.as_ptr());
        let mut mode = DEVMODEW {
            dmSize: mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        // SAFETY: the name is null terminated and the size field of `mode` is set.
        let known = unsafe {
            EnumDisplaySettingsExW(name, ENUM_CURRENT_SETTINGS, &mut mode, Default::default())
        };
        let Some(monitor) = device(name, 0, EDD_GET_DEVICE_INTERFACE_NAME) else {
            continue;
        };
        if known.as_bool() {
            let end = monitor.DeviceID.iter().position(|c| *c == 0);
            let id = &monitor.DeviceID[..end.unwrap_or(monitor.DeviceID.len())];
            found.push(Screen {
                id: String::from_utf16_lossy(id),
                size: Size::new(mode.dmPelsWidth, mode.dmPelsHeight),
                primary: has(DISPLAY_DEVICE_PRIMARY_DEVICE),
            });
        }
    }
    found.sort_by_key(|screen| !screen.primary);
    found
}

fn device(parent: PCWSTR, index: u32, flags: u32) -> Option<DISPLAY_DEVICEW> {
    let mut device = DISPLAY_DEVICEW {
        cb: mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    // SAFETY: `parent` is null or null terminated and the size field of `device` is set.
    unsafe { EnumDisplayDevicesW(parent, index, &mut device, flags) }
        .as_bool()
        .then_some(device)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_screens_with_real_sizes_and_the_primary_first() {
        let all = screens();
        assert!(
            all.iter()
                .all(|s| s.size.width >= 640 && s.size.height >= 480 && !s.id.is_empty()),
            "{all:?}"
        );
        assert!(all.first().is_none_or(|s| s.primary), "{all:?}");
    }
}
