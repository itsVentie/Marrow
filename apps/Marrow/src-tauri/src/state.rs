use r_crypto::handshake::HandshakeInitiator;
use r_crypto::ratchet::DoubleRatchet;
use r_crypto::Identity;
use r_network::NetworkCommand;
use r_storage::StorageEngine;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct CryptoSession {
    pub ratchet: DoubleRatchet,
    pub peer_pubkey_hex: String,
    pub sequence_number: u64,
}

pub struct AppState {
    pub storage: Mutex<Option<StorageEngine>>,
    pub identity: Mutex<Option<Identity>>,
    pub network_cmd: Mutex<Option<mpsc::Sender<NetworkCommand>>>,
    pub crypto_sessions: Mutex<HashMap<String, CryptoSession>>,
    pub pending_handshakes: Mutex<HashMap<String, HandshakeInitiator>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            storage: Mutex::new(None),
            identity: Mutex::new(None),
            network_cmd: Mutex::new(None),
            crypto_sessions: Mutex::new(HashMap::new()),
            pending_handshakes: Mutex::new(HashMap::new()),
        }
    }
}