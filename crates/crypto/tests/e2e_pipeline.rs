use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};
use r_crypto::Identity;
use r_protocol::{EncryptedMessagePayload, Frame, HandshakeInitPayload, HandshakeResponsePayload};

#[test]
fn test_e2e_handshake_and_frame_pipeline() {
    let Ventie_identity = Identity::generate();
    let Anek_identity = Identity::generate();

    let Ventie_pubkey = Ventie_identity.verifying_key().to_bytes();
    let Anek_pubkey = Anek_identity.verifying_key().to_bytes();

    let initiator = HandshakeInitiator::new();
    let init_out = initiator.generate_init_payload();

    let init_payload = HandshakeInitPayload::new(Ventie_pubkey, init_out);
    let init_frame = Frame::HandshakeInit(init_payload);

    let encoded_init = init_frame
        .encode()
        .expect("Failed to encode HandshakeInit frame");

    let decoded_init_frame =
        Frame::decode(&encoded_init).expect("Failed to decode HandshakeInit frame");

    let (resp_out, responder_secret) = match decoded_init_frame {
        Frame::HandshakeInit(payload) => {
            let resp_out = HandshakeResponder::process_init_and_respond(
                &payload.ephemeral_x25519,
                &payload.ml_kem_pk,
            )
            .expect("Failed to process init at responder");

            let secret = resp_out.master_secret.0;
            (resp_out, secret)
        }
        _ => panic!("Expected HandshakeInit frame"),
    };

    let resp_payload = HandshakeResponsePayload::new(Anek_pubkey, &resp_out);
    let resp_frame = Frame::HandshakeResponse(resp_payload);

    let encoded_resp = resp_frame
        .encode()
        .expect("Failed to encode HandshakeResponse frame");

    let decoded_resp_frame =
        Frame::decode(&encoded_resp).expect("Failed to decode HandshakeResponse frame");

    let initiator_secret = match decoded_resp_frame {
        Frame::HandshakeResponse(payload) => {
            initiator
                .process_response(&payload.ephemeral_x25519, &payload.ml_kem_ct)
                .expect("Failed to process response at initiator")
                .0
        }
        _ => panic!("Expected HandshakeResponse frame"),
    };

    assert_eq!(
        initiator_secret, responder_secret,
        "Master secrets must match after PQ-hybrid handshake"
    );

    let msg_payload = EncryptedMessagePayload {
        recipient_pubkey: Anek_pubkey,
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

    let decoded_msg_frame =
        Frame::decode(&padded_encoded_msg).expect("Failed to decode padded message frame");

    assert_eq!(msg_frame, decoded_msg_frame);
}
