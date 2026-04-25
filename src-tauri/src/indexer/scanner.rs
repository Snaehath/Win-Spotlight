use std::path::{PathBuf};
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

pub fn get_active_drives() -> Vec<String> {
    let mut drives = Vec::new();
    unsafe {
        let mask = GetLogicalDrives();
        for i in 0..26 {
            if (mask & (1 << i)) != 0 {
                let letter = (b'A' + i as u8) as char;
                drives.push(format!("{}:\\", letter));
            }
        }
    }
    drives
}

pub fn get_base_scan_paths() -> Vec<PathBuf> {
    let mut base_paths: Vec<PathBuf> = Vec::new();
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());

    // 1. Standard Locations (Start Menu, Desktop)
    if let Ok(appdata) = std::env::var("APPDATA") {
        base_paths.push(PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    
    base_paths.push(PathBuf::from(format!("{}\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs", system_drive)));
    
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        base_paths.push(PathBuf::from(userprofile).join("Desktop"));
    }
    
    base_paths.push(PathBuf::from(format!("{}\\Users\\Public\\Desktop", system_drive)));
    
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let windows_apps_alias = PathBuf::from(local_appdata).join("Microsoft\\WindowsApps");
        if windows_apps_alias.exists() {
            base_paths.push(windows_apps_alias);
        }
    }

    // 2. Drive Roots
    for drive in get_active_drives() {
        base_paths.push(PathBuf::from(drive));
    }

    base_paths
}

pub fn should_skip_directory(name: &str, depth: usize) -> bool {
    let lower_name = name.to_lowercase();
    
    // Critical system folders to skip during shallow scans
    if depth >= 1 && depth <= 2 && (
        lower_name == "windows" || 
        lower_name == "users" || 
        lower_name == "program files" || 
        lower_name == "program files (x86)" ||
        lower_name.starts_with('$') || 
        lower_name.contains("system volume") ||
        lower_name == "recovery" ||
        lower_name == "config.msi" ||
        lower_name == "perflogs"
    ) {
        return true;
    }
    
    // General speed exclusions
    matches!(name, "node_modules" | ".git" | "target" | "dist" | "__pycache__" | "AppData" | "Common Files" | "bin" | "obj")
}
