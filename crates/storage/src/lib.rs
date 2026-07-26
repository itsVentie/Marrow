#![allow(clippy::result_large_err)]

use r_crypto::EncryptedVault;
use rand::RngCore;
use rand::rngs::OsRng;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

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

        Ok(Self { db })
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
        let bytes = bincode::serialize(contact).map_err(|_| StorageError::SerializationError)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTACTS_TABLE)?;
            table.insert(contact.pubkey_hex.as_str(), bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_contact(&self, pubkey_hex: &str) -> Result<Contact, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;
        let value = table.get(pubkey_hex)?.ok_or(StorageError::NotFound)?;

        let contact: Contact =
            bincode::deserialize(value.value()).map_err(|_| StorageError::SerializationError)?;

        Ok(contact)
    }

    pub fn list_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;
        let mut contacts = Vec::new();

        for entry in table.iter()? {
            let (_key_guard, val_guard) = entry?;
            let contact: Contact = bincode::deserialize(val_guard.value())
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
        let mut id_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut id_bytes);
        let id = hex::encode(id_bytes);

        let session = Session {
            id: id.clone(),
            peer_pubkey_hex: peer_pubkey_hex.to_string(),
            created_at: now,
            last_activity: now,
        };

        let bytes = bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.insert(id.as_str(), bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Session, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SESSIONS_TABLE)?;
        let value = table.get(session_id)?.ok_or(StorageError::NotFound)?;

        bincode::deserialize(value.value()).map_err(|_| StorageError::SerializationError)
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>, StorageError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SESSIONS_TABLE)?;
        let mut sessions = Vec::new();

        for entry in table.iter()? {
            let (_k, v) = entry?;
            let session: Session =
                bincode::deserialize(v.value()).map_err(|_| StorageError::SerializationError)?;
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

        let bytes = bincode::serialize(&session).map_err(|_| StorageError::SerializationError)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.insert(session_id, bytes.as_slice())?;
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
        let bytes = bincode::serialize(msg).map_err(|_| StorageError::SerializationError)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(MESSAGES_TABLE)?;
            table.insert(key.as_str(), bytes.as_slice())?;
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
        let prefix_end = format!("{session_id}0"); // '0' > '/' in ASCII

        let mut messages = Vec::new();
        for entry in table.range(prefix.as_str()..prefix_end.as_str())? {
            let (_k, v) = entry?;
            let msg: StoredMessage =
                bincode::deserialize(v.value()).map_err(|_| StorageError::SerializationError)?;
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
        let e = StorageEngine::open(f.path()).unwrap();
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
    fn test_contact_crud_operations() {
        let (_f, engine) = open_tmp();

        let c1 = Contact {
            pubkey_hex: "pubkey_1".to_string(),
            alias: "Ventie".to_string(),
            added_at: 100,
        };
        let c2 = Contact {
            pubkey_hex: "pubkey_2".to_string(),
            alias: "Anek".to_string(),
            added_at: 200,
        };

        engine.save_contact(&c1).unwrap();
        engine.save_contact(&c2).unwrap();

        let contacts = engine.list_contacts().unwrap();
        assert_eq!(contacts.len(), 2);

        let deleted = engine.delete_contact(&c1.pubkey_hex).unwrap();
        assert!(deleted);

        let contacts_after = engine.list_contacts().unwrap();
        assert_eq!(contacts_after.len(), 1);
        assert_eq!(contacts_after[0], c2);

        let not_found = engine.get_contact(&c1.pubkey_hex);
        assert!(matches!(not_found, Err(StorageError::NotFound)));
    }

    #[test]
    fn test_session_crud() {
        let (_f, engine) = open_tmp();

        let session = engine.create_session("deadbeef", 1_000_000).unwrap();
        assert_eq!(session.peer_pubkey_hex, "deadbeef");
        assert_eq!(session.created_at, 1_000_000);
        assert_eq!(session.last_activity, 1_000_000);

        let loaded = engine.get_session(&session.id).unwrap();
        assert_eq!(loaded, session);

        engine.update_session_activity(&session.id, 2_000_000).unwrap();
        let updated = engine.get_session(&session.id).unwrap();
        assert_eq!(updated.last_activity, 2_000_000);

        let sessions = engine.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);

        let removed = engine.delete_session(&session.id).unwrap();
        assert!(removed);
        assert!(matches!(
            engine.get_session(&session.id),
            Err(StorageError::NotFound)
        ));
    }

    #[test]
    fn test_message_store_and_retrieve() {
        let (_f, engine) = open_tmp();

        let session = engine.create_session("aabbccdd", 0).unwrap();

        let messages: Vec<StoredMessage> = (0..5)
            .map(|i| StoredMessage {
                session_id: session.id.clone(),
                sender_pubkey_hex: "aabbccdd".to_string(),
                ciphertext: vec![i as u8; 32],
                timestamp: i as i64 * 1000,
                direction: if i % 2 == 0 {
                    MessageDirection::Outbound
                } else {
                    MessageDirection::Inbound
                },
                sequence_number: i,
            })
            .collect();

        for msg in &messages {
            engine.store_message(msg).unwrap();
        }

        let retrieved = engine.get_messages_for_session(&session.id).unwrap();
        assert_eq!(retrieved.len(), 5);

        for (i, msg) in retrieved.iter().enumerate() {
            assert_eq!(msg.sequence_number, i as u64);
        }
    }

    #[test]
    fn test_delete_messages_for_session() {
        let (_f, engine) = open_tmp();

        let s1 = engine.create_session("peer1", 0).unwrap();
        let s2 = engine.create_session("peer2", 0).unwrap();

        for seq in 0..3u64 {
            engine
                .store_message(&StoredMessage {
                    session_id: s1.id.clone(),
                    sender_pubkey_hex: "peer1".to_string(),
                    ciphertext: vec![0u8; 4],
                    timestamp: 0,
                    direction: MessageDirection::Inbound,
                    sequence_number: seq,
                })
                .unwrap();
            engine
                .store_message(&StoredMessage {
                    session_id: s2.id.clone(),
                    sender_pubkey_hex: "peer2".to_string(),
                    ciphertext: vec![0u8; 4],
                    timestamp: 0,
                    direction: MessageDirection::Outbound,
                    sequence_number: seq,
                })
                .unwrap();
        }

        let deleted = engine.delete_messages_for_session(&s1.id).unwrap();
        assert_eq!(deleted, 3);

        assert!(engine.get_messages_for_session(&s1.id).unwrap().is_empty());
        assert_eq!(engine.get_messages_for_session(&s2.id).unwrap().len(), 3);
    }

    #[test]
    fn test_messages_isolated_by_session() {
        let (_f, engine) = open_tmp();

        let s1 = engine.create_session("alice", 0).unwrap();
        let s2 = engine.create_session("bob", 0).unwrap();

        engine
            .store_message(&StoredMessage {
                session_id: s1.id.clone(),
                sender_pubkey_hex: "alice".to_string(),
                ciphertext: vec![1u8; 4],
                timestamp: 0,
                direction: MessageDirection::Outbound,
                sequence_number: 0,
            })
            .unwrap();

        engine
            .store_message(&StoredMessage {
                session_id: s2.id.clone(),
                sender_pubkey_hex: "bob".to_string(),
                ciphertext: vec![2u8; 4],
                timestamp: 0,
                direction: MessageDirection::Inbound,
                sequence_number: 0,
            })
            .unwrap();

        let s1_msgs = engine.get_messages_for_session(&s1.id).unwrap();
        let s2_msgs = engine.get_messages_for_session(&s2.id).unwrap();

        assert_eq!(s1_msgs.len(), 1);
        assert_eq!(s2_msgs.len(), 1);
        assert_eq!(s1_msgs[0].ciphertext, vec![1u8; 4]);
        assert_eq!(s2_msgs[0].ciphertext, vec![2u8; 4]);
    }
}
