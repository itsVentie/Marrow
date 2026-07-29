use r_crypto::Identity;
use r_storage::StorageEngine;
use std::sync::Mutex;

pub struct AppState {
    pub storage: Mutex<Option<StorageEngine>>,
    pub identity: Mutex<Option<Identity>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            storage: Mutex::new(None),
            identity: Mutex::new(None),
        }
    }
}
