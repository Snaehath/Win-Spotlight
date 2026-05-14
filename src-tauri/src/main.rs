// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shell;

// core modules
mod shortcuts;
mod indexer;
mod launcher;
mod search;
mod history;
mod ranking;
mod commands;
mod index_engine;
mod watcher;
mod currency;

use std::sync::{Arc, Mutex};
use indexer::{scan_items, get_base_scan_paths};
use search::{search_items, AppCache, IndexState, CommandState};
use launcher::{launch_app, reveal_in_explorer};
use history::HistoryManager;
use commands::CommandRegistry;
use index_engine::IndexEngine;
use shortcuts::{ShortcutManager, save_shortcut, clear_shortcuts};

// types
struct ShortcutEnabled(Arc<Mutex<bool>>);

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, ShortcutState};
use tauri_plugin_autostart::ManagerExt;
use tauri::{AppHandle, Manager, Emitter};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
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

// tauri commands
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

/// Detects if a full-screen application (like a game) is in the foreground.
/// We use this to automatically ignore Ctrl+Space during intense gameplay.
fn is_fullscreen_app_active() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetSystemMetrics, 
        SM_CXSCREEN, SM_CYSCREEN, GetWindowTextW
    };
    
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() { return false; }

        // Skip detection for our own window
        let mut title = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut title);
        let title_str = String::from_utf16_lossy(&title[..len as usize]);
        if title_str.contains("Spotlight-Win") { return false; }

        let mut rect = Default::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            
            // If the window covers the full primary screen, it's likely a game.
            width >= screen_w && height >= screen_h
        } else {
            false
        }
    }
}

#[tauri::command]
fn remove_from_history(path: String, history_manager: tauri::State<'_, HistoryManager>) {
    history_manager.remove_entry(&path);
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
                        let items = scan_items(Some(&icon_cache_clone));
                        let _ = engine_clone.bulk_add(&items);
                    }
                    
                    // Run a disk vacuum on every boot to prune old tantivy cache
                    let _ = engine_clone.vacuum();
                });
            }

            // ── In-memory cache for instant first-keystroke response ────────
            let items = scan_items(Some(&icon_cache));
            app.manage(AppCache {
                apps: Mutex::new(items.clone()),
            });

            // ── Global Shortcut Enabled State ──────────────────────────────
            app.manage(ShortcutEnabled(Arc::new(Mutex::new(true))));

            // ── Tantivy state ──────────────────────────────────────────────
            app.manage(IndexState(engine.clone()));

            // ── Command registry ───────────────────────────────────────────
            app.manage(CommandState(CommandRegistry::new()));

            // ── File watcher (background) ──────────────────────────────────
            {
                let cache_arc: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(items));
                
                // Get refined paths (Start Menu, Desktop, etc.) instead of raw drive roots
                let watch_paths: Vec<String> = get_base_scan_paths()
                    .into_iter()
                    .filter(|p: &std::path::PathBuf| {
                        // Don't watch drive roots (e.g. "C:\") recursively!
                        // That is the primary cause of high CPU.
                        p.components().count() > 1 
                    })
                    .map(|p: std::path::PathBuf| p.to_string_lossy().to_string())
                    .collect();

                watcher::start_watcher(
                    engine.clone(),
                    cache_arc,
                    icon_cache.clone(),
                    watch_paths,
                );
            }

            // ── System Tray ──────────────────────────────────────────────────
            let show_i = MenuItem::with_id(app, "show", "Show Spotlight", true, None::<&str>)?;
            let pause_i = MenuItem::with_id(app, "pause", "Pause Global Shortcut", true, None::<&str>)?;
            let about_1 = MenuItem::with_id(app, "about", "About Spotlight", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &pause_i, &about_1, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            toggle_window(app.clone());
                        }
                        "pause" => {
                            let state = app.state::<ShortcutEnabled>();
                            let mut enabled = state.0.lock().unwrap();
                            *enabled = !*enabled;
                            
                            let new_text = if *enabled { "Pause Global Shortcut" } else { "▶ Resume Global Shortcut" };
                            
                            if let Some(menu) = app.menu() {
                                if let Some(item_kind) = menu.get("pause") {
                                    if let Some(menu_item) = item_kind.as_menuitem() {
                                        let _ = menu_item.set_text(new_text);
                                    }
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        "about" => {
                            // Silent about
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
        // shortcuts
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    let state = app.state::<ShortcutEnabled>();
                    let is_manual_enabled = *state.0.lock().unwrap();

                    // SMART DETECTION: Automatically ignore if a full-screen game is in foreground
                    let is_gaming = is_fullscreen_app_active();

                    if is_manual_enabled 
                        && !is_gaming
                        && shortcut.matches(Modifiers::CONTROL, Code::Space)
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
