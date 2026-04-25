// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod indexer;
mod launcher;
mod search;
mod history;
mod ranking;
mod commands;
mod shortcuts;
mod index_engine;
mod watcher;
mod currency;

use std::sync::{Arc, Mutex};
use indexer::scan_items;
use search::{search_items, AppCache, IndexState, CommandState};
use launcher::launch_app;
use history::HistoryManager;
use commands::CommandRegistry;
use index_engine::IndexEngine;
use shortcuts::{ShortcutManager, save_shortcut, clear_shortcuts};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, ShortcutState};
use tauri_plugin_autostart::ManagerExt;
use tauri::{AppHandle, Manager, Emitter};
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

#[tauri::command]
fn toggle_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide().unwrap();
        } else {
            window.show().unwrap();
            window.set_focus().unwrap();
            let _ = window.emit("window-shown", ());
        }
    }
}

#[tauri::command]
fn hide_window(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().unwrap();
    }
}

fn get_drive_roots() -> Vec<String> {
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // ── Auto-launch ──────────────────────────────────────────────────
            let _ = app.handle().autolaunch().enable();

            // ── Global shortcut: Ctrl + Space ────────────────────────────────
            let ctrl_space = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
            if let Err(e) = app.global_shortcut().register(ctrl_space) {
                eprintln!("Failed to register Ctrl+Space: {}", e);
            }

            // ── History ──────────────────────────────────────────────────────
            let history_manager = HistoryManager::new(app);
            app.manage(history_manager);

            // ── Shortcuts ────────────────────────────────────────────────────
            let shortcut_manager = ShortcutManager::new(app.handle());
            app.manage(shortcut_manager);

            // ── Tantivy Index ──────────────────────────────────────────────
            let mut index_dir = app.path().app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            index_dir.push("spotlight_index");
            let engine = Arc::new(
                IndexEngine::open(&index_dir).expect("Failed to open Tantivy index")
            );

            // ── Icon Cache ──────────────────────────────────────────────────
            let icon_cache = Arc::new(indexer::IconCache::new(app.handle()));

            // ── Initial crawl & Vacuum (background — non-blocking) ─────────────
            {
                let engine_clone = engine.clone();
                let icon_cache_clone = icon_cache.clone();
                let needs_index = !index_dir.join("meta.json").exists();
                std::thread::spawn(move || {
                    if needs_index {
                        eprintln!("[indexer] Bootstrap crawl started...");
                        let items = scan_items(Some(&icon_cache_clone));
                        let _ = engine_clone.bulk_add(&items);
                        eprintln!("[indexer] Bootstrap crawl done ({} items)", items.len());
                    }
                    
                    // Run a disk vacuum on every boot to prune old tantivy cache
                    eprintln!("[indexer] Running segment vacuum...");
                    engine_clone.vacuum();
                    eprintln!("[indexer] Segment vacuum completed.");
                });
            }

            // ── In-memory cache for instant first-keystroke response ────────
            let items = scan_items(Some(&icon_cache));
            app.manage(AppCache {
                apps: Mutex::new(items.clone()),
            });

            // ── Tantivy state ──────────────────────────────────────────────
            app.manage(IndexState(engine.clone()));

            // ── Command registry ───────────────────────────────────────────
            app.manage(CommandState(CommandRegistry::new()));

            // ── File watcher (background) ──────────────────────────────────
            {
                let cache_arc: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(items));
                watcher::start_watcher(
                    engine.clone(),
                    cache_arc,
                    icon_cache.clone(),
                    get_drive_roots(),
                );
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if shortcut.matches(Modifiers::CONTROL, Code::Space)
                        && event.state() == ShortcutState::Pressed
                    {
                        toggle_window(app.clone());
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            search_items,
            launch_app,
            toggle_window,
            hide_window,
            save_shortcut,
            clear_shortcuts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
