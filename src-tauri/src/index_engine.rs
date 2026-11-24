//! Persistent index backed by Tantivy 0.22.
//! Acts as the high-throughput candidate retrieval engine.

use std::path::Path;
use std::sync::Mutex;
use tantivy::{
    schema::*,
    Index, IndexWriter, IndexReader,
    query::{QueryParser, RegexQuery, BooleanQuery, Occur, Query},
    collector::TopDocs,
    ReloadPolicy,
    doc,
    TantivyDocument,
    Term,
};
use crate::indexer::{SearchItem, ItemType};

// ── Schema ─────────────────────────────────────────────────────────────────

pub struct SpotlightSchema {
    pub schema: Schema,
    pub f_name: Field,
    pub f_path: Field,
    pub f_item_type: Field,
    pub f_category: Field,
}

impl SpotlightSchema {
    pub fn build() -> Self {
        let mut sb = Schema::builder();
        let f_name      = sb.add_text_field("name", TEXT | STORED);
        let f_path      = sb.add_text_field("path", STRING | STORED);
        let f_item_type = sb.add_text_field("item_type", STRING | STORED);
        let f_category  = sb.add_text_field("category", STRING | STORED);
        // Icons are not stored: results are materialised from the in-memory
        // cache by path, so storing ~0.5 KB data URIs per document only bloated
        // the index and slowed candidate retrieval.
        let schema = sb.build();
        SpotlightSchema {
            schema, f_name, f_path, f_item_type, f_category
        }
    }
}

// ── Index Engine ─────────────────────────────────────────────────────────────

pub struct IndexEngine {
    #[allow(dead_code)]
    pub index: Index,
    pub schema: SpotlightSchema,
    pub writer: Mutex<IndexWriter>,
    pub reader: IndexReader,
}

impl IndexEngine {
    pub fn open(index_dir: &Path) -> tantivy::Result<Self> {
        std::fs::create_dir_all(index_dir).ok();
        let schema_def = SpotlightSchema::build();

        let try_open = || -> tantivy::Result<(Index, IndexWriter)> {
            if index_dir.join("meta.json").exists() {
                if let Ok(existing) = Index::open_in_dir(index_dir) {
                    if existing.schema() == schema_def.schema {
                        if let Ok(writer) = existing.writer(50_000_000) {
                            return Ok((existing, writer));
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(index_dir);
            let _ = std::fs::create_dir_all(index_dir);
            let index = Index::create_in_dir(index_dir, schema_def.schema.clone())?;
            let writer = index.writer(50_000_000)?;
            Ok((index, writer))
        };

        let (index, writer) = match try_open() {
            Ok(pair) => pair,
            Err(_) => {
                // Self-healing fallback: wipe corrupted directory and recreate cleanly
                let _ = std::fs::remove_dir_all(index_dir);
                let _ = std::fs::create_dir_all(index_dir);
                let fresh_index = Index::create_in_dir(index_dir, schema_def.schema.clone())?;
                let fresh_writer = fresh_index.writer(50_000_000)?;
                (fresh_index, fresh_writer)
            }
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(IndexEngine {
            index,
            schema: schema_def,
            writer: Mutex::new(writer),
            reader,
        })
    }

    fn make_doc(&self, item: &SearchItem) -> TantivyDocument {
        let s = &self.schema;
        let item_type_str = match item.item_type {
            ItemType::App    => "app",
            ItemType::File   => "file",
            ItemType::Folder => "folder",
        };
        doc!(
            s.f_name => item.name.clone(),
            s.f_path => item.path.clone(),
            s.f_item_type => item_type_str,
            s.f_category => item.category.clone(),
        )
    }

    pub fn upsert(&self, item: &SearchItem) -> tantivy::Result<()> {
        let writer = self.writer.lock().unwrap();
        writer.delete_term(Term::from_field_text(self.schema.f_path, &item.path));
        writer.add_document(self.make_doc(item))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn bulk_add(&self, items: &[SearchItem]) -> tantivy::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        for item in items {
            writer.delete_term(Term::from_field_text(self.schema.f_path, &item.path));
            writer.add_document(self.make_doc(item))?;
        }
        writer.commit()?;
        drop(writer);
        self.reader.reload()?;
        Ok(())
    }

    /// Replace the whole index with `items` in one commit, so files deleted or
    /// renamed while the app wasn't running don't linger as stale documents.
    pub fn replace_all(&self, items: &[SearchItem]) -> tantivy::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.delete_all_documents()?;
        for item in items {
            writer.add_document(self.make_doc(item))?;
        }
        writer.commit()?;
        drop(writer);
        self.reader.reload()?;
        Ok(())
    }

    pub fn remove_by_path(&self, path: &str) -> tantivy::Result<()> {
        let writer = self.writer.lock().unwrap();
        let path_term = Term::from_field_text(self.schema.f_path, path);
        writer.delete_term(path_term);
        Ok(())
    }

    pub fn commit(&self) -> tantivy::Result<()> {
        self.writer.lock().unwrap().commit()?;
        self.reader.reload()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn vacuum(&self) {
        let writer = self.writer.lock().unwrap();
        let fut = writer.garbage_collect_files();
        let _ = tauri::async_runtime::block_on(fut);
    }

    /// Retrieve matching candidate file paths and match scores from Tantivy.
    /// Fast indexed query parsing with wildcard support.
    pub fn search_candidates(&self, query_str: &str, limit: usize) -> Vec<(String, i64)> {
        let trimmed = query_str.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let searcher = self.reader.searcher();
        let s = &self.schema;

        // Sanitize punctuation for QueryParser
        let sanitized: String = trimmed
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect();
        let clean_str = sanitized.trim();
        if clean_str.is_empty() {
            return Vec::new();
        }

        let tokens: Vec<&str> = clean_str.split_whitespace().collect();
        if tokens.is_empty() {
            return Vec::new();
        }

        // Build prefix RegexQuery for each token (e.g. "aqua.*")
        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for t in &tokens {
            let escaped = regex::escape(&t.to_lowercase());
            let pattern = format!("{}.*", escaped);
            if let Ok(rq) = RegexQuery::from_pattern(&pattern, s.f_name) {
                subqueries.push((Occur::Must, Box::new(rq)));
            }
        }

        let query: Box<dyn Query> = if !subqueries.is_empty() {
            Box::new(BooleanQuery::new(subqueries))
        } else {
            let mut qp = QueryParser::for_index(&self.index, vec![s.f_name]);
            qp.set_conjunction_by_default();
            match qp.parse_query(clean_str) {
                Ok(q) => q,
                Err(_) => return Vec::new(),
            }
        };

        if let Ok(top) = searcher.search(&query, &TopDocs::with_limit(limit)) {
            return top.into_iter().filter_map(|(score, addr)| {
                searcher.doc::<TantivyDocument>(addr).ok().and_then(|d| {
                    d.get_first(s.f_path)
                        .and_then(|v| v.as_str())
                        .map(|p| (p.to_string(), (score * 1000.0) as i64))
                })
            }).collect();
        }

        Vec::new()
    }
}
