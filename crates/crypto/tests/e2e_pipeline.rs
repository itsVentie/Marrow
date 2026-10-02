use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};
use r_crypto::ratchet::DoubleRatchet;
use r_crypto::x25519_dalek::PublicKey as X25519PublicKey;
use r_crypto::Identity;
use r_protocol::{EncryptedMessagePayload, Frame, HandshakeInitPayload, HandshakeResponsePayload};

#[test]
fn test_e2e_handshake_frame_and_ratchet_pipeline() {
    let ventie_identity = Identity::generate();
    let anek_identity = Identity::generate();

    let ventie_pubkey = *ventie_identity.verifying_key().as_bytes();
    let anek_pubkey = *anek_identity.verifying_key().as_bytes();

    let mut initiator = HandshakeInitiator::new();

    let ventie_sk = ventie_identity.signing_key();

    let init_out = initiator.generate_init_payload(&ventie_sk, &anek_pubkey);

    let init_payload = HandshakeInitPayload::new(ventie_pubkey, init_out);

    let init_frame = Frame::HandshakeInit(init_payload);

    let encoded_init = init_frame
        .encode()
        .expect("Failed to encode HandshakeInit frame");

    let decoded_init_frame =
        Frame::decode(&encoded_init).expect("Failed to decode HandshakeInit frame");

    let (resp_payload, responder_secret, responder_dh_secret) = match decoded_init_frame {
        Frame::HandshakeInit(payload) => {
            let anek_sk = anek_identity.signing_key();

            let resp_out = HandshakeResponder::process_init_and_respond(
                &anek_sk,
                &payload.sender_pubkey,
                &payload.recipient_pubkey,
                &payload.ephemeral_x25519,
                &payload.ml_kem_pk,
                &payload.signature,
            )
            .expect("Failed to process init at responder");

            let resp_payload =
                HandshakeResponsePayload::new(anek_pubkey, payload.sender_pubkey, &resp_out);

            let responder_secret = resp_out.master_secret.0;
            let responder_dh_secret = resp_out.x25519_secret;

            (resp_payload, responder_secret, responder_dh_secret)
        }

        _ => panic!("Expected HandshakeInit frame"),
    };

    let resp_frame = Frame::HandshakeResponse(resp_payload);

    let encoded_resp = resp_frame
        .encode()
        .expect("Failed to encode HandshakeResponse frame");

    let decoded_resp_frame =
        Frame::decode(&encoded_resp).expect("Failed to decode HandshakeResponse frame");

    let initiator_secret = match decoded_resp_frame {
        Frame::HandshakeResponse(payload) => {
            initiator
                .process_response(
                    &payload.sender_pubkey,
                    &payload.recipient_pubkey,
                    &payload.ephemeral_x25519,
                    &payload.ml_kem_ct,
                    &payload.signature,
                )
                .expect("Failed to process response at initiator")
                .0
        }

        _ => panic!("Expected HandshakeResponse frame"),
    };

    assert_eq!(
        initiator_secret, responder_secret,
        "Master secrets must match after PQ-hybrid handshake"
    );

    let responder_dh_public = X25519PublicKey::from(&responder_dh_secret);

    let mut ventie_ratchet = DoubleRatchet::init_initiator(initiator_secret, responder_dh_public);

    let mut anek_ratchet = DoubleRatchet::init_responder(responder_secret, responder_dh_secret);

    let ad = b"marrow-e2e-v1";

    let encrypted_from_ventie = ventie_ratchet
        .encrypt(b"Hello Anek", ad)
        .expect("Ventie's ratchet encryption failed");

    let decrypted_by_anek = anek_ratchet
        .decrypt(&encrypted_from_ventie, ad)
        .expect("Anek failed to decrypt Ventie's message");

    assert_eq!(decrypted_by_anek, b"Hello Anek");

    let encrypted_from_anek = anek_ratchet
        .encrypt(b"Hello Ventie", ad)
        .expect("Anek's ratchet encryption failed");

    let decrypted_by_ventie = ventie_ratchet
        .decrypt(&encrypted_from_anek, ad)
        .expect("Ventie failed to decrypt Anek's message");

    assert_eq!(decrypted_by_ventie, b"Hello Ventie");
}

#[test]
fn test_padded_message_frame_roundtrip() {
    let msg_payload = EncryptedMessagePayload {
        recipient_pubkey: [0xAA; 32],
        dh_pubkey: [0x42; 32],
        sequence_number: 1,
        previous_chain_length: 0,
        nonce: [0x07; 12],
        ciphertext: vec![1, 2, 3, 4, 5],
    };

    let msg_frame = Frame::Message(msg_payload);

    let padded_encoded_msg = msg_frame
        .encode_padded()
        .expect("Failed to encode padded message frame");

    assert_eq!(
        padded_encoded_msg.len() % 256,
        0,
        "Padded frames must align to 256-byte blocks"
    );

    let decoded_msg_frame =
        Frame::decode(&padded_encoded_msg).expect("Failed to decode padded message frame");

    assert_eq!(msg_frame, decoded_msg_frame);
}
