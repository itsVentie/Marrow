use r_crypto::handshake::HandshakeResponder;
use r_network::NetworkCommand;
use r_protocol::{Frame, HandshakeResponsePayload};
use r_storage::{MessageDirection, StoredMessage};
use tauri::{Emitter, Manager};

use crate::dto::{DecryptedMessageDto, NetworkEventPayload};
use crate::network::{parse_peer_pk_array, pubkey_hex_to_peer_id};
use crate::state::{AppState, CryptoSession};

pub async fn handle_network_frame(handle: tauri::AppHandle, peer_id: String, data: Vec<u8>) {
    let frame = match Frame::decode(&data) {
        Ok(frame) => frame,
        Err(_) => return,
    };

    let state = handle.state::<AppState>();

    match frame {
        Frame::HandshakeInit(payload) => {
            let transport_peer_id = match peer_id.parse::<libp2p::PeerId>() {
                Ok(peer_id) => peer_id,
                Err(_) => return,
            };

            let sender_pubkey_hex = hex::encode(payload.sender_pubkey);

            let expected_peer_id = match pubkey_hex_to_peer_id(&sender_pubkey_hex) {
                Ok(peer_id) => peer_id,
                Err(_) => return,
            };

            if transport_peer_id != expected_peer_id {
                return;
            }

            let responder_signing_key = {
                let identity_guard = match state.identity.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                match identity_guard.as_ref() {
                    Some(identity) => identity.signing_key(),
                    None => return,
                }
            };

            let my_pubkey = {
                let identity_guard = match state.identity.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                match identity_guard.as_ref() {
                    Some(identity) => identity.public_hex(),
                    None => return,
                }
            };

            let responder_pubkey = match parse_peer_pk_array(&my_pubkey) {
                Ok(pubkey) => pubkey,
                Err(_) => return,
            };

            let resp_out = match HandshakeResponder::process_init_and_respond(
                &responder_signing_key,
                &payload.sender_pubkey,
                &payload.recipient_pubkey,
                &payload.ephemeral_x25519,
                &payload.ml_kem_pk,
                &payload.signature,
            ) {
                Ok(output) => output,
                Err(_) => return,
            };

            let response_payload =
                HandshakeResponsePayload::new(responder_pubkey, payload.sender_pubkey, &resp_out);

            let response_frame = Frame::HandshakeResponse(response_payload);

            let encoded_response = match response_frame.encode_padded() {
                Ok(bytes) => bytes,
                Err(_) => return,
            };

            let cmd_tx = {
                let guard = match state.network_cmd.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                guard.as_ref().cloned()
            };

            let Some(tx) = cmd_tx else {
                return;
            };

            let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();

            if tx
                .send(NetworkCommand::SendFrame {
                    peer_id: transport_peer_id,
                    data: encoded_response,
                    sender: oneshot_tx,
                })
                .await
                .is_err()
            {
                return;
            }

            // Wait for the request/response layer to confirm that the
            // handshake response was successfully delivered.
            //
            // The response payload itself is not currently used here;
            // receiving Ok(_) is sufficient to establish the local session.
            match oneshot_rx.await {
                Ok(Ok(_)) => {}
                Ok(Err(_)) | Err(_) => return,
            }

            {
                let mut sessions_guard = match state.crypto_sessions.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                sessions_guard.insert(
                    sender_pubkey_hex.clone(),
                    CryptoSession {
                        ratchet: r_crypto::DoubleRatchet::init_responder(
                            resp_out.master_secret.0,
                            resp_out.x25519_secret,
                        ),
                        peer_pubkey_hex: sender_pubkey_hex.clone(),
                        sequence_number: 0,
                    },
                );
            }

            {
                let mut mapping_guard = match state.peer_id_to_pubkey.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                mapping_guard.insert(transport_peer_id, sender_pubkey_hex.clone());
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            {
                let storage_guard = match state.storage.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                if let Some(storage) = storage_guard.as_ref() {
                    let _ = storage.create_session(&sender_pubkey_hex, now);
                }
            }
        }

        Frame::HandshakeResponse(payload) => {
            let transport_peer_id = match peer_id.parse::<libp2p::PeerId>() {
                Ok(peer_id) => peer_id,
                Err(_) => return,
            };

            let responder_pubkey_hex = hex::encode(payload.sender_pubkey);

            let expected_peer_id = match pubkey_hex_to_peer_id(&responder_pubkey_hex) {
                Ok(peer_id) => peer_id,
                Err(_) => return,
            };

            if transport_peer_id != expected_peer_id {
                return;
            }

            let initiator = {
                let mut pending_guard = match state.pending_handshakes.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                match pending_guard.remove(&responder_pubkey_hex) {
                    Some(initiator) => initiator,
                    None => return,
                }
            };

            let master_secret = match initiator.process_response(
                &payload.sender_pubkey,
                &payload.recipient_pubkey,
                &payload.ephemeral_x25519,
                &payload.ml_kem_ct,
                &payload.signature,
            ) {
                Ok(secret) => secret,
                Err(_) => return,
            };

            let peer_x25519_pk = r_crypto::x25519_dalek::PublicKey::from(payload.ephemeral_x25519);

            {
                let mut sessions_guard = match state.crypto_sessions.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                sessions_guard.insert(
                    responder_pubkey_hex.clone(),
                    CryptoSession {
                        ratchet: r_crypto::DoubleRatchet::init_initiator(
                            master_secret.0,
                            peer_x25519_pk,
                        ),
                        peer_pubkey_hex: responder_pubkey_hex.clone(),
                        sequence_number: 0,
                    },
                );
            }

            {
                let mut mapping_guard = match state.peer_id_to_pubkey.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                mapping_guard.insert(transport_peer_id, responder_pubkey_hex.clone());
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            {
                let storage_guard = match state.storage.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                if let Some(storage) = storage_guard.as_ref() {
                    let _ = storage.create_session(&responder_pubkey_hex, now);
                }
            }
        }

        Frame::Message(payload) => {
            let transport_peer_id = match peer_id.parse::<libp2p::PeerId>() {
                Ok(peer_id) => peer_id,
                Err(_) => return,
            };

            let session_pubkey_hex = {
                let mapping_guard = match state.peer_id_to_pubkey.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                match mapping_guard.get(&transport_peer_id) {
                    Some(pubkey_hex) => pubkey_hex.clone(),
                    None => return,
                }
            };

            let mut sessions_guard = match state.crypto_sessions.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };

            let Some(session) = sessions_guard.get_mut(&session_pubkey_hex) else {
                return;
            };

            let ad = session_pubkey_hex.as_bytes();

            let encrypted_msg = match bincode::deserialize::<r_crypto::ratchet::EncryptedMessage>(
                &payload.ciphertext,
            ) {
                Ok(message) => message,
                Err(_) => return,
            };

            if payload.dh_pubkey != encrypted_msg.header.dh_pub
                || payload.sequence_number != encrypted_msg.header.n as u64
                || payload.previous_chain_length != encrypted_msg.header.pn
            {
                return;
            }

            let plaintext_bytes = match session.ratchet.decrypt(&encrypted_msg, ad) {
                Ok(plaintext) => plaintext,
                Err(_) => return,
            };

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let sequence_number = encrypted_msg.header.n as u64;
            let msg_id = format!("{}/{}", peer_id, sequence_number);

            let stored_msg = StoredMessage {
                session_id: session_pubkey_hex.clone(),
                sender_pubkey_hex: session.peer_pubkey_hex.clone(),
                ciphertext: plaintext_bytes.clone(),
                timestamp: now,
                direction: MessageDirection::Inbound,
                sequence_number,
            };

            {
                let storage_guard = match state.storage.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                if let Some(storage) = storage_guard.as_ref() {
                    let _ = storage.store_message(&stored_msg);
                    let _ = storage.update_session_activity(&session_pubkey_hex, now);
                }
            }

            {
                let search_guard = match state.search.lock() {
                    Ok(guard) => guard,
                    Err(_) => return,
                };

                if let Some(search_index) = search_guard.as_ref() {
                    if let Ok(text_content) = String::from_utf8(plaintext_bytes.clone()) {
                        let _ = search_index.index_message(
                            &msg_id,
                            &session_pubkey_hex,
                            now as u64,
                            &text_content,
                        );
                    }
                }
            }

            let _ = handle.emit(
                "chat://message_received",
                DecryptedMessageDto {
                    session_id: session_pubkey_hex,
                    sender_pubkey_hex: session.peer_pubkey_hex.clone(),
                    payload_hex: hex::encode(plaintext_bytes),
                    timestamp: now,
                    direction: MessageDirection::Inbound,
                    sequence_number,
                },
            );
        }

        Frame::Ack { .. } | Frame::Ping | Frame::Pong | Frame::Dummy(_) => {}
    }
}

pub fn handle_hole_punch_success(handle: tauri::AppHandle, peer_id: String) {
    let _ = handle.emit(
        "network://hole_punch_success",
        NetworkEventPayload {
            peer_id,
            data_hex: None,
        },
    );
}

pub fn handle_network_listening(handle: tauri::AppHandle, address: String) {
    let _ = handle.emit(
        "network://listening",
        NetworkEventPayload {
            peer_id: address,
            data_hex: None,
        },
    );
}

pub fn handle_connection_established(handle: tauri::AppHandle, peer_id: String) {
    let _ = handle.emit(
        "network://connection_established",
        NetworkEventPayload {
            peer_id,
            data_hex: None,
        },
    );
}

pub fn handle_connection_closed(handle: tauri::AppHandle, peer_id: String) {
    let _ = handle.emit(
        "network://connection_closed",
        NetworkEventPayload {
            peer_id,
            data_hex: None,
        },
    );
}
