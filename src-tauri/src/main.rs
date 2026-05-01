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
mod shell;

use std::sync::{Arc, Mutex};
use indexer::scan_items;
use search::{search_items, AppCache, IndexState, CommandState};
use launcher::{launch_app, reveal_in_explorer};
use history::HistoryManager;
use commands::CommandRegistry;
use index_engine::IndexEngine;
use shortcuts::{ShortcutManager, save_shortcut, clear_shortcuts};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, ShortcutState};
use tauri_plugin_autostart::ManagerExt;
use tauri::{AppHandle, Manager, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
use windows::Win32::Storage::FileSystem::GetLogicalDrives;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
use windows::core::PCWSTR;

fn show_error_and_exit(title: &str, message: &str) -> ! {
    let title_u16: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    let message_u16: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();

    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(message_u16.as_ptr()),
            PCWSTR(title_u16.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
    std::process::exit(1);
}

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

#[tauri::command]
fn remove_from_history(path: String, history_manager: tauri::State<'_, HistoryManager>) {
    history_manager.remove_entry(&path);
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
            if let Err(_e) = app.global_shortcut().register(ctrl_space) {
               show_error_and_exit(
                "Shortcut Conflict",
                "The Ctrl+Space shortcut is already in use. Please close any other instances of Spotlight-Win"
               ) 
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
            let engine_result = IndexEngine::open(&index_dir);
            let engine = match engine_result {
                Ok(e) => Arc::new(e),
                Err(_e) => show_error_and_exit(
                    "Search Engine Error",
                    "Failed to open the search index,  This usually means another instance of the app is already running."
                )
            };

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

            // ── System Tray ──────────────────────────────────────────────────
            let show_i = MenuItem::with_id(app, "show", "Show Spotlight", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let about_1 = MenuItem::with_id(app, "about", "About Spotlight", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &about_1, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            toggle_window(app.clone());
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        "about" => {
                            println!("Spotlight-Win v0.4.0")
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|app, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        toggle_window(app.app_handle().clone());
                    }
                })
                .build(app)?;

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
            remove_from_history,
            reveal_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
