use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Contact {
    pub pubkey_hex: String,
    pub alias: String,
    pub added_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub peer_pubkey_hex: String,
    pub created_at: i64,
    pub last_activity: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StoredMessage {
    pub session_id: String,
    pub sender_pubkey_hex: String,
    pub ciphertext: Vec<u8>,
    pub timestamp: i64,
    pub direction: MessageDirection,
    pub sequence_number: u64,
}