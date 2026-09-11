use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningAppInfo {
    pub is_running: bool,
    pub app_name: String,
    pub exe_name: String,
    pub kind: String, // "window" or "instance"
}

/// Returns a set of all currently running executable names in lower-case (e.g. "chrome.exe", "code.exe").
pub fn get_running_process_names() -> HashSet<String> {
    let mut names = HashSet::new();

    unsafe {
        let snapshot: HANDLE = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return names,
        };

        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                // Find null terminator in szExeFile
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&entry.szExeFile[..len])
                    .trim()
                    .to_lowercase();
                if !exe_name.is_empty() {
                    names.insert(exe_name);
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    names
}

/// Extracts any candidate `.exe` names from a `.lnk` file's binary content.
/// Windows shortcuts store the target executable path as ASCII or UTF-16 strings.
pub fn extract_exes_from_lnk(path: &Path) -> Vec<String> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let mut found = Vec::new();

    // 1. Scan for ASCII ".exe" (case-insensitive)
    let len = bytes.len();
    for i in 0..len.saturating_sub(4) {
        if (bytes[i] == b'.' || bytes[i] == b'.')
            && (bytes[i + 1] == b'e' || bytes[i + 1] == b'E')
            && (bytes[i + 2] == b'x' || bytes[i + 2] == b'X')
            && (bytes[i + 3] == b'e' || bytes[i + 3] == b'E')
        {
            // Walk backwards to find the start of the filename (delimiter: null, slash, backslash, quote)
            let mut start = i;
            while start > 0 {
                let prev = bytes[start - 1];
                if prev == 0 || prev == b'/' || prev == b'\\' || prev == b'"' || prev < 32 || prev > 126 {
                    break;
                }
                start -= 1;
            }
            if start < i {
                if let Ok(s) = std::str::from_utf8(&bytes[start..i + 4]) {
                    let cleaned = s.trim().to_lowercase();
                    if cleaned.ends_with(".exe") && cleaned.len() > 4 && !found.contains(&cleaned) {
                        found.push(cleaned);
                    }
                }
            }
        }
    }

    // 2. Scan for UTF-16-LE ".exe" (e.g. `.\0e\0x\0e\0`)
    for i in 0..len.saturating_sub(8) {
        if bytes[i] == b'.' && bytes[i + 1] == 0
            && (bytes[i + 2] == b'e' || bytes[i + 2] == b'E') && bytes[i + 3] == 0
            && (bytes[i + 4] == b'x' || bytes[i + 4] == b'X') && bytes[i + 5] == 0
            && (bytes[i + 6] == b'e' || bytes[i + 6] == b'E') && bytes[i + 7] == 0
        {
            let mut start = i;
            while start >= 2 {
                let b_low = bytes[start - 2];
                let b_high = bytes[start - 1];
                if b_high != 0 || b_low == 0 || b_low == b'/' || b_low == b'\\' || b_low == b'"' || b_low < 32 || b_low > 126 {
                    break;
                }
                start -= 2;
            }
            if start < i {
                let mut u16_chars = Vec::new();
                for chunk in bytes[start..i + 8].chunks_exact(2) {
                    u16_chars.push(u16::from_le_bytes([chunk[0], chunk[1]]));
                }
                let s = String::from_utf16_lossy(&u16_chars).trim().to_lowercase();
                if s.ends_with(".exe") && s.len() > 4 && !found.contains(&s) {
                    found.push(s);
                }
            }
        }
    }

    found
}

/// Identifies whether the application uses window-centric or instance-centric semantics.
pub fn get_process_kind(exe_name: &str) -> &'static str {
    let lower = exe_name.to_lowercase();
    match lower.as_str() {
        // Browsers
        "chrome.exe" | "brave.exe" | "msedge.exe" | "firefox.exe" | "opera.exe" | "vivaldi.exe" => "window",
        // Editors & IDEs
        "code.exe" | "devenv.exe" | "sublime_text.exe" | "notepad++.exe" | "notepad.exe" => "window",
        // Terminals & System Explorers
        "windowsterminal.exe" | "cmd.exe" | "powershell.exe" | "pwsh.exe" | "explorer.exe" => "window",
        _ => "instance",
    }
}

/// Inspects whether the application associated with `path` is currently running.
pub fn check_app_is_running(path_str: &str) -> Option<RunningAppInfo> {
    let path = Path::new(path_str);
    let running_processes = get_running_process_names();
    if running_processes.is_empty() {
        return None;
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

    let mut candidate_exes = Vec::new();

    if ext == "exe" {
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            candidate_exes.push(file_name.to_lowercase());
        }
    } else if ext == "lnk" {
        // Extract embedded target exes from shortcut bytes
        let extracted = extract_exes_from_lnk(path);
        for exe in extracted {
            if !candidate_exes.contains(&exe) {
                candidate_exes.push(exe);
            }
        }

        // Also synthesize from shortcut stem (e.g. "Brave" -> "brave.exe", "Google Chrome" -> "chrome.exe")
        let stem_lower = stem.to_lowercase();
        let direct_stem_exe = format!("{}.exe", stem_lower.replace(' ', ""));
        if !candidate_exes.contains(&direct_stem_exe) {
            candidate_exes.push(direct_stem_exe);
        }

        // Common known shortcut stems to process names mapping
        if stem_lower.contains("chrome") && !candidate_exes.contains(&"chrome.exe".to_string()) {
            candidate_exes.push("chrome.exe".to_string());
        }
        if stem_lower.contains("brave") && !candidate_exes.contains(&"brave.exe".to_string()) {
            candidate_exes.push("brave.exe".to_string());
        }
        if stem_lower.contains("edge") && !candidate_exes.contains(&"msedge.exe".to_string()) {
            candidate_exes.push("msedge.exe".to_string());
        }
        if stem_lower.contains("visual studio code") || stem_lower == "vscode" {
            if !candidate_exes.contains(&"code.exe".to_string()) {
                candidate_exes.push("code.exe".to_string());
            }
        }
    }

    // Check if any candidate exe is actively running
    for candidate in candidate_exes {
        if running_processes.contains(&candidate) {
            let kind = get_process_kind(&candidate);
            let display_name = if !stem.is_empty() {
                stem
            } else {
                candidate.strip_suffix(".exe").unwrap_or(&candidate).to_string()
            };

            return Some(RunningAppInfo {
                is_running: true,
                app_name: display_name,
                exe_name: candidate,
                kind: kind.to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_processes_contains_current() {
        let procs = get_running_process_names();
        // At least some Windows system processes or test runner must be running
        assert!(!procs.is_empty());
    }

    #[test]
    fn test_process_kind_semantics() {
        assert_eq!(get_process_kind("chrome.exe"), "window");
        assert_eq!(get_process_kind("Code.exe"), "window");
        assert_eq!(get_process_kind("notepad.exe"), "window");
        assert_eq!(get_process_kind("spotify.exe"), "instance");
        assert_eq!(get_process_kind("vlc.exe"), "instance");
    }
}
