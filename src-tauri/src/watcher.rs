//! File watcher — monitors drive roots and emits incremental index updates.
//! Uses `notify-debouncer-mini` v0.7 which has its own DebouncedEventKind enum.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, DebouncedEvent};
use notify::RecursiveMode;

use crate::index_engine::IndexEngine;
use crate::indexer::{SearchItem, classify_path, is_ignored_path, IconCache};

/// Spawn the file watcher on a background thread.
pub fn start_watcher(
    engine: Arc<IndexEngine>,
    cache: Arc<Mutex<Vec<SearchItem>>>,
    path_lookup: Arc<Mutex<HashMap<String, usize>>>,
    app_indices: Arc<Mutex<Vec<usize>>>,
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

        for events in rx.into_iter().flatten() {
            for evt in events {
                process_event(&evt, &engine, &cache, &path_lookup, &app_indices, &icon_cache);
            }
            // BATCH COMMIT: Save all changes to disk once per batch cycle
            let _ = engine.commit();
        }
    });
}

fn process_event(
    evt: &DebouncedEvent, 
    engine: &Arc<IndexEngine>, 
    cache: &Arc<Mutex<Vec<SearchItem>>>,
    path_lookup: &Arc<Mutex<HashMap<String, usize>>>,
    app_indices: &Arc<Mutex<Vec<usize>>>,
    icon_cache: &IconCache,
) {
    let path = &evt.path;
    if is_ignored_path(path) {
        return;
    }

    let path_str = path.to_string_lossy().to_string();

    if let DebouncedEventKind::Any = evt.kind {
        if path.exists() {
            if let Some(item) = classify_path(path, Some(icon_cache)) {
                let _ = engine.upsert(&item);
                let mut lock = cache.lock().unwrap();
                let mut lookup_lock = path_lookup.lock().unwrap();
                let mut app_indices_lock = app_indices.lock().unwrap();

                // Case-insensitive removal from cache to prevent duplicates
                if let Some(pos) = lock.iter().position(|i| i.path.eq_ignore_ascii_case(&item.path)) {
                    lock.remove(pos);
                }
                lock.push(item);

                // Re-sync path lookup & app indices
                lookup_lock.clear();
                app_indices_lock.clear();
                for (idx, itm) in lock.iter().enumerate() {
                    lookup_lock.insert(itm.path.clone(), idx);
                    if itm.category == "APP" {
                        app_indices_lock.push(idx);
                    }
                }
            }
        } else {
            let _ = engine.remove_by_path(&path_str);
            let mut lock = cache.lock().unwrap();
            let mut lookup_lock = path_lookup.lock().unwrap();
            let mut app_indices_lock = app_indices.lock().unwrap();

            if let Some(pos) = lock.iter().position(|i| i.path.eq_ignore_ascii_case(&path_str)) {
                lock.remove(pos);
                lookup_lock.clear();
                app_indices_lock.clear();
                for (idx, itm) in lock.iter().enumerate() {
                    lookup_lock.insert(itm.path.clone(), idx);
                    if itm.category == "APP" {
                        app_indices_lock.push(idx);
                    }
                }
            }
        }
    }
}
