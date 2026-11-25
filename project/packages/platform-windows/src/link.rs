use windows::{
    Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    core::{HSTRING, w},
};

/// Opens an https link in the default browser. Other schemes are refused.
pub fn open_link(url: &str) -> bool {
    if !url.starts_with("https://") {
        return false;
    }
    // SAFETY: every string outlives the call.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            &HSTRING::from(url),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    result.0 as isize > 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_links_that_are_not_https() {
        assert!(!open_link("file:///C:/Windows/notepad.exe"));
        assert!(!open_link("calc.exe"));
    }
}
