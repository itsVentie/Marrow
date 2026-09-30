use crate::dto::{DecryptedMessageDto, SearchResultDto};
use crate::network::{map_err_str, parse_peer_pk_array, pubkey_hex_to_peer_id};
use crate::state::AppState;
use r_crypto::handshake::HandshakeInitiator;
use r_network::NetworkCommand;
use r_protocol::{EncryptedMessagePayload, Frame, HandshakeInitPayload};
use r_storage::{MessageDirection, Session, StoredMessage};
use tauri::State;

#[tauri::command]
pub fn create_session(
    peer_pubkey_hex: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
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
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage.list_sessions().map_err(map_err_str)
}

#[tauri::command]
pub fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let storage_guard = state.storage.lock().map_err(map_err_str)?;
    let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;

    storage
        .delete_messages_for_session(&session_id)
        .map_err(map_err_str)?;
    storage.delete_session(&session_id).map_err(map_err_str)
}

#[tauri::command]
pub async fn send_chat_message(
    _session_id: String,
    peer_pubkey_hex: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<DecryptedMessageDto, String> {
    let canonical_session_id = peer_pubkey_hex.clone();
    let plaintext = text.as_bytes();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(map_err_str)?
        .as_secs() as i64;

    let my_pubkey = {
        let identity_guard = state.identity.lock().map_err(map_err_str)?;
        let identity = identity_guard.as_ref().ok_or("Identity not unlocked")?;
        identity.public_hex()
    };

    let peer_pk_array = parse_peer_pk_array(&peer_pubkey_hex)?;

    let is_session_active = {
        let sessions_guard = state.crypto_sessions.lock().map_err(map_err_str)?;
        sessions_guard.contains_key(&canonical_session_id)
    };

    if !is_session_active {
        let signing_key = {
            let identity_guard = state.identity.lock().map_err(map_err_str)?;
            let identity = identity_guard.as_ref().ok_or("Identity not unlocked")?;
            identity.signing_key().clone()
        };

        let mut initiator = HandshakeInitiator::new();

        let init_output = initiator.generate_init_payload(
           &signing_key,
           &peer_pk_array,
        );
        let my_pk_array = parse_peer_pk_array(&my_pubkey)?;
        let init_payload = HandshakeInitPayload::new(my_pk_array, init_output);
        let init_frame = Frame::HandshakeInit(init_payload);

        let encoded_init = init_frame.encode_padded().map_err(map_err_str)?;

        {
            let mut pending_guard = state.pending_handshakes.lock().map_err(map_err_str)?;
            pending_guard.insert(peer_pubkey_hex.clone(), initiator);
        }

        let cmd_tx = {
            let guard = state.network_cmd.lock().map_err(map_err_str)?;
            guard.as_ref().cloned()
        };

        if let Some(tx) = cmd_tx {
            if let Ok(peer_id) = pubkey_hex_to_peer_id(&peer_pubkey_hex) {
                let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
                let _ = tx
                    .send(NetworkCommand::SendFrame {
                        peer_id,
                        data: encoded_init,
                        sender: oneshot_tx,
                    })
                    .await;
                let _ = oneshot_rx.await;
            }
        }

        return Err(
            "PQC Handshake initiated. Please wait for peer response before sending message.".into(),
        );
    }

    let (wire_bytes, sequence_number, plaintext_bytes) = {
        let mut sessions_guard = state.crypto_sessions.lock().map_err(map_err_str)?;
        let session = sessions_guard
            .get_mut(&canonical_session_id)
            .ok_or("Active session not found")?;

        let ad = canonical_session_id.as_bytes();
        let encrypted_msg = session
            .ratchet
            .encrypt(plaintext, ad)
            .map_err(map_err_str)?;

        let serialized_msg = bincode::serialize(&encrypted_msg).map_err(map_err_str)?;

        let msg_payload = EncryptedMessagePayload {
            recipient_pubkey: peer_pk_array,
            dh_pubkey: encrypted_msg.header.dh_pub,
            sequence_number: encrypted_msg.header.n as u64,
            previous_chain_length: encrypted_msg.header.pn as u32,
            nonce: [0u8; 12],
            ciphertext: serialized_msg,
        };

        let frame = Frame::Message(msg_payload);
        let encoded_frame = frame.encode_padded().map_err(map_err_str)?;

        let seq = session.sequence_number;
        session.sequence_number += 1;

        (encoded_frame, seq, plaintext.to_vec())
    };

    let stored_msg = StoredMessage {
        session_id: canonical_session_id.clone(),
        sender_pubkey_hex: my_pubkey.clone(),
        ciphertext: plaintext_bytes.clone(),
        timestamp: now,
        direction: MessageDirection::Outbound,
        sequence_number,
    };

    {
        let storage_guard = state.storage.lock().map_err(map_err_str)?;
        let storage = storage_guard.as_ref().ok_or("Storage not initialized")?;
        storage.store_message(&stored_msg).map_err(map_err_str)?;
        storage
            .update_session_activity(&canonical_session_id, now)
            .map_err(map_err_str)?;
    }

    {
        let search_guard = state.search.lock().map_err(map_err_str)?;
        if let Some(ref search_index) = *search_guard {
            let msg_id = format!("{}/{}", canonical_session_id, sequence_number);
            let _ = search_index.index_message(&msg_id, &canonical_session_id, now as u64, &text);
        }
    }

    let cmd_tx = {
        let guard = state.network_cmd.lock().map_err(map_err_str)?;
        guard.as_ref().cloned()
    };

    if let Some(tx) = cmd_tx {
        if let Ok(peer_id) = pubkey_hex_to_peer_id(&peer_pubkey_hex) {
            let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
            let _ = tx
                .send(NetworkCommand::SendFrame {
                    peer_id,
                    data: wire_bytes,
                    sender: oneshot_tx,
                })
                .await;
            let _ = oneshot_rx.await;
        }
    }

    Ok(DecryptedMessageDto {
        session_id: canonical_session_id,
        sender_pubkey_hex: my_pubkey,
        payload_hex: hex::encode(plaintext),
        timestamp: now,
        direction: MessageDirection::Outbound,
        sequence_number,
    })
}

#[tauri::command]
pub fn get_session_messages(
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

#[tauri::command]
pub fn search_messages(
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultDto>, String> {
    let search_guard = state.search.lock().map_err(map_err_str)?;
    let search_index = search_guard
        .as_ref()
        .ok_or("Search index not initialized")?;

    let max_results = limit.unwrap_or(20);
    let raw_results = search_index
        .search(&query, max_results)
        .map_err(map_err_str)?;

    let dto_results = raw_results
        .into_iter()
        .map(|r| SearchResultDto {
            msg_id: r.msg_id,
            peer_id: r.peer_id,
            timestamp: r.timestamp,
        })
        .collect();

    Ok(dto_results)
}
