//! Application Identity & Entity Resolution
//! Maps raw shortcut (.lnk) files and executable representations into canonical Application Entities.

use std::path::Path;
use crate::process::extract_exes_from_lnk;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppIdentity {
    pub key: String,
}

impl AppIdentity {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

/// Helper executables that are wrapper stubs rather than the core application identity.
const HELPER_EXES: &[&str] = &[
    "update.exe",
    "uninstall.exe",
    "setup.exe",
    "installer.exe",
    "crashpad_handler.exe",
    "elevate.exe",
];

/// Extracts an AUMID (AppUserModelID) string from shortcut binary bytes if present.
/// In Windows LNK property stores, AUMID is stored with key PKEY_AppUserModel_ID
/// ({9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}, 5) or formatted as standard dot-separated / exclamation-marked ID.
pub fn extract_aumid_from_lnk(bytes: &[u8]) -> Option<String> {
    // Scan for UTF-16 strings with typical AUMID patterns (e.g. "com.squirrel.", "Microsoft.", "!App")
    if bytes.len() < 16 {
        return None;
    }

    // Look for common AUMID prefixes in ASCII or UTF-16
    let patterns = [
        "com.squirrel.",
        "Microsoft.Windows.",
        "Microsoft.",
    ];

    let bytes_len = bytes.len();
    for &pattern in &patterns {
        let pattern_bytes = pattern.as_bytes();
        // UTF-16 check
        let mut u16_pattern = Vec::with_capacity(pattern_bytes.len() * 2);
        for &b in pattern_bytes {
            u16_pattern.push(b);
            u16_pattern.push(0);
        }

        if let Some(pos) = bytes.windows(u16_pattern.len()).position(|w| w == u16_pattern) {
            // Read until null terminator
            let mut end = pos;
            while end + 1 < bytes_len {
                if bytes[end] == 0 && bytes[end + 1] == 0 {
                    break;
                }
                end += 2;
            }
            if end > pos {
                let u16_slice: Vec<u16> = bytes[pos..end]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let candidate = String::from_utf16_lossy(&u16_slice).trim().to_lowercase();
                if candidate.len() > pattern.len() && candidate.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '!' || c == '-') {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Resolves the canonical identity of an application shortcut or executable.
pub fn resolve_app_identity(path: &Path) -> AppIdentity {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").trim().to_lowercase();

    if ext == "exe" {
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or(&stem).to_lowercase();
        return AppIdentity::new(format!("exe:{}", file_name));
    }

    if ext == "lnk" {
        // 1. Try reading AUMID from shortcut
        if let Ok(bytes) = std::fs::read(path) {
            if let Some(aumid) = extract_aumid_from_lnk(&bytes) {
                return AppIdentity::new(format!("aumid:{}", aumid));
            }
        }

        // 2. Try extracting candidate target executables from the shortcut
        let target_exes = extract_exes_from_lnk(path);
        
        // Filter out helper/updater executables (like Squirrel's update.exe)
        let main_app_exes: Vec<String> = target_exes
            .into_iter()
            .filter(|e| !HELPER_EXES.contains(&e.as_str()))
            .collect();

        if let Some(first_main_exe) = main_app_exes.first() {
            return AppIdentity::new(format!("exe:{}", first_main_exe));
        }

        // 3. Fallback: normalize the shortcut stem
        // Remove common suffixes like " - Shortcut", "(x64)", "(x86)"
        let cleaned_stem = stem
            .replace(" - shortcut", "")
            .replace(" shortcut", "")
            .replace(" (x64)", "")
            .replace(" (x86)", "")
            .replace(" (64-bit)", "")
            .replace(" (32-bit)", "")
            .trim()
            .to_string();

        return AppIdentity::new(format!("stem:{}", cleaned_stem));
    }

    // Default fallback
    AppIdentity::new(format!("file:{}", stem))
}

/// Returns a numeric priority score for choosing the primary representation of an app.
/// Start Menu > System32 / Windows tools > Desktop > Raw exe.
pub fn representation_priority(path_str: &str) -> u32 {
    let lower = path_str.to_lowercase();
    if lower.contains("start menu") {
        100
    } else if lower.contains("system32") || lower.contains("\\windows\\") {
        80
    } else if lower.contains("desktop") {
        50
    } else if lower.ends_with(".exe") {
        20
    } else {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exe_identity() {
        let p = Path::new(r"C:\Program Files\Google\Chrome\Application\chrome.exe");
        let id = resolve_app_identity(p);
        assert_eq!(id.key, "exe:chrome.exe");
    }

    #[test]
    fn test_representation_priority() {
        assert!(representation_priority(r"C:\Users\Test\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Discord.lnk")
            > representation_priority(r"C:\Users\Test\Desktop\Discord.lnk"));

        assert!(representation_priority(r"C:\Users\Test\Desktop\Discord.lnk")
            > representation_priority(r"C:\Users\Test\AppData\Local\Discord\app-1.0.9\Discord.exe"));
    }

    #[test]
    fn test_stem_cleaning() {
        let p = Path::new(r"C:\Users\Test\Desktop\Discord - Shortcut.lnk");
        let id = resolve_app_identity(p);
        // Either resolved from exe or fallback to cleaned stem "discord"
        assert!(id.key == "stem:discord" || id.key == "exe:discord.exe");
    }

    #[test]
    fn test_app_deduplication_merges_representations() {
        use crate::indexer::{SearchItem, ItemType, deduplicate_apps};

        let start_menu_item = SearchItem::new(
            "Discord".to_string(),
            r"C:\Users\Test\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Discord.lnk".to_string(),
            None,
            ItemType::App,
            "APP".to_string(),
        );

        let desktop_item = SearchItem::new(
            "Discord".to_string(),
            r"C:\Users\Test\Desktop\Discord.lnk".to_string(),
            None,
            ItemType::App,
            "APP".to_string(),
        );

        let items = vec![desktop_item, start_menu_item];
        let deduplicated = deduplicate_apps(items);

        // Exactly one canonical application entity
        assert_eq!(deduplicated.len(), 1);
        let canonical = &deduplicated[0];
        assert_eq!(canonical.name, "Discord");
        // Start Menu representation chosen as primary
        assert!(canonical.path.contains("Start Menu"));
        // Desktop representation preserved in alternate_paths
        assert_eq!(canonical.alternate_paths.len(), 1);
        assert!(canonical.alternate_paths[0].contains("Desktop"));
    }
}
