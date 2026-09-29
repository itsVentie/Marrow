use crate::network::map_err_str;
use crate::state::AppState;
use r_storage::StorageEngine;
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn init_storage(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut storage_guard = state.storage.lock().map_err(map_err_str)?;
    if storage_guard.is_some() {
        return Ok(());
    }

    let app_dir = app_handle.path().app_data_dir().map_err(map_err_str)?;
    fs::create_dir_all(&app_dir).map_err(map_err_str)?;
    let db_path: PathBuf = app_dir.join("vault.redb");

    let engine = StorageEngine::open(db_path).map_err(map_err_str)?;
    *storage_guard = Some(engine);

    Ok(())
}