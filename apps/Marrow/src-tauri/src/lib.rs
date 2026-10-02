mod commands;
mod dto;
mod network;
mod state;
mod tray;

use r_storage::{SearchIndex, StorageEngine};
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            tray::create_tray(app.handle())?;

            if let Ok(app_dir) = app.path().app_data_dir() {
                let _ = std::fs::create_dir_all(&app_dir);

                let db_path = app_dir.join("marrow.redb");

                let state = app.state::<AppState>();

                if let Ok(storage) = StorageEngine::open(&db_path) {
                    if let Ok(mut storage_guard) = state.storage.lock() {
                        *storage_guard = Some(storage);
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_storage,
            commands::list_identity_files,
            commands::create_identity,
            commands::unlock_identity_from_file,
            commands::import_identity_file,
            commands::get_current_identity,
            commands::logout_identity,
            commands::save_contact,
            commands::list_contacts,
            commands::delete_contact,
            commands::create_session,
            commands::list_sessions,
            commands::delete_session,
            commands::send_chat_message,
            commands::get_session_messages,
            commands::search_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
