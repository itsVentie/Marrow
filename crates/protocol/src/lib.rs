use r_crypto::handshake::{InitiatorOutput, ResponderOutput};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_FRAME_SIZE: usize = 65536;

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
    pub ephemeral_x25519: [u8; 32],
    pub ml_kem_pk: Vec<u8>,
}

impl HandshakeInitPayload {
    pub fn new(sender_pubkey: [u8; 32], init_output: InitiatorOutput) -> Self {
        Self {
            sender_pubkey,
            ephemeral_x25519: init_output.x25519_public,
            ml_kem_pk: init_output.ml_kem_public,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HandshakeResponsePayload {
    pub recipient_pubkey: [u8; 32],
    pub ephemeral_x25519: [u8; 32],
    pub ml_kem_ct: Vec<u8>,
}

impl HandshakeResponsePayload {
    pub fn new(recipient_pubkey: [u8; 32], resp_output: &ResponderOutput) -> Self {
        Self {
            recipient_pubkey,
            ephemeral_x25519: resp_output.x25519_public,
            ml_kem_ct: resp_output.ml_kem_ciphertext.clone(),
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
        let sender_identity = [0x01; 32];
        let responder_identity = [0x02; 32];

        let initiator = HandshakeInitiator::new();
        let init_out = initiator.generate_init_payload();

        let init_frame = Frame::HandshakeInit(HandshakeInitPayload::new(sender_identity, init_out));
        let init_bytes = init_frame.encode().expect("Failed to encode init frame");

        let decoded_init_frame = Frame::decode(&init_bytes).expect("Failed to decode init frame");

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

        let resp_frame = Frame::HandshakeResponse(HandshakeResponsePayload::new(
            responder_identity,
            &resp_out,
        ));
        let resp_bytes = resp_frame.encode().expect("Failed to encode resp frame");

        let decoded_resp_frame = Frame::decode(&resp_bytes).expect("Failed to decode resp frame");

        let initiator_secret = match decoded_resp_frame {
            Frame::HandshakeResponse(payload) => initiator
                .process_response(&payload.ephemeral_x25519, &payload.ml_kem_ct)
                .expect("Failed to process response at initiator")
                .0,
            _ => panic!("Expected HandshakeResponse frame"),
        };

        assert_eq!(initiator_secret, responder_secret);
    }
}