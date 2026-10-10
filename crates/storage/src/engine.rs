use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use r_crypto::EncryptedVault;
use rand::rngs::OsRng;
use rand::RngCore;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use zeroize::Zeroize;

use crate::error::StorageError;
use crate::models::{Contact, Session, StoredMessage};

const VAULT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("system_vault");
const CONTACTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("contacts");
const CONTACT_ADDRESSES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("contact_addresses");
const SESSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");
const MESSAGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("messages");

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
            let _ = write_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
            let _ = write_txn.open_table(SESSIONS_TABLE)?;
            let _ = write_txn.open_table(MESSAGES_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db, key: None })
    }

    pub fn set_encryption_key(&mut self, key: [u8; 32]) {
        self.clear_encryption_key();
        self.key = Some(key);
    }

    pub fn clear_encryption_key(&mut self) {
        if let Some(mut key) = self.key.take() {
            key.zeroize();
        }
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
        self.save_contact_with_address(contact, None)
    }

    pub fn save_contact_with_address(
        &self,
        contact: &Contact,
        multiaddr: Option<&str>,
    ) -> Result<(), StorageError> {
        let raw_contact =
            bincode::serialize(contact).map_err(|_| StorageError::SerializationError)?;
        let encrypted_contact = self.encrypt_bytes(&raw_contact)?;

        let encrypted_address = match multiaddr {
            Some(address) => {
                let raw_address = bincode::serialize(&address.to_string())
                    .map_err(|_| StorageError::SerializationError)?;
                Some(self.encrypt_bytes(&raw_address)?)
            }
            None => None,
        };

        let write_txn = self.db.begin_write()?;
        {
            let mut contacts_table = write_txn.open_table(CONTACTS_TABLE)?;
            contacts_table.insert(contact.pubkey_hex.as_str(), encrypted_contact.as_slice())?;
        }
        if let Some(encrypted_address) = encrypted_address {
            let mut addresses_table = write_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
            addresses_table.insert(contact.pubkey_hex.as_str(), encrypted_address.as_slice())?;
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

    pub fn get_contact_address(&self, pubkey_hex: &str) -> Result<String, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
        let value = table.get(pubkey_hex)?.ok_or(StorageError::NotFound)?;
        let decrypted_bytes = self.decrypt_bytes(value.value())?;
        let multiaddr: String =
            bincode::deserialize(&decrypted_bytes).map_err(|_| StorageError::SerializationError)?;
        Ok(multiaddr)
    }

    pub fn save_contact_address(
        &self,
        pubkey_hex: &str,
        multiaddr: &str,
    ) -> Result<(), StorageError> {
        let raw_bytes = bincode::serialize(&multiaddr.to_string())
            .map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
            table.insert(pubkey_hex, encrypted_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_contact_address(&self, pubkey_hex: &str) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
            let result = table.remove(pubkey_hex)?;
            result.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn delete_contact(&self, pubkey_hex: &str) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut contacts_table = write_txn.open_table(CONTACTS_TABLE)?;
            let result = contacts_table.remove(pubkey_hex)?;
            result.is_some()
        };
        {
            let mut addresses_table = write_txn.open_table(CONTACT_ADDRESSES_TABLE)?;
            let _ = addresses_table.remove(pubkey_hex)?;
        }
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
        let raw_bytes =
            bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
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
            let session: Session = bincode::deserialize(&decrypted_bytes)
                .map_err(|_| StorageError::SerializationError)?;
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
        let raw_bytes =
            bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
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
            let result = table.remove(session_id)?;
            result.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn store_message(&self, msg: &StoredMessage) -> Result<(), StorageError> {
        let raw_bytes = bincode::serialize(msg).map_err(|_| StorageError::SerializationError)?;
        let encrypted_bytes = self.encrypt_bytes(&raw_bytes)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;

            let prefix = format!("{}/", msg.session_id);
            let prefix_end = format!("{}0", msg.session_id);

            let next_index = {
                let last = table
                    .range(prefix.as_str()..prefix_end.as_str())?
                    .next_back();
                match last {
                    Some(entry) => {
                        let (k, _) = entry?;
                        let suffix = k
                            .value()
                            .rsplit_once('/')
                            .map(|(_, s)| s.to_string())
                            .ok_or(StorageError::SerializationError)?;
                        let last_index = u64::from_str_radix(&suffix, 16)
                            .map_err(|_| StorageError::SerializationError)?;
                        last_index
                            .checked_add(1)
                            .ok_or(StorageError::SerializationError)?
                    }
                    None => 0,
                }
            };

            let key = message_key(&msg.session_id, next_index);
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
            let msg: StoredMessage = bincode::deserialize(&decrypted_bytes)
                .map_err(|_| StorageError::SerializationError)?;
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

fn message_key(session_id: &str, storage_index: u64) -> String {
    format!("{session_id}/{storage_index:016x}")
}

impl Drop for StorageEngine {
    fn drop(&mut self) {
        self.clear_encryption_key();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MessageDirection;

    struct TempDb(std::path::PathBuf);

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn open_engine() -> (StorageEngine, TempDb) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("marrow_test_{}_{}.redb", std::process::id(), nanos));
        let mut engine = StorageEngine::open(&path).unwrap();
        engine.set_encryption_key([7u8; 32]);
        (engine, TempDb(path))
    }

    fn msg(dir: MessageDirection, seq: u64, tag: u8) -> StoredMessage {
        StoredMessage {
            session_id: "sess".into(),
            sender_pubkey_hex: "peer".into(),
            ciphertext: vec![tag],
            timestamp: tag as i64,
            direction: dir,
            sequence_number: seq,
        }
    }

    #[test]
    fn same_sequence_number_does_not_overwrite() {
        let (engine, _db) = open_engine();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 1))
            .unwrap();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 2))
            .unwrap();

        let got = engine.get_messages_for_session("sess").unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].ciphertext, vec![1]);
        assert_eq!(got[1].ciphertext, vec![2]);
    }

    #[test]
    fn inbound_and_outbound_with_same_n_coexist() {
        let (engine, _db) = open_engine();
        engine
            .store_message(&msg(MessageDirection::Outbound, 0, 1))
            .unwrap();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 2))
            .unwrap();
        assert_eq!(engine.get_messages_for_session("sess").unwrap().len(), 2);
    }

    #[test]
    fn legacy_keys_stay_readable_and_new_messages_append_after() {
        let (engine, _db) = open_engine();
        let old = msg(MessageDirection::Inbound, 5, 1);
        let enc = engine
            .encrypt_bytes(&bincode::serialize(&old).unwrap())
            .unwrap();
        let txn = engine.db.begin_write().unwrap();
        {
            let mut t = txn.open_table(MESSAGES_TABLE).unwrap();
            t.insert("sess/0000000000000005", enc.as_slice()).unwrap();
        }
        txn.commit().unwrap();

        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 2))
            .unwrap();

        let got = engine.get_messages_for_session("sess").unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].ciphertext, vec![1]);
        assert_eq!(got[1].ciphertext, vec![2]);
    }

    #[test]
    fn sessions_do_not_share_indexes() {
        let (engine, _db) = open_engine();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 1))
            .unwrap();
        let mut other = msg(MessageDirection::Inbound, 0, 2);
        other.session_id = "other".into();
        engine.store_message(&other).unwrap();

        assert_eq!(engine.get_messages_for_session("sess").unwrap().len(), 1);
        assert_eq!(engine.get_messages_for_session("other").unwrap().len(), 1);
    }

    #[test]
    fn delete_messages_removes_only_target_session() {
        let (engine, _db) = open_engine();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 1))
            .unwrap();
        engine
            .store_message(&msg(MessageDirection::Inbound, 0, 2))
            .unwrap();
        let mut other = msg(MessageDirection::Inbound, 0, 3);
        other.session_id = "other".into();
        engine.store_message(&other).unwrap();

        assert_eq!(engine.delete_messages_for_session("sess").unwrap(), 2);
        assert!(engine.get_messages_for_session("sess").unwrap().is_empty());
        assert_eq!(engine.get_messages_for_session("other").unwrap().len(), 1);
    }
}
