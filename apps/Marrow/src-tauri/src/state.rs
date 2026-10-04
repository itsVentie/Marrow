use r_crypto::handshake::HandshakeInitiator;
use r_crypto::{DoubleRatchet, Identity};
use r_network::NetworkCommand;
use r_storage::{SearchIndex, StorageEngine};
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
    pub search: Mutex<Option<SearchIndex>>,
    pub identity: Mutex<Option<Identity>>,
    pub network_cmd: Mutex<Option<mpsc::Sender<NetworkCommand>>>,
    pub crypto_sessions: Mutex<HashMap<String, CryptoSession>>,
    pub pending_handshakes: Mutex<HashMap<String, HandshakeInitiator>>,
    pub network_task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    pub peer_id_to_pubkey: Mutex<HashMap<libp2p::PeerId, String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            storage: Mutex::new(None),
            search: Mutex::new(None),
            identity: Mutex::new(None),
            network_cmd: Mutex::new(None),
            network_task: Mutex::new(None),
            crypto_sessions: Mutex::new(HashMap::new()),
            pending_handshakes: Mutex::new(HashMap::new()),
            peer_id_to_pubkey: Mutex::new(HashMap::new()),
        }
    }
}
