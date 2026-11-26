use std::{ffi::c_void, thread};

use wallpaper_core::Rgb;
use windows::{
    Win32::{
        Foundation::{
            COLORREF, CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, WAIT_OBJECT_0,
        },
        Graphics::Dwm::{
            DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE,
            DWMWINDOWATTRIBUTE, DwmSetWindowAttribute,
        },
        System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForSingleObject},
        UI::WindowsAndMessaging::{
            ASFW_ANY, AllowSetForegroundWindow, IsIconic, SW_RESTORE, SetForegroundWindow,
            ShowWindow,
        },
    },
    core::HSTRING,
};

/// Held by the first running copy of the app.
pub struct Instance {
    event: HANDLE,
}

impl Instance {
    /// Returns `None` if a copy is running already, after asking it to come to the front.
    pub fn acquire(name: &str) -> Option<Self> {
        let name = HSTRING::from(format!(r"Local\{name}"));
        // SAFETY: the name outlives the call, and the last error is read right after it.
        let (event, exists) = unsafe {
            let event = CreateEventW(None, false, false, &name).ok()?;
            (event, GetLastError() == ERROR_ALREADY_EXISTS)
        };
        if !exists {
            return Some(Self { event });
        }
        // SAFETY: the handle was just opened. The other copy may only raise its window if this
        // one, which the user just started, allows it.
        unsafe {
            let _ = AllowSetForegroundWindow(ASFW_ANY);
            let _ = SetEvent(event);
            let _ = CloseHandle(event);
        }
        None
    }

    /// Calls `wanted` on another thread whenever the user starts the app again.
    pub fn on_start_again(self, wanted: impl Fn() + Send + 'static) {
        // The handle stays open until the process ends, like the thread that waits for it.
        let event = self.event.0 as usize;
        thread::spawn(move || {
            let event = HANDLE(event as _);
            // SAFETY: the handle is never closed.
            while unsafe { WaitForSingleObject(event, INFINITE) } == WAIT_OBJECT_0 {
                wanted();
            }
        });
    }
}

/// Brings a window of this process to the front.
pub fn raise(window: isize) {
    let window = HWND(window as _);
    // SAFETY: a stale handle only makes the calls fail.
    unsafe {
        if IsIconic(window).as_bool() {
            let _ = ShowWindow(window, SW_RESTORE);
        }
        let _ = SetForegroundWindow(window);
    }
}

/// Gives the title bar of a window the colour of its content, with light buttons. Windows 10
/// only knows the dark title bar and ignores the colours. Returns whether anything was accepted.
pub fn tint_title_bar(window: isize, color: Rgb, border: Rgb) -> bool {
    let window = HWND(window as _);
    let set = |attribute: DWMWINDOWATTRIBUTE, value: u32| {
        let value: *const c_void = (&raw const value).cast();
        // SAFETY: the pointer and the size describe the same four bytes; a stale handle fails.
        unsafe { DwmSetWindowAttribute(window, attribute, value, 4) }.is_ok()
    };
    let packed = |Rgb(r, g, b): Rgb| COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16).0;
    let dark = set(DWMWA_USE_IMMERSIVE_DARK_MODE, 1);
    let caption = set(DWMWA_CAPTION_COLOR, packed(color));
    let edge = set(DWMWA_BORDER_COLOR, packed(border));
    dark || caption || edge
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

    #[test]
    fn a_window_that_is_gone_is_not_tinted() {
        assert!(!tint_title_bar(0, Rgb(0, 0, 0), Rgb(0, 0, 0)));
    }

    #[test]
    fn the_second_copy_calls_the_first() {
        let name = format!("WallpaperDesktopTest-{}", std::process::id());
        let first = Instance::acquire(&name).expect("the first copy");
        let (send, called) = mpsc::channel();
        first.on_start_again(move || send.send(()).unwrap());
        for _ in 0..2 {
            assert!(Instance::acquire(&name).is_none());
            called.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        assert!(called.recv_timeout(Duration::from_millis(100)).is_err());
    }
}
