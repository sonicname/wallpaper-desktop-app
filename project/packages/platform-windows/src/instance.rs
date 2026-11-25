use std::thread;

use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, WAIT_OBJECT_0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};

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
