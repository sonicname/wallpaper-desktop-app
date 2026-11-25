use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        System::Com::{CLSCTX_ALL, CoCreateInstance},
        UI::{
            Shell::{IVirtualDesktopManager, VirtualDesktopManager},
            WindowsAndMessaging::{EnumWindows, GetForegroundWindow, IsWindowVisible},
        },
    },
    core::{BOOL, GUID},
};

/// The id of the virtual desktop in front. COM must be initialized on this thread.
///
/// Windows only tells which desktop a window is on, so this asks windows that are in front.
/// There is no answer on an empty desktop.
pub fn current() -> Option<String> {
    // SAFETY: plain COM and window calls; a window that closes in between only fails the call.
    let manager: IVirtualDesktopManager =
        unsafe { CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL) }.ok()?;
    let id_of = |window: HWND| unsafe {
        let in_front = manager.IsWindowOnCurrentVirtualDesktop(window).ok()?;
        let id = manager.GetWindowDesktopId(window).ok()?;
        // Windows pinned to every desktop have an empty id.
        (in_front.as_bool() && id != GUID::zeroed()).then(|| format!("{id:?}"))
    };
    id_of(unsafe { GetForegroundWindow() })
        .or_else(|| visible_windows().into_iter().find_map(id_of))
}

fn visible_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(window: HWND, list: LPARAM) -> BOOL {
        // SAFETY: `list` is the vector below, which outlives the enumeration.
        unsafe {
            if IsWindowVisible(window).as_bool() {
                (*(list.0 as *mut Vec<HWND>)).push(window);
            }
        }
        true.into()
    }
    let mut windows: Vec<HWND> = vec![];
    // SAFETY: the callback only runs during this call.
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&raw mut windows as isize)) };
    windows
}
