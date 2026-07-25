#![allow(clippy::result_large_err)]

use r_crypto::EncryptedVault;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

const VAULT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("system_vault");
const CONTACTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("contacts");

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use r_crypto::Identity;
    use tempfile::NamedTempFile;

    #[test]
    fn test_vault_storage_cycle() {
        let tmp_file = NamedTempFile::new().unwrap();
        let engine = StorageEngine::open(tmp_file.path()).unwrap();

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
        let tmp_file = NamedTempFile::new().unwrap();
        let engine = StorageEngine::open(tmp_file.path()).unwrap();

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
        let tmp_file = NamedTempFile::new().unwrap();
        let engine = StorageEngine::open(tmp_file.path()).unwrap();

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
}