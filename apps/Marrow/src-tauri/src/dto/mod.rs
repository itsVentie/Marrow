use r_storage::MessageDirection;
use serde::{Deserialize, Serialize};

#[derive(serde::Serialize)]
pub struct PublicIdentityDto {
    pub pubkey_hex: String,
}

#[derive(serde::Serialize)]
pub struct KeyFileInfoDto {
    pub filename: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    pub msg_id: String,
    pub peer_id: String,
    pub timestamp: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct DecryptedMessageDto {
    pub session_id: String,
    pub sender_pubkey_hex: String,
    pub payload_hex: String,
    pub timestamp: i64,
    pub direction: MessageDirection,
    pub sequence_number: u64,
}

#[derive(Clone, serde::Serialize)]
pub struct NetworkEventPayload {
    pub peer_id: String,
    pub data_hex: Option<String>,
}
