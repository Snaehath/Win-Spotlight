use std::os::windows::process::CommandExt;
use tauri::{State, AppHandle};
use crate::history::HistoryManager;
use crate::search::{AppCache, CommandState};
use crate::commands::execute_command_result;

// launcher
#[tauri::command]
pub fn launch_app(
    path: String,
    _app_handle: AppHandle,
    history_manager: State<'_, HistoryManager>,
    _index_state: State<'_, crate::search::IndexState>,
    cache: State<'_, AppCache>,
    cmd_state: State<'_, CommandState>,
) -> Result<bool, String> {
    if let Some(query) = path.strip_prefix("COMMAND:") {
        // Handle raw URLs from shortcuts or intent detection
        if query.starts_with("http") {
            use crate::commands::CommandResult;
            let _ = execute_command_result(CommandResult::Launch("https".to_string(), vec![query.to_string()]))?;
            return Ok(true);
        }

        if let Some(result) = cmd_state.0.handle(query) {
            let _ = execute_command_result(result)?;
            return Ok(true);
        }
    }

    // Persist to adaptive JSON history (for Recents UI + Time Ranking)
    history_manager.record_launch(path.clone());

    // Ensure the path was properly identified in the AppCache or is a valid URL
    let is_url = path.starts_with("http://") || path.starts_with("https://");

    if !is_url {
        let is_valid = {
            let items = cache.apps.lock().unwrap();
            items.iter().any(|item| item.path == path)
        };
        
        if !is_valid {
            return Err("Path validation failed: Path not present in search index. Launch aborted for security.".to_string());
        }

        if !std::path::Path::new(&path).exists() {
            return Err("Path validation failed: File no longer exists on disk.".to_string());
        }
    }

    // Launch the item or URL securely via native ShellExecuteW.
    // Handles URLs in default browser, apps, files, and folders with default associations.
    crate::shell::open_path_or_url(&path).map_err(|e| e.to_string())?;

    Ok(true) // Hide window
}

#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    if !std::path::Path::new(&path).exists() {
        return Err("File does not exist".to_string());
    }

    // explorer.exe /select,"path" highlights the file in its folder
    std::process::Command::new("explorer.exe")
        .args(["/select,", &path])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn check_app_running(path: String) -> Result<Option<crate::process::RunningAppInfo>, String> {
    let lower = path.to_lowercase();
    if !lower.ends_with(".exe") && !lower.ends_with(".lnk") {
        return Ok(None);
    }
    Ok(crate::process::check_app_is_running(&path))
}
