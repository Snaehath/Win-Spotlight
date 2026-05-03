/// File watcher — monitors drive roots and emits incremental index updates.
/// Uses `notify-debouncer-mini` v0.7 which has its own DebouncedEventKind enum.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, DebouncedEvent};
use notify::RecursiveMode;

use crate::index_engine::IndexEngine;
use crate::indexer::{SearchItem, ItemType, get_file_category_and_icon, get_app_icon, IconCache};

const IGNORED_NAMES: &[&str] = &[
    "node_modules", ".git", "target", "dist", "__pycache__",
    "AppData", "Common Files", "bin", "obj", "Windows", "Recovery",
];

fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        IGNORED_NAMES.iter().any(|&ign| s.eq_ignore_ascii_case(ign))
            || s.starts_with('$')
    })
}

fn classify_path(path: &Path, icon_cache: &IconCache) -> Option<SearchItem> {
    if is_ignored(path) { return None; }

    let name = path.file_name()?.to_str()?;
    if name.starts_with('.') { return None; }

    if path.is_dir() {
        let parent_name = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let display = if !parent_name.is_empty() {
            format!("{} > {}", parent_name, name)
        } else {
            name.to_string()
        };
        return Some(SearchItem {
            name: display,
            path: path.to_string_lossy().to_string(),
            icon: Some(crate::indexer::ICON_FOLDER.to_string()),
            item_type: ItemType::Folder,
            category: "FOLDER".to_string(),
        });
    }

    let path_str = path.to_string_lossy().to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "exe" | "lnk" => {
            let stem = path.file_stem()?.to_str()?;
            if stem.to_lowercase().contains("uninstall") { return None; }
            
            // Use IconCache
            let icon = if let Some(cached) = icon_cache.get(&path_str) {
                Some(cached)
            } else {
                let extracted = get_app_icon(path);
                if let Some(ref icon_b64) = extracted {
                    icon_cache.set(&path_str, icon_b64);
                }
                extracted
            };

            Some(SearchItem {
                name: stem.to_string(),
                path: path_str,
                icon,
                item_type: ItemType::App,
                category: "APP".to_string(),
            })
        }
        _ => {
            let (cat_str, icon) = get_file_category_and_icon(path);
            if cat_str != "FILE" || ext == "txt" || ext == "md" {
                 Some(SearchItem {
                    name: name.to_string(),
                    path: path_str,
                    icon,
                    item_type: ItemType::File,
                    category: cat_str,
                })
            } else {
                None
            }
        }
    }
}

/// Spawn the file watcher on a background thread.
pub fn start_watcher(
    engine: Arc<IndexEngine>,
    cache: Arc<Mutex<Vec<SearchItem>>>,
    icon_cache: Arc<IconCache>,
    watch_paths: Vec<String>,
) {
    thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_secs(3), tx) {
            Ok(d) => d,
            Err(_) => {
                return;
            }
        };

        for path_str in &watch_paths {
            let p = Path::new(path_str);
            if p.exists() {
                let _ = debouncer.watcher().watch(p, RecursiveMode::Recursive);
            }
        }

        for result in rx {
            match result {
                Ok(events) => {
                    for evt in events {
                        process_event(&evt, &engine, &cache, &icon_cache);
                    }
                }
                Err(_) => {}
            };
        }
    });
}

fn process_event(
    evt: &DebouncedEvent, 
    engine: &Arc<IndexEngine>, 
    cache: &Arc<Mutex<Vec<SearchItem>>>,
    icon_cache: &IconCache,
) {
    let path = &evt.path;
    let path_str = path.to_string_lossy().to_string();

    match evt.kind {
        DebouncedEventKind::Any => {
            if path.exists() {
                if let Some(item) = classify_path(path, icon_cache) {
                    let _ = engine.upsert(&item);
                    let _ = engine.commit();
                    let mut lock = cache.lock().unwrap();
                    // Case-insensitive removal from cache to prevent duplicates
                    lock.retain(|i| !i.path.eq_ignore_ascii_case(&item.path));
                    lock.push(item);
                }
            } else {
                let _ = engine.remove_by_path(&path_str);
                let mut lock = cache.lock().unwrap();
                // Case-insensitive removal on deletion
                lock.retain(|i| !i.path.eq_ignore_ascii_case(&path_str));
            }
        }
        _ => {}
    }
}
