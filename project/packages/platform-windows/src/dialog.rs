use std::path::{Path, PathBuf};

use windows::{
    Win32::{
        Foundation::HWND,
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
            Threading::GetCurrentProcessId,
        },
        UI::{
            Shell::{
                FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog,
                SIGDN_FILESYSPATH, ShellExecuteW,
            },
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, SW_SHOWNORMAL},
        },
    },
    core::{HSTRING, w},
};

use crate::desktop::with_com;

/// Shows the folder dialog of Windows. Blocks until it is closed; nothing if the user cancels.
pub fn pick_folder() -> Option<PathBuf> {
    // The dialog belongs to the window the user just clicked in, if that is one of ours.
    // SAFETY: plain queries.
    let owner = unsafe {
        let front = GetForegroundWindow();
        let mut process = 0;
        GetWindowThreadProcessId(front, Some(&mut process));
        (process == GetCurrentProcessId()).then_some(front.0 as isize)
    };
    let picked = with_com(move || {
        // SAFETY: COM is initialized on this thread, and the returned string is freed below.
        let path = unsafe {
            let pick = || -> windows::core::Result<String> {
                let dialog: IFileOpenDialog =
                    CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
                let options = dialog.GetOptions()?;
                dialog.SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)?;
                dialog.SetTitle(w!("Chọn thư mục ảnh"))?;
                // Fails when the user cancels.
                dialog.Show(owner.map(|handle| HWND(handle as _)))?;
                let name = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
                let path = name.to_string().unwrap_or_default();
                CoTaskMemFree(Some(name.0.cast()));
                Ok(path)
            };
            pick().ok()
        };
        Ok(path)
    });
    picked
        .ok()
        .flatten()
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

/// Opens the folder of a file in Explorer, with the file selected.
pub fn reveal(file: &Path) -> bool {
    if !file.is_absolute() {
        return false;
    }
    // File names cannot contain quotes, so the path cannot leave its argument.
    let arguments = HSTRING::from(format!("/select,\"{}\"", file.display()));
    // SAFETY: every string outlives the call.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            w!("explorer.exe"),
            &arguments,
            None,
            SW_SHOWNORMAL,
        )
    };
    result.0 as isize > 32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    use windows::{
        Win32::{
            Foundation::{LPARAM, WPARAM},
            UI::WindowsAndMessaging::{
                EnumWindows, GetClassNameW, IsWindowVisible, PostMessageW, WM_CLOSE,
            },
        },
        core::BOOL,
    };

    #[test]
    fn refuses_to_reveal_what_is_not_a_full_path() {
        assert!(!reveal(Path::new("picture.jpg")));
        assert!(!reveal(Path::new("")));
    }

    /// Shows the real dialog for a moment and closes it again:
    /// `cargo test -- --ignored the_folder_dialog`.
    #[test]
    #[ignore = "opens a dialog on the screen"]
    fn the_folder_dialog_opens_and_can_be_cancelled() {
        unsafe extern "system" fn close_dialogs(window: HWND, found: LPARAM) -> BOOL {
            // SAFETY: `found` points to the counter below, which outlives the enumeration.
            unsafe {
                let mut process = 0;
                GetWindowThreadProcessId(window, Some(&mut process));
                let mut class = [0_u16; 32];
                let length = GetClassNameW(window, &mut class) as usize;
                let is_dialog = String::from_utf16_lossy(&class[..length]) == "#32770";
                if process == GetCurrentProcessId()
                    && is_dialog
                    && IsWindowVisible(window).as_bool()
                {
                    let _ = PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0));
                    *(found.0 as *mut u32) += 1;
                }
            }
            true.into()
        }
        let picking = thread::spawn(pick_folder);
        let mut closed = 0_u32;
        for _ in 0..100 {
            if picking.is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
            // SAFETY: the callback only runs during this call.
            let _ = unsafe { EnumWindows(Some(close_dialogs), LPARAM(&raw mut closed as isize)) };
        }
        assert!(picking.is_finished(), "the dialog did not close");
        assert_eq!(picking.join().unwrap(), None);
        assert!(closed > 0, "the dialog never appeared");
    }
}
