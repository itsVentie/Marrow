use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};
use r_crypto::ratchet::DoubleRatchet;
use r_crypto::x25519_dalek::PublicKey as X25519PublicKey;
use r_crypto::Identity;
use r_protocol::{EncryptedMessagePayload, Frame, HandshakeInitPayload, HandshakeResponsePayload};

#[test]
fn test_e2e_handshake_frame_and_ratchet_pipeline() {
    let alice_identity = Identity::generate();
    let bob_identity = Identity::generate();

    let alice_pubkey = *alice_identity.verifying_key().as_bytes();
    let bob_pubkey = *bob_identity.verifying_key().as_bytes();

    let mut initiator = HandshakeInitiator::new();

    let alice_sk = alice_identity.signing_key();

    let init_out = initiator.generate_init_payload(&alice_sk, &bob_pubkey);

    let init_payload = HandshakeInitPayload::new(alice_pubkey, init_out);
    let init_frame = Frame::HandshakeInit(init_payload);

    let encoded_init = init_frame
        .encode()
        .expect("Failed to encode HandshakeInit frame");

    let decoded_init_frame =
        Frame::decode(&encoded_init).expect("Failed to decode HandshakeInit frame");

    let (resp_payload, responder_secret, responder_dh_secret) = match decoded_init_frame {
        Frame::HandshakeInit(payload) => {
            let bob_sk = bob_identity.signing_key();

            let resp_out = HandshakeResponder::process_init_and_respond(
                &bob_sk,
                &payload.sender_pubkey,
                &payload.recipient_pubkey,
                &payload.ephemeral_x25519,
                &payload.ml_kem_pk,
                &payload.signature,
            )
            .expect("Failed to process init at responder");

            let resp_payload =
                HandshakeResponsePayload::new(bob_pubkey, payload.sender_pubkey, &resp_out);

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

    let mut alice_ratchet = DoubleRatchet::init_initiator(initiator_secret, responder_dh_public);

    let mut bob_ratchet = DoubleRatchet::init_responder(responder_secret, responder_dh_secret);

    let ad = b"marrow-e2e-v1";

    let encrypted_from_alice = alice_ratchet
        .encrypt(b"Hello Bob", ad)
        .expect("Alice's ratchet encryption failed");

    let decrypted_by_bob = bob_ratchet
        .decrypt(&encrypted_from_alice, ad)
        .expect("Bob failed to decrypt Alice's message");

    assert_eq!(decrypted_by_bob, b"Hello Bob");

    let encrypted_from_bob = bob_ratchet
        .encrypt(b"Hello Alice", ad)
        .expect("Bob's ratchet encryption failed");

    let decrypted_by_alice = alice_ratchet
        .decrypt(&encrypted_from_bob, ad)
        .expect("Alice failed to decrypt Bob's message");

    assert_eq!(decrypted_by_alice, b"Hello Alice");
}

#[test]
fn test_padded_message_frame_roundtrip() {
    let msg_payload = EncryptedMessagePayload {
        recipient_pubkey: [0x42; 32],
        ciphertext: vec![0xAB; 64],
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
