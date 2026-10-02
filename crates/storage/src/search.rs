use std::path::Path;

use hmac::{Hmac, Mac};
use tantivy::collector::TopDocs;
use tantivy::directory::error::OpenDirectoryError;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::{
    Field, IndexRecordOption, OwnedValue, Schema, TextFieldIndexing, TextOptions, FAST, STORED,
    STRING,
};
use tantivy::tokenizer::{
    LowerCaser, RemoveLongFilter, SimpleTokenizer, TextAnalyzer, TokenStream, Tokenizer,
};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};
use thiserror::Error;

use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const INDEX_WRITER_MEMORY_BYTES: usize = 50_000_000;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("Open directory error: {0}")]
    OpenDirectory(#[from] OpenDirectoryError),

    #[error("Query parse error: {0}")]
    QueryParse(#[from] tantivy::query::QueryParserError),

    #[error("invalid search key")]
    InvalidSearchKey,
}

#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    search_key: [u8; 32],
    msg_id_field: Field,
    peer_id_field: Field,
    timestamp_field: Field,
    content_field: Field,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub msg_id: String,
    pub peer_id: String,
    pub timestamp: u64,
}

impl SearchIndex {
    pub fn open_or_create<P: AsRef<Path>>(
        path: P,
        search_key: [u8; 32],
    ) -> Result<Self, SearchError> {
        let mut schema_builder = Schema::builder();

        let msg_id_field = schema_builder.add_text_field("msg_id", STRING | STORED);

        let peer_id_field = schema_builder.add_text_field("peer_id", STRING | STORED);

        let timestamp_field = schema_builder.add_u64_field("timestamp", FAST | STORED);

        let content_indexing = TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);

        let content_options = TextOptions::default().set_indexing_options(content_indexing);

        let content_field = schema_builder.add_text_field("content", content_options);

        let schema = schema_builder.build();

        std::fs::create_dir_all(&path).ok();

        let dir = MmapDirectory::open(path)?;
        let index = Index::open_or_create(dir, schema)?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            search_key,
            msg_id_field,
            peer_id_field,
            timestamp_field,
            content_field,
        })
    }

    pub fn index_message(
        &self,
        msg_id: &str,
        peer_id: &str,
        timestamp: u64,
        content: &str,
    ) -> Result<(), SearchError> {
        let mut writer: IndexWriter = self.index.writer(INDEX_WRITER_MEMORY_BYTES)?;

        let hashed_content = self.hash_content(content);

        let mut doc = tantivy::TantivyDocument::default();

        doc.add_text(self.msg_id_field, msg_id);
        doc.add_text(self.peer_id_field, peer_id);
        doc.add_u64(self.timestamp_field, timestamp);
        doc.add_text(self.content_field, hashed_content);

        writer.add_document(doc)?;
        writer.commit()?;

        self.reader.reload()?;

        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        if query_str.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let searcher = self.reader.searcher();

        let hashed_query = self.hash_content(query_str);

        if hashed_query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut query_parser = QueryParser::for_index(&self.index, vec![self.content_field]);

        query_parser.set_conjunction_by_default();

        let query = query_parser.parse_query(&hashed_query)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::with_capacity(top_docs.len());

        for (_score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;

            let extract_str = |value: Option<&OwnedValue>| match value {
                Some(OwnedValue::Str(s)) => s.clone(),
                _ => String::new(),
            };

            let extract_u64 = |value: Option<&OwnedValue>| match value {
                Some(OwnedValue::U64(v)) => *v,
                _ => 0,
            };

            let msg_id = extract_str(retrieved_doc.get_first(self.msg_id_field));

            let peer_id = extract_str(retrieved_doc.get_first(self.peer_id_field));

            let timestamp = extract_u64(retrieved_doc.get_first(self.timestamp_field));

            results.push(SearchResult {
                msg_id,
                peer_id,
                timestamp,
            });
        }

        Ok(results)
    }

    fn hash_content(&self, content: &str) -> String {
        let mut analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .build();

        let mut stream = analyzer.token_stream(content);

        let mut hashed_tokens = Vec::new();

        while stream.advance() {
            let token = stream.token();

            if token.text.is_empty() {
                continue;
            }

            hashed_tokens.push(self.hash_token(&token.text));
        }

        hashed_tokens.join(" ")
    }

    fn hash_token(&self, token: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.search_key)
            .expect("HMAC-SHA256 accepts a 32-byte key");

        mac.update(token.as_bytes());

        let digest = mac.finalize().into_bytes();

        let mut output = String::with_capacity(digest.len() * 2);

        for byte in digest {
            use std::fmt::Write;
            write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_key() -> [u8; 32] {
        [0x42; 32]
    }

    #[test]
    fn test_search_index_basic() {
        let dir = tempdir().unwrap();

        let search_index = SearchIndex::open_or_create(dir.path(), test_key()).unwrap();

        search_index
            .index_message("msg1", "peer_Ventie", 1000, "Hello post quantum world")
            .unwrap();

        search_index
            .index_message("msg2", "peer_bob", 1001, "Secret handshake completed")
            .unwrap();

        let results = search_index.search("quantum", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].msg_id, "msg1");
        assert_eq!(results[0].peer_id, "peer_Ventie");

        let results_handshake = search_index.search("handshake", 10).unwrap();

        assert_eq!(results_handshake.len(), 1);
        assert_eq!(results_handshake[0].msg_id, "msg2");
    }

    #[test]
    fn test_search_is_case_insensitive() {
        let dir = tempdir().unwrap();

        let search_index = SearchIndex::open_or_create(dir.path(), test_key()).unwrap();

        search_index
            .index_message("msg1", "peer_alice", 1000, "Post Quantum Cryptography")
            .unwrap();

        let results = search_index.search("QUANTUM", 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].msg_id, "msg1");
    }

    #[test]
    fn test_search_requires_same_key() {
        let dir = tempdir().unwrap();

        let index = SearchIndex::open_or_create(dir.path(), test_key()).unwrap();

        index
            .index_message("msg1", "peer_alice", 1000, "secret quantum message")
            .unwrap();

        let different_key = [0x99; 32];

        let other_index =
            SearchIndex::open_or_create(dir.path().join("other"), different_key).unwrap();

        let results = other_index.search("quantum", 10).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let dir = tempdir().unwrap();

        let search_index = SearchIndex::open_or_create(dir.path(), test_key()).unwrap();

        search_index
            .index_message("msg1", "peer_alice", 1000, "hello world")
            .unwrap();

        assert!(search_index.search("", 10).unwrap().is_empty());

        assert!(search_index.search("   ", 10).unwrap().is_empty());
    }

    #[test]
    fn test_zero_limit() {
        let dir = tempdir().unwrap();

        let search_index = SearchIndex::open_or_create(dir.path(), test_key()).unwrap();

        search_index
            .index_message("msg1", "peer_alice", 1000, "hello world")
            .unwrap();

        assert!(search_index.search("hello", 0).unwrap().is_empty());
    }
}
