use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize, Deserialize, Default)]
pub struct ShortcutData {
    pub shortcuts: HashMap<String, String>, // alias -> url
}

pub struct ShortcutManager {
    path: PathBuf,
    data: std::sync::Mutex<ShortcutData>,
}

impl ShortcutManager {
    pub fn new(app: &AppHandle) -> Self {
        let mut path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        let _ = fs::create_dir_all(&path);
        path.push("shortcuts.json");

        let data = if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            ShortcutData::default()
        };

        Self {
            path,
            data: std::sync::Mutex::new(data),
        }
    }

    pub fn add(&self, alias: String, url: String) {
        let mut data = self.data.lock().unwrap();
        data.shortcuts.insert(alias.to_lowercase(), url);
        if let Ok(content) = serde_json::to_string_pretty(&*data) {
            let _ = fs::write(&self.path, content);
        }
    }

    pub fn get_all(&self) -> HashMap<String, String> {
        self.data.lock().unwrap().shortcuts.clone()
    }

    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.shortcuts.clear();
        if let Ok(content) = serde_json::to_string_pretty(&*data) {
            let _ = fs::write(&self.path, content);
        }
    }
}

#[tauri::command]
pub fn save_shortcut(alias: String, url: String, manager: tauri::State<'_, ShortcutManager>) {
    manager.add(alias, url);
}

#[tauri::command]
pub fn clear_shortcuts(
    manager: tauri::State<'_, ShortcutManager>,
    history_manager: tauri::State<'_, crate::history::HistoryManager>
) {
    manager.clear();
    history_manager.clear_web_history();
}
