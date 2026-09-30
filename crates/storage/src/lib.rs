#![allow(clippy::result_large_err)]

pub mod engine;
pub mod error;
pub mod models;
pub mod search;

pub use engine::StorageEngine;
pub use error::StorageError;
pub use models::{Contact, MessageDirection, Session, StoredMessage};
pub use search::{SearchError, SearchIndex, SearchResult};