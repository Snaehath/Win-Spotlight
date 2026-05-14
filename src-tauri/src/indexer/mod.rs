pub mod icons;
pub mod scanner;
pub mod cache;

use std::path::{PathBuf};
use walkdir::WalkDir;
use serde::Serialize;
use directories::UserDirs;

pub use icons::{ICON_FOLDER, get_file_category_and_icon, get_app_icon};
pub use scanner::{get_base_scan_paths, should_skip_directory};
pub use cache::IconCache;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ItemType {
    App,
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchItem {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub item_type: ItemType,
    pub category: String,
}

// scanner
pub fn scan_items(cache: Option<&IconCache>) -> Vec<SearchItem> {
    let mut items = Vec::new();
    let base_paths = get_base_scan_paths();
    let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());

    for path in base_paths {
        if !path.exists() { continue; }
        
        let path_str = path.to_string_lossy();
        let is_start_menu = path_str.contains("Start Menu");
        let is_system_root = path_str == format!("{}\\", system_drive) || path_str == system_drive;
        
        let max_depth = if is_start_menu { 5 } else if is_system_root { 2 } else { 5 };

        let walker = WalkDir::new(&path)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !should_skip_directory(name, e.depth())
            });

        for entry in walker.filter_map(|e| e.ok()) {
            let file_path = entry.path();
            let name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.is_empty() || name.starts_with('.') { continue; }

            if file_path.is_file() {
                let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                
                if ext == "lnk" || ext == "exe" {
                    if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                        if stem.to_lowercase().contains("uninstall") { continue; }
                        
                        let path_str = file_path.to_string_lossy().to_string();
                        
                        // Use IconCache
                        let icon = if let Some(c) = cache {
                            if let Some(cached) = c.get(&path_str) {
                                Some(cached)
                            } else {
                                let extracted = get_app_icon(file_path);
                                if let Some(ref icon_b64) = extracted {
                                    c.set(&path_str, icon_b64);
                                }
                                extracted
                            }
                        } else {
                            get_app_icon(file_path)
                        };

                        items.push(SearchItem {
                            name: stem.to_string(),
                            path: path_str,
                            icon,
                            item_type: ItemType::App,
                            category: "APP".to_string(),
                        });
                    }
                } else {
                    let (cat_str, icon) = get_file_category_and_icon(file_path);
                    if cat_str != "FILE" || ext == "txt" || ext == "md" {
                        items.push(SearchItem {
                            name: name.to_string(),
                            path: file_path.to_string_lossy().to_string(),
                            icon,
                            item_type: ItemType::File,
                            category: cat_str,
                        });
                    }
                }
            } else if file_path.is_dir() && entry.depth() > 0 {
                let parent_name = file_path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("");
                let display_name = if !parent_name.is_empty() && entry.depth() > 3 {
                    format!("{} > {}", parent_name, name)
                } else {
                    name.to_string()
                };

                items.push(SearchItem {
                    name: display_name,
                    path: file_path.to_string_lossy().to_string(),
                    icon: Some(ICON_FOLDER.to_string()),
                    item_type: ItemType::Folder,
                    category: "FOLDER".to_string(),
                });
            }
        }
    }

    // SCAN USER FOLDERS
    if let Some(user_dirs) = UserDirs::new() {
        let folders = vec![
            (user_dirs.download_dir(), "Downloads"),
            (user_dirs.document_dir(), "Documents"),
            (user_dirs.picture_dir(), "Pictures"),
        ];

        for (dir_opt, cat) in folders {
            if let Some(path) = dir_opt {
                if !path.exists() { continue; }
                let walker = WalkDir::new(path).max_depth(2).into_iter().filter_entry(|e| {
                    let name = e.file_name().to_str().unwrap_or("");
                    !should_skip_directory(name, e.depth())
                });

                for entry in walker.filter_map(|e| e.ok()) {
                    let file_path = entry.path();
                    let name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if name.is_empty() || name.starts_with('.') { continue; }

                    if file_path.is_dir() {
                        items.push(SearchItem {
                            name: name.to_string(),
                            path: file_path.to_string_lossy().to_string(),
                            icon: Some(ICON_FOLDER.to_string()),
                            item_type: ItemType::Folder,
                            category: cat.to_uppercase(),
                        });
                    } else {
                        let (cat_str, icon) = get_file_category_and_icon(file_path);
                        items.push(SearchItem {
                            name: name.to_string(),
                            path: file_path.to_string_lossy().to_string(),
                            icon,
                            item_type: ItemType::File,
                            category: cat_str,
                        });
                    }
                }
            }
        }
    }
    
    // COMMON SYSTEM TOOLS
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let sys32 = PathBuf::from(&system_root).join("System32");
    
    let common_tools = vec![
        ("Calculator", sys32.join("calc.exe")),
        ("Command Prompt", sys32.join("cmd.exe")),
        ("Notepad", sys32.join("notepad.exe")),
        ("Paint", sys32.join("mspaint.exe")),
        ("PowerShell", sys32.join("WindowsPowerShell\\v1.0\\powershell.exe")),
    ];

    for (tool_name, tool_path) in common_tools {
        if tool_path.exists() {
            let path_str = tool_path.to_string_lossy().to_string();
            
            // Use IconCache
            let icon = if let Some(c) = cache {
                if let Some(cached) = c.get(&path_str) {
                    Some(cached)
                } else {
                    let extracted = get_app_icon(&tool_path);
                    if let Some(ref icon_b64) = extracted {
                        c.set(&path_str, icon_b64);
                    }
                    extracted
                }
            } else {
                get_app_icon(&tool_path)
            };

            items.push(SearchItem {
                name: tool_name.to_string(),
                path: path_str,
                icon,
                item_type: ItemType::App,
                category: "APP".to_string(),
            });
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    items.dedup_by(|a, b| a.name == b.name && a.path == b.path);
    items
}
