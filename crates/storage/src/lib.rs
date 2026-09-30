#![allow(clippy::result_large_err)]

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use r_crypto::EncryptedVault;
use rand::rngs::OsRng;
use rand::RngCore;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod search;

pub use search::{SearchError, SearchIndex, SearchResult};

const VAULT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("system_vault");
const CONTACTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("contacts");
const SESSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");
const MESSAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("messages");

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),

    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),

    #[error("Storage error: {0}")]
    Storage(#[from] redb::StorageError),

    #[error("Serialization error")]
    SerializationError,

    #[error("Encryption error")]
    EncryptionError,

    #[error("Decryption error")]
    DecryptionError,

    #[error("Key not set for encrypted storage")]
    KeyNotSet,

    #[error("Item not found")]
    NotFound,
}

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

pub struct StorageEngine {
    db: Database,
    key: Option<[u8; 32]>,
}

impl StorageEngine {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;

        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(VAULT_TABLE)?;
            let _ = write_txn.open_table(CONTACTS_TABLE)?;
            let _ = write_txn.open_table(SESSIONS_TABLE)?;
            let _ = write_txn.open_table(MESSAGES_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db, key: None })
    }

    pub fn set_encryption_key(&mut self, key: [u8; 32]) {
        self.key = Some(key);
    }

    fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<Vec<u8>, StorageError> {
        let key = self.key.as_ref().ok_or(StorageError::KeyNotSet)?;
        let cipher = XChaCha20Poly1305::new(key.into());

        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| StorageError::EncryptionError)?;

        let mut out = Vec::with_capacity(24 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn decrypt_bytes(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        if data.len() < 24 {
            return Err(StorageError::DecryptionError);
        }

        let key = self.key.as_ref().ok_or(StorageError::KeyNotSet)?;
        let cipher = XChaCha20Poly1305::new(key.into());

        let nonce = XNonce::from_slice(&data[..24]);
        let ciphertext = &data[24..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| StorageError::DecryptionError)
    }

    pub fn save_vault(&self, vault: &EncryptedVault) -> Result<(), StorageError> {
        let bytes = bincode::serialize(vault).map_err(|_| StorageError::SerializationError)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(VAULT_TABLE)?;
            table.insert("identity", bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn load_vault(&self) -> Result<EncryptedVault, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(VAULT_TABLE)?;
        let value = table.get("identity")?.ok_or(StorageError::NotFound)?;

        let vault: EncryptedVault =
            bincode::deserialize(value.value()).map_err(|_| StorageError::SerializationError)?;

        Ok(vault)
    }

    pub fn save_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let raw_bytes = bincode::serialize(contact).map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTACTS_TABLE)?;
            table.insert(contact.pubkey_hex.as_str(), encrypted_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_contact(&self, pubkey_hex: &str) -> Result<Contact, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;
        let value = table.get(pubkey_hex)?.ok_or(StorageError::NotFound)?;

        let decrypted_bytes = self.decrypt_bytes(value.value())?;
        let contact: Contact =
            bincode::deserialize(&decrypted_bytes).map_err(|_| StorageError::SerializationError)?;

        Ok(contact)
    }

    pub fn list_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;
        let mut contacts = Vec::new();

        for entry in table.iter()? {
            let (_key_guard, val_guard) = entry?;
            let decrypted_bytes = self.decrypt_bytes(val_guard.value())?;
            let contact: Contact = bincode::deserialize(&decrypted_bytes)
                .map_err(|_| StorageError::SerializationError)?;
            contacts.push(contact);
        }

        Ok(contacts)
    }

    pub fn delete_contact(&self, pubkey_hex: &str) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(CONTACTS_TABLE)?;
            let opt = table.remove(pubkey_hex)?;
            opt.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn create_session(&self, peer_pubkey_hex: &str, now: i64) -> Result<Session, StorageError> {
        let session = Session {
            id: peer_pubkey_hex.to_string(),
            peer_pubkey_hex: peer_pubkey_hex.to_string(),
            created_at: now,
            last_activity: now,
        };

        let raw_bytes = bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.insert(peer_pubkey_hex, encrypted_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Session, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SESSIONS_TABLE)?;
        let value = table.get(session_id)?.ok_or(StorageError::NotFound)?;

        let decrypted_bytes = self.decrypt_bytes(value.value())?;
        bincode::deserialize(&decrypted_bytes).map_err(|_| StorageError::SerializationError)
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SESSIONS_TABLE)?;
        let mut sessions = Vec::new();

        for entry in table.iter()? {
            let (_k, v) = entry?;
            let decrypted_bytes = self.decrypt_bytes(v.value())?;
            let session: Session =
                bincode::deserialize(&decrypted_bytes).map_err(|_| StorageError::SerializationError)?;
            sessions.push(session);
        }

        Ok(sessions)
    }

    pub fn update_session_activity(
        &self,
        session_id: &str,
        timestamp: i64,
    ) -> Result<(), StorageError> {
        let mut session = self.get_session(session_id)?;
        session.last_activity = timestamp;

        let raw_bytes = bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.insert(session_id, encrypted_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_session(&self, session_id: &str) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            let opt = table.remove(session_id)?;
            opt.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn store_message(&self, msg: &StoredMessage) -> Result<(), StorageError> {
        let key = message_key(&msg.session_id, msg.sequence_number);
        let raw_bytes = bincode::serialize(msg).map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;
            table.insert(key.as_str(), encrypted_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_messages_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<StoredMessage>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(MESSAGES_TABLE)?;

        let prefix = format!("{session_id}/");
        let prefix_end = format!("{session_id}0");

        let mut messages = Vec::new();
        for entry in table.range(prefix.as_str()..prefix_end.as_str())? {
            let (_k, v) = entry?;
            let decrypted_bytes = self.decrypt_bytes(v.value())?;
            let msg: StoredMessage =
                bincode::deserialize(&decrypted_bytes).map_err(|_| StorageError::SerializationError)?;
            messages.push(msg);
        }

        Ok(messages)
    }

    pub fn delete_messages_for_session(&self, session_id: &str) -> Result<u64, StorageError> {
        let prefix = format!("{session_id}/");
        let prefix_end = format!("{session_id}0");

        let keys: Vec<String> = {
            let read_txn = self.db.begin_read()?;
            let table = read_txn.open_table(MESSAGES_TABLE)?;
            table
                .range(prefix.as_str()..prefix_end.as_str())?
                .map(|e| e.map(|(k, _)| k.value().to_string()))
                .collect::<Result<_, _>>()?
        };

        let count = keys.len() as u64;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;
            for key in &keys {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(count)
    }
}

fn message_key(session_id: &str, sequence_number: u64) -> String {
    format!("{session_id}/{sequence_number:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use r_crypto::Identity;
    use tempfile::NamedTempFile;

    fn open_tmp() -> (NamedTempFile, StorageEngine) {
        let f = NamedTempFile::new().unwrap();
        let mut e = StorageEngine::open(f.path()).unwrap();
        e.set_encryption_key([7u8; 32]);
        (f, e)
    }

    #[test]
    fn test_vault_storage_cycle() {
        let (_f, engine) = open_tmp();

        let identity = Identity::generate();
        let password = b"super_secret_master_password";
        let vault = identity.export_encrypted(password).unwrap();

        engine.save_vault(&vault).unwrap();
        let loaded_vault = engine.load_vault().unwrap();

        let decrypted = Identity::import_encrypted(&loaded_vault, password).unwrap();
        assert_eq!(
            identity.verifying_key().to_bytes(),
            decrypted.verifying_key().to_bytes()
        );
    }

    #[test]
    fn test_contact_storage_cycle() {
        let (_f, engine) = open_tmp();

        let contact = Contact {
            pubkey_hex: "1234567890abcdef".to_string(),
            alias: "Ventie".to_string(),
            added_at: 1700000000,
        };

        engine.save_contact(&contact).unwrap();
        let loaded_contact = engine.get_contact(&contact.pubkey_hex).unwrap();

        assert_eq!(contact, loaded_contact);
    }

    #[test]
    fn test_session_crud() {
        let (_f, engine) = open_tmp();

        let session = engine.create_session("deadbeef", 1_000_000).unwrap();
        assert_eq!(session.peer_pubkey_hex, "deadbeef");
        assert_eq!(session.id, "deadbeef");

        let loaded = engine.get_session(&session.id).unwrap();
        assert_eq!(loaded, session);
    }
}