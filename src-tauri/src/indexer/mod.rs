pub mod icons;
pub mod scanner;
pub mod cache;
pub mod identity;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::Serialize;
use directories::UserDirs;

pub use icons::{
    ICON_FOLDER,
    get_file_category_and_icon, get_app_icon
};
pub use scanner::{get_base_scan_paths, should_skip_directory, is_ignored_path};
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
    #[serde(skip)]
    pub normalized_name: String,
    #[serde(skip)]
    pub acronym: String,
    #[serde(skip)]
    pub alternate_paths: Vec<String>,
}

impl SearchItem {
    pub fn new(
        name: String,
        path: String,
        icon: Option<String>,
        item_type: ItemType,
        category: String,
    ) -> Self {
        let normalized_name = name.to_lowercase();
        let acronym: String = name
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .filter_map(|w| w.chars().next())
            .collect::<String>()
            .to_lowercase();

        SearchItem {
            name,
            path,
            icon,
            item_type,
            category,
            normalized_name,
            acronym,
            alternate_paths: Vec::new(),
        }
    }

    pub fn synthetic(name: impl Into<String>, path: impl Into<String>, category: impl Into<String>) -> Self {
        let n = name.into();
        Self::new(n, path.into(), None, ItemType::File, category.into())
    }
}

/// Unified classifier for files, apps, and folders across both scanner and watcher
pub fn classify_path(path: &Path, cache: Option<&IconCache>) -> Option<SearchItem> {
    let name = path.file_name().and_then(|s| s.to_str())?;
    // Hidden files and Office lock files (`~$Report.docx`) are never launch targets.
    if name.is_empty() || name.starts_with('.') || name.starts_with("~$") {
        return None;
    }

    if path.is_dir() {
        return Some(SearchItem::new(
            name.to_string(),
            path.to_string_lossy().to_string(),
            Some(ICON_FOLDER.to_string()),
            ItemType::Folder,
            "FOLDER".to_string(),
        ));
    }

    let path_str = path.to_string_lossy().to_string();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // In-progress downloads and temp files churn constantly in Downloads.
    if matches!(ext.as_str(), "crdownload" | "part" | "partial" | "download" | "tmp" | "temp") {
        return None;
    }

    if ext == "lnk" || ext == "exe" {
        let stem = path.file_stem().and_then(|s| s.to_str())?;
        if stem.to_lowercase().contains("uninstall") {
            return None;
        }

        let icon = if let Some(c) = cache {
            if let Some(cached) = c.get(&path_str) {
                Some(cached)
            } else {
                let extracted = get_app_icon(path);
                if let Some(ref icon_b64) = extracted {
                    c.set(&path_str, icon_b64);
                }
                extracted
            }
        } else {
            get_app_icon(path)
        };

        Some(SearchItem::new(
            stem.to_string(),
            path_str,
            icon,
            ItemType::App,
            "APP".to_string(),
        ))
    } else {
        let (cat_str, icon) = get_file_category_and_icon(path);
        Some(SearchItem::new(
            name.to_string(),
            path_str,
            icon,
            ItemType::File,
            cat_str,
        ))
    }
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
        
        let max_depth = if is_start_menu { 5 } else if is_system_root { 2 } else { 4 };

        let walker = WalkDir::new(&path)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !should_skip_directory(name, e.depth())
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.depth() == 0 { continue; }
            let file_path = entry.path();
            if let Some(item) = classify_path(file_path, cache) {
                items.push(item);
            }
        }
    }

    // SCAN USER DIRECTORIES (Desktop, Documents, Downloads, Pictures, Videos, Music)
    for dir in user_scan_dirs() {
        scan_user_dir(&dir, cache, &mut items);
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
        ("Task Manager", sys32.join("Taskmgr.exe")),
        ("Device Manager", sys32.join("devmgmt.msc")),
        ("Registry Editor", PathBuf::from(&system_root).join("regedit.exe")),
    ];

    for (tool_name, tool_path) in common_tools {
        if tool_path.exists() {
            let path_str = tool_path.to_string_lossy().to_string();
            
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

            items.push(SearchItem::new(
                tool_name.to_string(),
                path_str,
                icon,
                ItemType::App,
                "APP".to_string(),
            ));
        }
    }

    // The same file can be reached twice (e.g. `D:\Downloads` via both the drive
    // walk and the user-folder walk). Keep the first occurrence of each path.
    let mut seen = std::collections::HashSet::with_capacity(items.len());
    items.retain(|item| seen.insert(item.path.to_lowercase()));
    deduplicate_apps(items)
}

/// How deep to look inside the user's own folders. Real documents often live
/// many levels down (`Documents\Projects\Client\2026\Q3\deck.pptx`); a limit of
/// 3 missed ~90% of them. Library/hidden trees are pruned, so a full-depth walk
/// stays cheap (~0.5 s for ~50k entries in testing).
pub const USER_DIR_MAX_DEPTH: usize = 12;

