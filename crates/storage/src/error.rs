use thiserror::Error;

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
