mod state;
mod tray;

use r_crypto::Identity;
use r_network::{NetworkCommand, NetworkEvent, NetworkNode};
use r_storage::{Contact, MessageDirection, Session, StorageEngine, StoredMessage};
use state::AppState;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use tauri::{Emitter, Manager, State};

#[inline]
fn map_err_str<E: Display>(err: E) -> String {
    err.to_string()
}

#[derive(serde::Serialize)]
struct PublicIdentityDto {
    pubkey_hex: String,
}

#[derive(serde::Serialize)]
struct KeyFileInfoDto {
    filename: String,
    path: String,
}

#[derive(serde::Serialize)]
struct DecryptedMessageDto {
    session_id: String,
    sender_pubkey_hex: String,
    payload_hex: String,
    timestamp: i64,
    direction: MessageDirection,
    sequence_number: u64,
}

#[derive(Clone, serde::Serialize)]
struct NetworkEventPayload {
    peer_id: String,
    data_hex: Option<String>,
}

#[tauri::command]
fn init_storage(app_handle: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
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

#[tauri::command]
fn list_identity_files(app_handle: tauri::AppHandle) -> Result<Vec<KeyFileInfoDto>, String> {
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
fn create_identity(
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

    let filename = match alias {
        Some(ref a) if !a.trim().is_empty() => format!("{}.key", a.trim()),
        _ => format!("identity_{}.key", short_pubkey),
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
fn unlock_identity_from_file(
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

    let keypair = libp2p::identity::Keypair::generate_ed25519();
    if let Ok((node, cmd_tx, mut event_rx)) = NetworkNode::new(keypair) {
        tauri::async_runtime::spawn(node.run());

        let handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    NetworkEvent::FrameReceived { peer_id, data } => {
                        let _ = handle_clone.emit(
                            "network://frame_received",
                            NetworkEventPayload {
                                peer_id: peer_id.to_string(),
                                data_hex: Some(hex::encode(data)),
                            },
                        );
                    }
                    NetworkEvent::HolePunchSuccessful { peer_id } => {
                        let _ = handle_clone.emit(
                            "network://hole_punch_success",
                            NetworkEventPayload {
                                peer_id: peer_id.to_string(),
                                data_hex: None,
                            },
                        );
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
fn import_identity_file(
    source_path: String,
    app_handle: tauri::AppHandle,
) -> Result<KeyFileInfoDto, String> {
    let src = PathBuf::from(&source_path);
    if !src.exists() {
        return Err("Source file does not exist".into());
    }

    let filename = src
        .file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();

    let app_dir = app_handle.path().app_data_dir().map_err(map_err_str)?;
    let dest = app_dir.join(&filename);

    fs::copy(&src, &dest).map_err(map_err_str)?;

    Ok(KeyFileInfoDto {
        filename,
        path: dest.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn get_current_identity(state: State<'_, AppState>) -> Result<Option<PublicIdentityDto>, String> {
    let identity_guard = state.identity.lock().map_err(map_err_str)?;
    Ok(identity_guard.as_ref().map(|id| PublicIdentityDto {
        pubkey_hex: id.public_hex(),
    }))
}

#[tauri::command]
fn logout_identity(state: State<'_, AppState>) -> Result<(), String> {
    let mut identity_guard = state.identity.lock().map_err(map_err_str)?;
    *identity_guard = None;
    let mut cmd_guard = state.network_cmd.lock().map_err(map_err_str)?;
    *cmd_guard = None;
    Ok(())
}

#[tauri::command]
fn save_contact(
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
fn list_contacts(state: State<'_, AppState>) -> Result<Vec<Contact>, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.list_contacts().map_err(map_err_str)
}

#[tauri::command]
fn delete_contact(pubkey_hex: String, state: State<'_, AppState>) -> Result<bool, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.delete_contact(&pubkey_hex).map_err(map_err_str)
}

#[tauri::command]
fn create_session(peer_pubkey_hex: String, state: State<'_, AppState>) -> Result<Session, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(map_err_str)?
        .as_secs() as i64;

    storage
        .create_session(&peer_pubkey_hex, now)
        .map_err(map_err_str)
}

#[tauri::command]
fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.list_sessions().map_err(map_err_str)
}

#[tauri::command]
fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage
        .delete_messages_for_session(&session_id)
        .map_err(map_err_str)?;
    storage.delete_session(&session_id).map_err(map_err_str)
}

#[tauri::command]
async fn send_chat_message(
    session_id: String,
    peer_pubkey_hex: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<DecryptedMessageDto, String> {
    let payload_bytes = text.into_bytes();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(map_err_str)?
        .as_secs() as i64;

    let pubkey = {
        let identity_guard = state.identity.lock().map_err(map_err_str)?;
        let identity = identity_guard.as_ref().ok_or("Identity not unlocked")?;
        identity.public_hex()
    };

    let stored_msg = StoredMessage {
        session_id: session_id.clone(),
        sender_pubkey_hex: pubkey.clone(),
        ciphertext: payload_bytes.clone(),
        timestamp: now,
        direction: MessageDirection::Outbound,
        sequence_number: 0,
    };

    {
        let storage_guard = state.storage.lock().map_err(map_err_str)?;
        let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;
        storage.store_message(&stored_msg).map_err(map_err_str)?;
        storage
            .update_session_activity(&session_id, now)
            .map_err(map_err_str)?;
    }

    let cmd_tx = {
        let guard = state.network_cmd.lock().map_err(map_err_str)?;
        guard.as_ref().cloned()
    };

    if let Some(tx) = cmd_tx {
        if let Ok(peer_id) = peer_pubkey_hex.parse::<libp2p::PeerId>() {
            let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
            let _ = tx
                .send(NetworkCommand::SendFrame {
                    peer_id,
                    data: payload_bytes.clone(),
                    sender: oneshot_tx,
                })
                .await;
            let _ = oneshot_rx.await;
        }
    }

    Ok(DecryptedMessageDto {
        session_id,
        sender_pubkey_hex: pubkey,
        payload_hex: hex::encode(payload_bytes),
        timestamp: now,
        direction: MessageDirection::Outbound,
        sequence_number: 0,
    })
}

#[tauri::command]
fn get_session_messages(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DecryptedMessageDto>, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    let messages = storage
        .get_messages_for_session(&session_id)
        .map_err(map_err_str)?;

    let result = messages
        .into_iter()
        .map(|m| DecryptedMessageDto {
            session_id: m.session_id,
            sender_pubkey_hex: m.sender_pubkey_hex,
            payload_hex: hex::encode(m.ciphertext),
            timestamp: m.timestamp,
            direction: m.direction,
            sequence_number: m.sequence_number,
        })
        .collect();

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            tray::create_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            init_storage,
            list_identity_files,
            create_identity,
            unlock_identity_from_file,
            import_identity_file,
            get_current_identity,
            logout_identity,
            save_contact,
            list_contacts,
            delete_contact,
            create_session,
            list_sessions,
            delete_session,
            send_chat_message,
            get_session_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
