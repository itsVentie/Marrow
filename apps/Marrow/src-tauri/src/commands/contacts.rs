use crate::network::map_err_str;
use crate::state::AppState;
use r_storage::Contact;
use tauri::State;

#[tauri::command]
pub fn save_contact(
    pubkey_hex: String,
    alias: String,
    state: State<'_, AppState>,
) -> Result<Contact, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(map_err_str)?
        .as_secs() as i64;

    let contact = Contact {
        pubkey_hex,
        alias,
        added_at: now,
    };

    storage.save_contact(&contact).map_err(map_err_str)?;
    Ok(contact)
}

#[tauri::command]
pub fn list_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.list_contacts().map_err(map_err_str)
}

#[tauri::command]
pub fn delete_contact(pubkey_hex: String, state: State<'_, AppState>) -> Result<bool, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.delete_contact(&pubkey_hex).map_err(map_err_str)
}