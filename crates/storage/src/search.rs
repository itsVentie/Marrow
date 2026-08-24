use tantivy::collector::TopDocs;
use tantivy::directory::error::OpenDirectoryError;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::schema::OwnedValue;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("Open directory error: {0}")]
    OpenDirectory(#[from] OpenDirectoryError),
    #[error("Query parse error: {0}")]
    QueryParse(#[from] tantivy::query::QueryParserError),
}

#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
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
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self, SearchError> {
        let mut schema_builder = Schema::builder();
        let msg_id_field = schema_builder.add_text_field("msg_id", STRING | STORED);
        let peer_id_field = schema_builder.add_text_field("peer_id", STRING | STORED);
        let timestamp_field = schema_builder.add_u64_field("timestamp", FAST | STORED);

        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored();

        let content_field = schema_builder.add_text_field("content", text_options);
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
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        let mut doc = TantivyDocument::default();
        doc.add_text(self.msg_id_field, msg_id);
        doc.add_text(self.peer_id_field, peer_id);
        doc.add_u64(self.timestamp_field, timestamp);
        doc.add_text(self.content_field, content);

        writer.add_document(doc)?;
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let searcher = self.reader.searcher();
        let mut query_parser = QueryParser::for_index(&self.index, vec![self.content_field]);
        query_parser.set_conjunction_by_default();

        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;

            let extract_str = |val: Option<&OwnedValue>| match val {
                Some(OwnedValue::Str(s)) => s.clone(),
                _ => String::new(),
            };

            let extract_u64 = |val: Option<&OwnedValue>| match val {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_search_index_basic() {
        let dir = tempdir().unwrap();
        let search_index = SearchIndex::open_or_create(dir.path()).unwrap();

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
}
