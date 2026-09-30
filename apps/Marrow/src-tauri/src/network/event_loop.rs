use r_crypto::handshake::HandshakeResponder;
use r_crypto::x25519_dalek;
use r_network::NetworkCommand;
use r_protocol::{Frame, HandshakeResponsePayload};
use r_storage::{MessageDirection, StoredMessage};
use tauri::{Emitter, Manager};

use crate::dto::{DecryptedMessageDto, NetworkEventPayload};
use crate::network::{parse_peer_pk_array, pubkey_hex_to_peer_id};
use crate::state::{AppState, CryptoSession};

pub async fn handle_network_frame(handle: tauri::AppHandle, peer_pk_hex: String, data: Vec<u8>) {
    let frame = match Frame::decode(&data) {
        Ok(f) => f,
        Err(_) => return,
    };

    let state = handle.state::<AppState>();

    match frame {
        Frame::HandshakeInit(payload) => {
            let sender_pubkey_hex = hex::encode(payload.sender_pubkey);
            if let Ok(resp_out) = HandshakeResponder::process_init_and_respond(
                &payload.ephemeral_x25519,
                &payload.ml_kem_pk,
            ) {
                let my_pubkey = {
                    let id_guard = state.identity.lock().unwrap();
                    id_guard.as_ref().map(|i| i.public_hex()).unwrap_or_default()
                };

                if let Ok(my_pk_array) = parse_peer_pk_array(&my_pubkey) {
                    let responder_dhs =
                        x25519_dalek::StaticSecret::from(resp_out.x25519_secret);

                    {
                        let mut sessions_guard = state.crypto_sessions.lock().unwrap();
                        sessions_guard.insert(
                            sender_pubkey_hex.clone(),
                            CryptoSession {
                                ratchet: r_crypto::DoubleRatchet::init_responder(
                                    resp_out.master_secret.0,
                                    responder_dhs,
                                ),
                                peer_pubkey_hex: sender_pubkey_hex.clone(),
                                sequence_number: 0,
                            },
                        );
                    }

                    let resp_payload = HandshakeResponsePayload::new(my_pk_array, &resp_out);
                    let response_frame = Frame::HandshakeResponse(resp_payload);

                    if let Ok(encoded_resp) = response_frame.encode_padded() {
                        let cmd_tx = {
                            let guard = state.network_cmd.lock().unwrap();
                            guard.as_ref().cloned()
                        };

                        if let Some(tx) = cmd_tx {
                            if let Ok(target_peer_id) = pubkey_hex_to_peer_id(&sender_pubkey_hex) {
                                let (oneshot_tx, _) = tokio::sync::oneshot::channel();
                                let _ = tx
                                    .send(NetworkCommand::SendFrame {
                                        peer_id: target_peer_id,
                                        data: encoded_resp,
                                        sender: oneshot_tx,
                                    })
                                    .await;
                            }
                        }
                    }
                }
            }
        }
        Frame::HandshakeResponse(payload) => {
            let mut pending_guard = state.pending_handshakes.lock().unwrap();
            if let Some(initiator) = pending_guard.remove(&peer_pk_hex) {
                if let Ok(master_secret) =
                    initiator.process_response(&payload.ephemeral_x25519, &payload.ml_kem_ct)
                {
                    let peer_x25519_pk =
                        x25519_dalek::PublicKey::from(payload.ephemeral_x25519);
                    let mut sessions_guard = state.crypto_sessions.lock().unwrap();

                    sessions_guard.insert(
                        peer_pk_hex.clone(),
                        CryptoSession {
                            ratchet: r_crypto::DoubleRatchet::init_initiator(
                                master_secret.0,
                                peer_x25519_pk,
                            ),
                            peer_pubkey_hex: peer_pk_hex.clone(),
                            sequence_number: 0,
                        },
                    );
                }
            }
        }
        Frame::Message(payload) => {
            let mut sessions_guard = state.crypto_sessions.lock().unwrap();

            if let Some(session) = sessions_guard.get_mut(&peer_pk_hex) {
                let ad = peer_pk_hex.as_bytes();

                if let Ok(encrypted_msg) =
                    bincode::deserialize::<r_crypto::ratchet::EncryptedMessage>(&payload.ciphertext)
                {
                    if let Ok(plaintext_bytes) = session.ratchet.decrypt(&encrypted_msg, ad) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;

                        let stored_msg = StoredMessage {
                            session_id: peer_pk_hex.clone(),
                            sender_pubkey_hex: peer_pk_hex.clone(),
                            ciphertext: plaintext_bytes.clone(),
                            timestamp: now,
                            direction: MessageDirection::Inbound,
                            sequence_number: encrypted_msg.header.n as u64,
                        };

                        let storage_guard = state.storage.lock().unwrap();
                        if let Some(ref storage) = *storage_guard {
                            let _ = storage.store_message(&stored_msg);
                            let _ = storage.update_session_activity(&peer_pk_hex, now);
                        }

                        let _ = handle.emit(
                            "chat://message_received",
                            DecryptedMessageDto {
                                session_id: peer_pk_hex.clone(),
                                sender_pubkey_hex: peer_pk_hex,
                                payload_hex: hex::encode(plaintext_bytes),
                                timestamp: now,
                                direction: MessageDirection::Inbound,
                                sequence_number: encrypted_msg.header.n as u64,
                            },
                        );
                    }
                }
            }
        }
        _ => {}
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