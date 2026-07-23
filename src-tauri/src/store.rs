use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
pub use shared_types::image_compressor::ImageCompressorState;
use tauri::Manager;

// ── Backend-specific types ───────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct CompressHistoryItem {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub state: ImageCompressorState,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AppStore {
    #[serde(default)]
    pub compressor: ImageCompressorState,
    #[serde(default)]
    pub compressor_history: Vec<CompressHistoryItem>,
}

impl AppStore {
    fn storage_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let dir = app.path().app_data_dir().map_err(|e| format!("app_data_dir: {e}"))?;
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
        }
        Ok(dir.join("app_store.json"))
    }

    pub fn load(app: &tauri::AppHandle) -> Self {
        let Ok(path) = Self::storage_path(app) else { return Self::default() };
        if let Ok(json) = fs::read_to_string(&path)
            && let Ok(store) = serde_json::from_str(&json)
        {
            return store;
        }
        Self::default()
    }

    pub fn save(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::storage_path(app)?;
        let mut cloned = self.clone();
        if cloned.compressor_history.len() > 50 {
            cloned.compressor_history.truncate(50);
        }
        let json = serde_json::to_string_pretty(&cloned).map_err(|e| format!("serialize: {e}"))?;
        fs::write(path, json).map_err(|e| format!("write: {e}"))?;
        Ok(())
    }
}

#[tauri::command]
pub fn open_dir(path: String) -> Result<(), String> {
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
pub fn load_store(app: tauri::AppHandle) -> AppStore {
    AppStore::load(&app)
}

#[tauri::command]
pub fn save_store(app: tauri::AppHandle, store: AppStore) -> Result<(), String> {
    store.save(&app)
}

#[tauri::command]
pub fn add_history(app: tauri::AppHandle, item: CompressHistoryItem) -> Result<(), String> {
    let mut store = AppStore::load(&app);
    store.compressor_history.insert(0, item);
    store.save(&app)
}
