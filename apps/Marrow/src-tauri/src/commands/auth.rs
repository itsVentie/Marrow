use crate::dto::{KeyFileInfoDto, PublicIdentityDto};
use crate::network::event_loop::{handle_hole_punch_success, handle_network_frame};
use crate::network::{derive_network_keypair, map_err_str};
use crate::state::AppState;
use r_crypto::Identity;
use r_network::{NetworkEvent, NetworkNode};
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, State};

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[tauri::command]
pub fn list_identity_files(app_handle: tauri::AppHandle) -> Result<Vec<KeyFileInfoDto>, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(map_err_str)?;
    if !app_dir.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let entries = fs::read_dir(app_dir).map_err(map_err_str)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "key" {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        result.push(KeyFileInfoDto {
                            filename: filename.to_string(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn create_identity(
    password: String,
    alias: Option<String>,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<PublicIdentityDto, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    let identity = Identity::generate();
    let vault = identity
        .export_encrypted(password.as_bytes())
        .map_err(map_err_str)?;

    storage.save_vault(&vault).map_err(map_err_str)?;

    let pubkey_hex = identity.public_hex();
    let short_pubkey = &pubkey_hex[..8];

    let clean_alias = alias.as_deref().map(sanitize_filename).unwrap_or_default();
    let filename = if !clean_alias.is_empty() {
        format!("{}.key", clean_alias)
    } else {
        format!("identity_{}.key", short_pubkey)
    };

    let bytes = bincode::serialize(&vault).map_err(map_err_str)?;
    let app_dir = app_handle.path().app_data_dir().map_err(map_err_str)?;
    let file_path = app_dir.join(filename);

    fs::write(&file_path, bytes).map_err(map_err_str)?;

    let mut identity_guard = state.identity.lock().map_err(map_err_str)?;
    *identity_guard = Some(identity);

    Ok(PublicIdentityDto { pubkey_hex })
}

#[tauri::command]
pub fn unlock_identity_from_file(
    file_path: String,
    password: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<PublicIdentityDto, String> {
    let bytes = fs::read(&file_path).map_err(map_err_str)?;
    let vault = bincode::deserialize(&bytes).map_err(map_err_str)?;

    let identity = Identity::import_encrypted(&vault, password.as_bytes()).map_err(map_err_str)?;
    let pubkey_hex = identity.public_hex();

    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    if let Some(storage) = storage_guard.as_ref() {
        let _ = storage.save_vault(&vault);
    }

    let keypair = derive_network_keypair(&identity)?;
    if let Ok((node, cmd_tx, mut event_rx)) = NetworkNode::new(keypair) {
        tauri::async_runtime::spawn(node.run());

        let handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    NetworkEvent::FrameReceived { peer_id, data } => {
                        handle_network_frame(handle_clone.clone(), peer_id.to_string(), data).await;
                    }
                    NetworkEvent::HolePunchSuccessful { peer_id } => {
                        handle_hole_punch_success(handle_clone.clone(), peer_id.to_string());
                    }
                }
            }
        });

        let mut cmd_guard = state.network_cmd.lock().map_err(map_err_str)?;
        *cmd_guard = Some(cmd_tx);
    }

    let mut identity_guard = state.identity.lock().map_err(map_err_str)?;
    *identity_guard = Some(identity);

    Ok(PublicIdentityDto { pubkey_hex })
}

#[tauri::command]
pub fn import_identity_file(
    source_path: String,
    app_handle: tauri::AppHandle,
) -> Result<KeyFileInfoDto, String> {
    let src = PathBuf::from(&source_path);
    if !src.exists() {
        return Err("Source file does not exist".into());
    }

    let raw_filename = src
        .file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy();
    let filename = sanitize_filename(&raw_filename);

    let app_dir = app_handle.path().app_data_dir().map_err(map_err_str)?;
    let dest = app_dir.join(format!("{}.key", filename));

    fs::copy(&src, &dest).map_err(map_err_str)?;

    Ok(KeyFileInfoDto {
        filename,
        path: dest.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn get_current_identity(state: State<'_, AppState>) -> Result<Option<PublicIdentityDto>, String> {
    let identity_guard = state.identity.lock().map_err(map_err_str)?;
    Ok(identity_guard.as_ref().map(|id| PublicIdentityDto {
        pubkey_hex: id.public_hex(),
    }))
}

#[tauri::command]
pub fn logout_identity(state: State<'_, AppState>) -> Result<(), String> {
    let mut identity_guard = state.identity.lock().map_err(map_err_str)?;
    *identity_guard = None;
    let mut cmd_guard = state.network_cmd.lock().map_err(map_err_str)?;
    *cmd_guard = None;
    Ok(())
}