use crate::network::{map_err_str, pubkey_hex_to_peer_id};
use crate::state::AppState;

use r_storage::Contact;

use tauri::State;

fn normalize_multiaddr(pubkey_hex: &str, raw_multiaddr: &str) -> Result<String, String> {
    let expected_peer_id = pubkey_hex_to_peer_id(pubkey_hex)?;

    let multiaddr = raw_multiaddr
        .parse::<libp2p::Multiaddr>()
        .map_err(map_err_str)?;

    let embedded_peer_id = multiaddr
        .iter()
        .filter_map(|protocol| match protocol {
            libp2p::multiaddr::Protocol::P2p(peer_id) => Some(peer_id),
            _ => None,
        })
        .last();

    let Some(embedded_peer_id) = embedded_peer_id else {
        return Err("Multiaddr must contain /p2p/<peer_id>".into());
    };

    if embedded_peer_id != expected_peer_id {
        return Err("Multiaddr peer ID does not match contact public key".into());
    }

    Ok(multiaddr.to_string())
}

#[tauri::command]
pub fn save_contact(
    pubkey_hex: String,
    alias: String,
    multiaddr: Option<String>,
    state: State<'_, AppState>,
) -> Result<Contact, String> {
    let normalized_multiaddr = match multiaddr {
        Some(value) if !value.trim().is_empty() => {
            Some(normalize_multiaddr(&pubkey_hex, value.trim())?)
        }
        _ => None,
    };

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

    storage
        .save_contact_with_address(&contact, normalized_multiaddr.as_deref())
        .map_err(map_err_str)?;

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
