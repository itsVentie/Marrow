use r_crypto::Identity;
use r_network::NetworkCommand;
use r_storage::StorageEngine;
use std::sync::Mutex;
use tokio::sync::mpsc;

pub struct AppState {
    pub storage: Mutex<Option<StorageEngine>>,
    pub identity: Mutex<Option<Identity>>,
    pub network_cmd: Mutex<Option<mpsc::Sender<NetworkCommand>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            storage: Mutex::new(None),
            identity: Mutex::new(None),
            network_cmd: Mutex::new(None),
        }
    }
}
