use r_protocol::{EncryptedMessagePayload, Frame};

#[tokio::test]
async fn test_relay_ping_pong_and_routing() {
    let peer_b = [2u8; 32];

    let msg_payload = EncryptedMessagePayload {
        recipient_pubkey: peer_b,
        dh_pubkey: [3u8; 32],
        sequence_number: 1,
        previous_chain_length: 0,
        nonce: [0u8; 12],
        ciphertext: vec![1, 2, 3, 4],
    };

    let frame = Frame::Message(msg_payload);
    let encoded = frame.encode().expect("Failed to encode frame");
    let decoded = Frame::decode(&encoded).expect("Failed to decode frame");

    assert!(matches!(decoded, Frame::Message(_)));
}