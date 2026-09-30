use r_crypto::handshake::{InitiatorOutput, ResponderOutput};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_FRAME_SIZE: usize = 65536;
pub const PADDING_BLOCK_SIZE: usize = 256;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Failed to serialize frame: {0}")]
    Serialization(String),

    #[error("Failed to deserialize frame: {0}")]
    Deserialization(String),

    #[error("Frame size ({0} bytes) exceeds maximum limit of {MAX_FRAME_SIZE} bytes")]
    FrameTooLarge(usize),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HandshakeInitPayload {
    pub sender_pubkey: [u8; 32],
    pub recipient_pubkey: [u8; 32],
    pub ephemeral_x25519: [u8; 32],
    pub ml_kem_pk: Vec<u8>,
    pub signature: Vec<u8>,
}

impl HandshakeInitPayload {
    pub fn new(sender_pubkey: [u8; 32], init_output: InitiatorOutput) -> Self {
        Self {
            sender_pubkey,
            recipient_pubkey: init_output.recipient_pubkey,
            ephemeral_x25519: init_output.x25519_public,
            ml_kem_pk: init_output.ml_kem_public,
            signature: init_output.signature.to_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HandshakeResponsePayload {
    pub sender_pubkey: [u8; 32],
    pub recipient_pubkey: [u8; 32],
    pub ephemeral_x25519: [u8; 32],
    pub ml_kem_ct: Vec<u8>,
    pub signature: Vec<u8>,
}

impl HandshakeResponsePayload {
    pub fn new(
        sender_pubkey: [u8; 32],
        recipient_pubkey: [u8; 32],
        resp_output: &ResponderOutput,
    ) -> Self {
        Self {
            sender_pubkey,
            recipient_pubkey,
            ephemeral_x25519: resp_output.x25519_public,
            ml_kem_ct: resp_output.ml_kem_ciphertext.clone(),
            signature: resp_output.signature.to_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EncryptedMessagePayload {
    pub recipient_pubkey: [u8; 32],
    pub dh_pubkey: [u8; 32],
    pub sequence_number: u64,
    pub previous_chain_length: u32,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Frame {
    HandshakeInit(HandshakeInitPayload),
    HandshakeResponse(HandshakeResponsePayload),
    Message(EncryptedMessagePayload),
    Ack { message_id: [u8; 16] },
    Ping,
    Pong,
    Dummy(Vec<u8>),
}

impl Frame {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let bytes =
            bincode::serialize(self).map_err(|e| ProtocolError::Serialization(e.to_string()))?;

        if bytes.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge(bytes.len()));
        }

        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge(bytes.len()));
        }

        bincode::deserialize(bytes).map_err(|e| ProtocolError::Deserialization(e.to_string()))
    }

    pub fn encode_padded(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut encoded = self.encode()?;
        let current_len = encoded.len();

        let target_len = current_len.div_ceil(PADDING_BLOCK_SIZE) * PADDING_BLOCK_SIZE;

        let padding_needed = target_len - current_len;

        if padding_needed > 0 {
            encoded.resize(target_len, 0);
        }

        if encoded.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge(encoded.len()));
        }

        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r_crypto::handshake::{HandshakeInitiator, HandshakeResponder};

    #[test]
    fn test_ping_pong_frame() {
        let frame = Frame::Ping;

        let encoded = frame.encode().unwrap();
        let decoded = Frame::decode(&encoded).unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_dummy_frame() {
        let frame = Frame::Dummy(vec![0xAA; 128]);

        let encoded = frame.encode().unwrap();
        let decoded = Frame::decode(&encoded).unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_padded_encoding() {
        let frame = Frame::Ping;

        let padded_bytes = frame.encode_padded().unwrap();

        assert_eq!(padded_bytes.len() % PADDING_BLOCK_SIZE, 0);

        let decoded = Frame::decode(&padded_bytes).unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_encrypted_message_frame() {
        let payload = EncryptedMessagePayload {
            recipient_pubkey: [0x11; 32],
            dh_pubkey: [0x22; 32],
            sequence_number: 10,
            previous_chain_length: 2,
            nonce: [0x33; 12],
            ciphertext: vec![0xde, 0xad, 0xbe, 0xef],
        };

        let frame = Frame::Message(payload);

        let encoded = frame.encode().unwrap();
        let decoded = Frame::decode(&encoded).unwrap();

        assert_eq!(frame, decoded);
    }

    #[test]
    fn test_frame_size_overflow() {
        let oversized_payload = EncryptedMessagePayload {
            recipient_pubkey: [0x00; 32],
            dh_pubkey: [0x00; 32],
            sequence_number: 0,
            previous_chain_length: 0,
            nonce: [0x00; 12],
            ciphertext: vec![0u8; MAX_FRAME_SIZE],
        };

        let frame = Frame::Message(oversized_payload);
        let result = frame.encode();

        assert!(matches!(result, Err(ProtocolError::FrameTooLarge(_))));
    }

    #[test]
    fn test_end_to_end_handshake_via_frames() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let initiator_signing_key = SigningKey::generate(&mut OsRng);

        let responder_signing_key = SigningKey::generate(&mut OsRng);

        let initiator_pubkey = *initiator_signing_key.verifying_key().as_bytes();

        let responder_pubkey = *responder_signing_key.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_out = initiator.generate_init_payload(&initiator_signing_key, &responder_pubkey);

        let init_frame =
            Frame::HandshakeInit(HandshakeInitPayload::new(initiator_pubkey, init_out));

        let init_bytes = init_frame.encode().expect("Failed to encode init frame");

        let decoded_init_frame = Frame::decode(&init_bytes).expect("Failed to decode init frame");

        let (resp_payload, responder_secret) = match decoded_init_frame {
            Frame::HandshakeInit(payload) => {
                let resp_out = HandshakeResponder::process_init_and_respond(
                    &responder_signing_key,
                    &payload.sender_pubkey,
                    &payload.recipient_pubkey,
                    &payload.ephemeral_x25519,
                    &payload.ml_kem_pk,
                    &payload.signature,
                )
                .expect("Failed to process init at responder");

                let secret = resp_out.master_secret.0;

                let resp_payload = HandshakeResponsePayload::new(
                    responder_pubkey,
                    payload.sender_pubkey,
                    &resp_out,
                );

                (resp_payload, secret)
            }

            _ => {
                panic!("Expected HandshakeInit frame");
            }
        };

        let resp_frame = Frame::HandshakeResponse(resp_payload);

        let resp_bytes = resp_frame
            .encode()
            .expect("Failed to encode response frame");

        let decoded_resp_frame =
            Frame::decode(&resp_bytes).expect("Failed to decode response frame");

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

            _ => {
                panic!("Expected HandshakeResponse frame");
            }
        };

        assert_eq!(
            initiator_secret, responder_secret,
            "Master secrets must match after PQ-hybrid handshake"
        );
    }

    #[test]
    fn test_handshake_init_recipient_survives_roundtrip() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let initiator_signing_key = SigningKey::generate(&mut OsRng);

        let responder_signing_key = SigningKey::generate(&mut OsRng);

        let initiator_pubkey = *initiator_signing_key.verifying_key().as_bytes();

        let responder_pubkey = *responder_signing_key.verifying_key().as_bytes();

        let mut initiator = HandshakeInitiator::new();

        let init_out = initiator.generate_init_payload(&initiator_signing_key, &responder_pubkey);

        let frame = Frame::HandshakeInit(HandshakeInitPayload::new(initiator_pubkey, init_out));

        let decoded = Frame::decode(&frame.encode().unwrap()).unwrap();

        match decoded {
            Frame::HandshakeInit(payload) => {
                assert_eq!(payload.sender_pubkey, initiator_pubkey);

                assert_eq!(payload.recipient_pubkey, responder_pubkey);
            }

            _ => panic!("Expected HandshakeInit frame"),
        }
    }
}
