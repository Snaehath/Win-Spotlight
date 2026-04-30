use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;
use std::os::windows::ffi::OsStrExt;

/// Opens a path or URL securely using the native Windows ShellExecuteW API.
/// This bypasses the shell (PowerShell/CMD) and prevents command injection.
pub fn open_path_or_url(path: &str) -> Result<(), String> {
    let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    // SAFETY: ShellExecuteW is a standard Win32 API. We provide valid wide-string pointers
    // and use SW_SHOWNORMAL to ensure the launched process is visible to the user.
    unsafe {
        ShellExecuteW(
            None,
            windows::core::w!("open"),
            PCWSTR(wide_path.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
    }
    Ok(())
}