/// The user's personal folders, resolved through Windows known-folder APIs so
/// OneDrive-redirected or relocated folders (e.g. `D:\Downloads`) are honoured.
pub fn user_scan_dirs() -> Vec<PathBuf> {
    let Some(user_dirs) = UserDirs::new() else { return Vec::new() };
    [
        user_dirs.desktop_dir(),
        user_dirs.download_dir(),
        user_dirs.document_dir(),
        user_dirs.picture_dir(),
        user_dirs.video_dir(),
        user_dirs.audio_dir(),
    ]
    .into_iter()
    .flatten()
    .map(Path::to_path_buf)
    .filter(|p| p.exists())
    .collect()
}

/// Walk one user folder (pruning ignored/hidden trees) and classify everything in it.
pub fn scan_user_dir(root: &Path, cache: Option<&IconCache>, items: &mut Vec<SearchItem>) {
    let walker = WalkDir::new(root)
        .max_depth(USER_DIR_MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            e.depth() == 0 || !should_skip_directory(name, e.depth())
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.depth() == 0 { continue; }
        if let Some(item) = classify_path(entry.path(), cache) {
            items.push(item);
        }
    }
}

/// Canonical application entity deduplication.
/// Groups shortcuts and executables by AppIdentity, choosing the highest priority representation
/// (Start Menu > System32 > Desktop > raw exe) as primary and preserving all other representations
/// in `alternate_paths`.
pub fn deduplicate_apps(items: Vec<SearchItem>) -> Vec<SearchItem> {
    use std::collections::HashMap;
    use crate::indexer::identity::{resolve_app_identity, representation_priority};

    let mut non_apps = Vec::new();
    let mut app_entities: HashMap<String, SearchItem> = HashMap::new();

    for item in items {
        if item.category == "APP" {
            let identity = resolve_app_identity(Path::new(&item.path));
            let item_prio = representation_priority(&item.path);

            match app_entities.entry(identity.key) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(item);
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    let existing = e.get_mut();
                    let existing_prio = representation_priority(&existing.path);

                    if item_prio > existing_prio {
                        let old_path = existing.path.clone();
                        let mut alt_paths = std::mem::take(&mut existing.alternate_paths);
                        if !alt_paths.contains(&old_path) {
                            alt_paths.push(old_path);
                        }

                        let mut new_primary = item;
                        new_primary.alternate_paths = alt_paths;
                        *existing = new_primary;
                    } else {
                        if !existing.alternate_paths.contains(&item.path) && existing.path != item.path {
                            existing.alternate_paths.push(item.path);
                        }
                    }
                }
            }
        } else {
            non_apps.push(item);
        }
    }

    let mut result: Vec<SearchItem> = app_entities.into_values().chain(non_apps).collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result.dedup_by(|a, b| a.name == b.name && a.path == b.path);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"x").unwrap();
    }

    #[test]
    fn user_dir_scan_finds_deep_documents_and_skips_noise() {
        let root = std::env::temp_dir().join(format!(
            "spotlight_scan_test_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let deep = root.join("Projects").join("Client").join("2026").join("Q3").join("Review").join("Final");
        let wanted = [
            deep.join("Board Deck.ppt"),
            deep.join("notes.txt"),
            root.join("Photos").join("Trip").join("Day 1").join("Beach").join("sunset.jpeg"),
            root.join("Photos").join("iphone").join("IMG_0001.HEIC"),
        ];
        let noise = [
            root.join("tool").join("python").join("Lib").join("site-packages").join("numpy").join("readme.txt"),
            root.join("site").join(".git").join("objects").join("info.txt"),
            root.join("Projects").join("~$Board Deck.pptx"),
            root.join("installer.exe.crdownload"),
        ];
        for p in wanted.iter().chain(noise.iter()) {
            touch(p);
        }

        let mut items = Vec::new();
        scan_user_dir(&root, None, &mut items);
        let paths: Vec<&str> = items.iter().map(|i| i.path.as_str()).collect();

        for p in &wanted {
            assert!(paths.contains(&p.to_str().unwrap()), "deep file should be indexed: {:?}", p);
        }
        for p in &noise {
            assert!(!paths.contains(&p.to_str().unwrap()), "noise should be skipped: {:?}", p);
        }

        let category = |name: &str| items.iter().find(|i| i.name == name).map(|i| i.category.as_str());
        assert_eq!(category("Board Deck.ppt"), Some("PPT"));
        assert_eq!(category("notes.txt"), Some("DOC"));
        assert_eq!(category("sunset.jpeg"), Some("IMG"));
        assert_eq!(category("IMG_0001.HEIC"), Some("IMG"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
