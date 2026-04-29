use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use chrono::{Local, Timelike};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchRecord {
    pub path: String,
    pub count: u32,
    #[serde(default)]
    pub last_launched: u64,
    pub hourly_distribution: [u32; 24], // Count of launches per hour (0-23)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct History {
    pub records: Vec<LaunchRecord>,
}

pub struct HistoryManager {
    path: PathBuf,
    cache: std::sync::Mutex<Option<History>>,
}

impl HistoryManager {
    pub fn new(app: &tauri::App) -> Self {
        let mut path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
        let _ = fs::create_dir_all(&path);
        path.push("history_v2.json");
        Self { 
            path,
            cache: std::sync::Mutex::new(None),
        }
    }

    pub fn load(&self) -> History {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(h) = &*cache {
                return h.clone();
            }
        }

        let history = if let Ok(content) = fs::read_to_string(&self.path) {
            serde_json::from_str::<History>(&content).unwrap_or_else(|_| History { records: Vec::new() })
        } else {
            History { records: Vec::new() }
        };

        let mut cache = self.cache.lock().unwrap();
        *cache = Some(history.clone());
        history
    }

    pub fn save(&self, history: &History) {
        let mut cache = self.cache.lock().unwrap();
        *cache = Some(history.clone());
        
        if let Ok(content) = serde_json::to_string_pretty(history) {
            let _ = fs::write(&self.path, content);
        }
    }

    pub fn record_launch(&self, item_path: String) {
        let mut history = self.load();
        let now = Local::now().timestamp() as u64;
        let hour = Local::now().hour() as usize;

        if let Some(record) = history.records.iter_mut().find(|r| r.path == item_path) {
            record.count += 1;
            record.last_launched = now;
            record.hourly_distribution[hour] += 1;
        } else {
            let mut record = LaunchRecord {
                path: item_path,
                count: 1,
                last_launched: now,
                hourly_distribution: [0; 24],
            };
            record.hourly_distribution[hour] = 1;
            history.records.push(record);
        }

        // Sort by recency (newest first)
        history.records.sort_by(|a, b| b.last_launched.cmp(&a.last_launched));

        // Prune old history if it gets too large (keep top 100)
        if history.records.len() > 100 {
            history.records.truncate(100);
        }

        self.save(&history);
    }

    /// Get the "Time Relevance" score for a path based on current hour.
    /// Scale: 0.0 to 1.0
    pub fn get_time_score(&self, item_path: &str) -> f32 {
        let history = self.load();
        let current_hour = Local::now().hour() as usize;

        if let Some(record) = history.records.iter().find(|r| r.path == item_path) {
            let total_launches = record.count as f32;
            if total_launches == 0.0 { return 0.0; }

            // Weight current hour + neighbors (smoothing)
            let prev_hour = if current_hour == 0 { 23 } else { current_hour - 1 };
            let next_hour = (current_hour + 1) % 24;

            let weight = record.hourly_distribution[current_hour] as f32 * 1.0
                       + record.hourly_distribution[prev_hour] as f32 * 0.5
                       + record.hourly_distribution[next_hour] as f32 * 0.5;

            (weight / total_launches).min(1.0)
        } else {
            0.0
        }
    }

    pub fn remove_entry(&self, item_path: &str) {
        let mut history = self.load();
        history.records.retain(|r| r.path != item_path);
        self.save(&history);
    }

    pub fn clear_web_history(&self) {
        let mut history = self.load();
        // Remove all entries that start with COMMAND: and look like URLs
        // We keep other commands like sys or calc if desired, but here we purge web-related ones.
        history.records.retain(|r| {
            !(r.path.starts_with("COMMAND:http") || r.path.starts_with("COMMAND:www."))
        });
        self.save(&history);
    }
}
