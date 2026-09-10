use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use std::sync::RwLock;
use tauri::AppHandle;
use tauri::Manager;

pub struct IconCache {
    cache_dir: PathBuf,
    memory: RwLock<HashMap<String, String>>,
}

impl IconCache {
    pub fn new(app: &AppHandle) -> Self {
        let mut cache_dir = app.path().app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        cache_dir.push("icon_cache");
        
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }
        
        Self { 
            cache_dir,
            memory: RwLock::new(HashMap::new()),
        }
    }

    fn get_cache_path(&self, file_path: &str) -> PathBuf {
        // Simple hash-like filename to avoid illegal characters in paths
        let hash = format!("{:x}", md5::compute(file_path));
        self.cache_dir.join(format!("{}.txt", hash))
    }

    pub fn get(&self, file_path: &str) -> Option<String> {
        // Fast path: in-memory cache
        if let Ok(mem) = self.memory.read() {
            if let Some(cached) = mem.get(file_path) {
                return Some(cached.clone());
            }
        }

        // Fallback: check persistent disk cache
        let cache_path = self.get_cache_path(file_path);
        if cache_path.exists() {
            if let Ok(icon) = fs::read_to_string(cache_path) {
                if let Ok(mut mem) = self.memory.write() {
                    mem.insert(file_path.to_string(), icon.clone());
                }
                return Some(icon);
            }
        }
        None
    }

    pub fn set(&self, file_path: &str, base64_icon: &str) {
        if let Ok(mut mem) = self.memory.write() {
            mem.insert(file_path.to_string(), base64_icon.to_string());
        }

        let cache_path = self.get_cache_path(file_path);
        let _ = fs::write(cache_path, base64_icon);
    }
}
