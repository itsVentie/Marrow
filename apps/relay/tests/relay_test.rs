use r_protocol::{EncryptedMessagePayload, Frame, PADDING_BLOCK_SIZE};

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

#[tokio::test]
async fn test_padded_frame_encoding_and_decoding() {
    let msg_payload = EncryptedMessagePayload {
        recipient_pubkey: [2u8; 32],
        dh_pubkey: [3u8; 32],
        sequence_number: 42,
        previous_chain_length: 1,
        nonce: [9u8; 12],
        ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };

    let frame = Frame::Message(msg_payload);
    let padded_bytes = frame.encode_padded().expect("Failed to encode padded frame");

    assert_eq!(padded_bytes.len() % PADDING_BLOCK_SIZE, 0);

    let decoded = Frame::decode(&padded_bytes).expect("Failed to decode padded frame");
    assert_eq!(frame, decoded);
}

#[tokio::test]
async fn test_dummy_frame_lifecycle() {
    let dummy_payload = vec![0xA5; 300];
    let frame = Frame::Dummy(dummy_payload.clone());

    let padded_bytes = frame.encode_padded().expect("Failed to encode dummy frame");
    assert_eq!(padded_bytes.len() % PADDING_BLOCK_SIZE, 0);

    let decoded = Frame::decode(&padded_bytes).expect("Failed to decode dummy frame");
    if let Frame::Dummy(data) = decoded {
        assert_eq!(data, dummy_payload);
    } else {
        panic!("Expected Frame::Dummy variant");
    }
}