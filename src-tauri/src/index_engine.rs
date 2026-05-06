/// Persistent index backed by Tantivy 0.22.

use std::path::Path;
use std::sync::Mutex;
use tantivy::{
    schema::*,
    Index, IndexWriter, IndexReader,
    query::{QueryParser, TermQuery},
    collector::TopDocs,
    ReloadPolicy,
    doc,
    TantivyDocument,
    Term,
    schema::IndexRecordOption,
};
use crate::indexer::{SearchItem, ItemType};

// ── Schema ─────────────────────────────────────────────────────────────────

pub struct SpotlightSchema {
    pub schema: Schema,
    pub f_name: Field,
    pub f_path: Field,
    pub f_item_type: Field,
    pub f_category: Field,
    pub f_icon: Field,
    pub f_launch_count: Field,
    pub f_last_launched: Field,
}

impl SpotlightSchema {
    pub fn build() -> Self {
        let mut sb = Schema::builder();
        let f_name          = sb.add_text_field("name", TEXT | STORED);
        let f_path          = sb.add_text_field("path", STRING | STORED);
        let f_item_type     = sb.add_text_field("item_type", STRING | STORED);
        let f_category      = sb.add_text_field("category", STRING | STORED);
        let f_icon          = sb.add_text_field("icon", STORED);
        let f_launch_count  = sb.add_u64_field("launch_count", FAST | STORED);
        let f_last_launched = sb.add_u64_field("last_launched", FAST | STORED);
        let schema = sb.build();
        SpotlightSchema {
            schema, f_name, f_path, f_item_type, f_category,
            f_icon, f_launch_count, f_last_launched
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

        let index = if index_dir.join("meta.json").exists() {
            let existing = Index::open_in_dir(index_dir)?;
            if existing.schema() != schema_def.schema {
                // Schema mismatch (version upgrade) — safely wipe and rebuild
                let _ = std::fs::remove_dir_all(index_dir);
                let _ = std::fs::create_dir_all(index_dir);
                Index::create_in_dir(index_dir, schema_def.schema.clone())?
            } else {
                existing
            }
        } else {
            Index::create_in_dir(index_dir, schema_def.schema.clone())?
        };

        let writer = index.writer(50_000_000)?;
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

    pub fn upsert(&self, item: &SearchItem) -> tantivy::Result<()> {
        let s = &self.schema;
        let writer = self.writer.lock().unwrap();

        let path_term = Term::from_field_text(s.f_path, &item.path);
        writer.delete_term(path_term);

        let item_type_str = match item.item_type {
            ItemType::App    => "app",
            ItemType::File   => "file",
            ItemType::Folder => "folder",
        };
        let icon_val = item.icon.clone().unwrap_or_default();

        let new_doc = doc!(
            s.f_name => item.name.clone(),
            s.f_path => item.path.clone(),
            s.f_item_type => item_type_str,
            s.f_category => item.category.clone(),
            s.f_icon => icon_val,
            s.f_launch_count => 0u64,
            s.f_last_launched => 0u64,
        );
        writer.add_document(new_doc)?;
        Ok(())
    }

    pub fn bulk_add(&self, items: &[SearchItem]) -> tantivy::Result<()> {
        for item in items {
            self.upsert(item)?;
        }
        self.commit()
    }

    pub fn remove_by_path(&self, path: &str) -> tantivy::Result<()> {
        let writer = self.writer.lock().unwrap();
        let path_term = Term::from_field_text(self.schema.f_path, path);
        writer.delete_term(path_term);
        Ok(())
    }

    pub fn commit(&self) -> tantivy::Result<()> {
        self.writer.lock().unwrap().commit()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn vacuum(&self) {
        let writer = self.writer.lock().unwrap();
        let fut = writer.garbage_collect_files();
        let _ = tauri::async_runtime::block_on(fut);
    }

    pub fn record_launch(&self, path: &str, items_cache: &[SearchItem]) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if let Some(item) = items_cache.iter().find(|i| i.path == path) {
            let (old_count, _) = self.get_stats(path);
            let s = &self.schema;
            let mut writer = self.writer.lock().unwrap();

            let path_term = Term::from_field_text(s.f_path, path);
            writer.delete_term(path_term);

            let item_type_str = match item.item_type {
                ItemType::App    => "app",
                ItemType::File   => "file",
                ItemType::Folder => "folder",
            };
            let icon_val = item.icon.clone().unwrap_or_default();

            let new_doc = doc!(
                s.f_name => item.name.clone(),
                s.f_path => item.path.clone(),
                s.f_item_type => item_type_str,
                s.f_category => item.category.clone(),
                s.f_icon => icon_val,
                s.f_launch_count => old_count + 1,
                s.f_last_launched => now,
            );
            let _ = writer.add_document(new_doc);
            let _ = writer.commit();
        }
    }

    pub fn get_stats(&self, path: &str) -> (u64, u64) {
        let searcher = self.reader.searcher();
        let s = &self.schema;
        let term = Term::from_field_text(s.f_path, path);
        let query = TermQuery::new(term, IndexRecordOption::Basic);

        if let Ok(top) = searcher.search(&query, &TopDocs::with_limit(1)) {
            if let Some((_, addr)) = top.first() {
                if let Ok(doc) = searcher.doc::<TantivyDocument>(*addr) {
                    let count = doc.get_first(s.f_launch_count)
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let last = doc.get_first(s.f_last_launched)
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    return (count, last);
                }
            }
        }
        (0, 0)
    }

    #[allow(dead_code)]
    pub fn search(&self, query_str: &str, limit: usize) -> Vec<(SearchItem, i64)> {
        if query_str.is_empty() {
            return self.top_by_launch_count(limit);
        }

        let searcher = self.reader.searcher();
        let s = &self.schema;
        let mut qp = QueryParser::for_index(&self.index, vec![s.f_name]);
        qp.set_conjunction_by_default();

        let tantivy_query = match qp.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return self.prefix_search(query_str, limit),
        };

        match searcher.search(&tantivy_query, &TopDocs::with_limit(limit)) {
            Ok(top) => top.into_iter().filter_map(|(score, addr)| {
                searcher.doc::<TantivyDocument>(addr).ok()
                    .map(|d| (doc_to_item(&d, s), (score * 1000.0) as i64))
            }).collect(),
            Err(_) => self.prefix_search(query_str, limit),
        }
    }

    #[allow(dead_code)]
    fn prefix_search(&self, query: &str, limit: usize) -> Vec<(SearchItem, i64)> {
        let q = query.to_lowercase();
        let searcher = self.reader.searcher();
        let s = &self.schema;
        let mut results = Vec::new();

        for seg_reader in searcher.segment_readers() {
            let store = seg_reader.get_store_reader(128).ok();
            if let Some(store) = store {
                for doc_id in 0..seg_reader.num_docs() {
                    if let Ok(doc) = store.get::<TantivyDocument>(doc_id) {
                        let name = doc.get_first(s.f_name)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default();
                        if name.contains(&q) {
                            let score = if name.starts_with(&q) { 500 } else { 200 };
                            results.push((doc_to_item(&doc, s), score));
                            if results.len() >= limit { break; }
                        }
                    }
                }
                if results.len() >= limit { break; }
            }
        }
        results
    }

    #[allow(dead_code)]
    fn top_by_launch_count(&self, limit: usize) -> Vec<(SearchItem, i64)> {
        let searcher = self.reader.searcher();
        let s = &self.schema;
        let query = tantivy::query::AllQuery;
        
        // Use field name for order_by_u64_field in 0.22 if Field object doesn't work
        let field_name = s.schema.get_field_name(s.f_launch_count);
        let collector = TopDocs::with_limit(limit)
            .order_by_u64_field(field_name, tantivy::Order::Desc);

        match searcher.search(&query, &collector) {
            Ok(top) => top.into_iter().filter_map(|(count, addr)| {
                searcher.doc::<TantivyDocument>(addr).ok().map(|d| {
                    (doc_to_item(&d, s), count as i64)
                })
            }).collect(),
            Err(_) => vec![],
        }
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn doc_to_item(doc: &TantivyDocument, s: &SpotlightSchema) -> SearchItem {
    let get_str = |f: Field| -> String {
        doc.get_first(f)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_default()
    };

    let name          = get_str(s.f_name);
    let path          = get_str(s.f_path);
    let category      = get_str(s.f_category);
    let icon_str      = get_str(s.f_icon);
    let item_type_str = get_str(s.f_item_type);

    let item_type = match item_type_str.as_str() {
        "app"    => ItemType::App,
        "folder" => ItemType::Folder,
        _        => ItemType::File,
    };

    let icon = if icon_str.is_empty() { None } else { Some(icon_str) };
    SearchItem { name, path, icon, item_type, category }
}
