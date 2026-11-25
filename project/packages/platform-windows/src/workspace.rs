use windows::{
    Win32::{
        Foundation::{HWND, LPARAM},
        System::{
            Com::{CLSCTX_ALL, CoCreateInstance},
            Registry::{
                HKEY, HKEY_CURRENT_USER, KEY_READ, RegCloseKey, RegEnumKeyExW, RegOpenKeyExW,
            },
        },
        UI::{
            Shell::{IVirtualDesktopManager, VirtualDesktopManager},
            WindowsAndMessaging::{EnumWindows, GetForegroundWindow, IsWindowVisible},
        },
    },
    core::{BOOL, GUID, PWSTR, w},
};

/// The id of the virtual desktop in front. COM must be initialized on this thread.
///
/// Windows only tells which desktop a window is on, so this asks windows that are in front.
/// On an empty desktop there is only an answer if it is the only one.
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
        .or_else(only_one)
}

/// The id of the virtual desktop if there is a single one. Where Windows keeps them is not
/// documented, so this is only used when nothing else helps.
fn only_one() -> Option<String> {
    let mut names = vec![];
    // SAFETY: the buffer and its length match, and the key is closed at the end.
    unsafe {
        let mut key = HKEY::default();
        let path =
            w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops\Desktops");
        RegOpenKeyExW(HKEY_CURRENT_USER, path, None, KEY_READ, &mut key)
            .ok()
            .ok()?;
        for index in 0..2 {
            let (mut name, mut length) = ([0_u16; 64], 64);
            let buffer = Some(PWSTR(name.as_mut_ptr()));
            let result = RegEnumKeyExW(key, index, buffer, &mut length, None, None, None, None);
            if result.is_ok() {
                names.push(String::from_utf16_lossy(&name[..length as usize]));
            }
        }
        let _ = RegCloseKey(key);
    }
    match &names[..] {
        [name] => Some(name.trim_matches(['{', '}']).to_uppercase()),
        _ => None,
    }
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
