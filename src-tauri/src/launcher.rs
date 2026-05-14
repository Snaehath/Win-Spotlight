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
    index_state: State<'_, crate::search::IndexState>,
    cache: State<'_, AppCache>,
    cmd_state: State<'_, CommandState>,
) -> Result<bool, String> {
    if path.starts_with("COMMAND:") {
        let query = &path[8..];
        
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

    // Update Tantivy launch stats (count + timestamp)
    let items = cache.apps.lock().unwrap();
    index_state.0.record_launch(&path, &items);
    drop(items);

    // Ensure the path was properly identified in the AppCache (prevent arbitrary path execution)
    let is_valid = {
        let items = cache.apps.lock().unwrap();
        items.iter().any(|item| item.path == path)
    };
    
    // Check if it's a URL
    let is_url = path.starts_with("http://") || path.starts_with("https://");

    if !is_url && !std::path::Path::new(&path).exists() && !is_valid {
        return Err("Path validation failed: Not found on disk or cache".to_string());
    }

    if is_url {
        // Use native ShellExecuteW to open URLs securely.
        crate::shell::open_path_or_url(&path).map_err(|e| e.to_string())?;
    } else {
        // Launch the item securely via the native default handler (ShellExecuteW)
        // This is more robust than calling explorer.exe directly and handles 
        // folders, files, and links with their default associations.
        crate::shell::open_path_or_url(&path).map_err(|e| e.to_string())?;
    }

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
