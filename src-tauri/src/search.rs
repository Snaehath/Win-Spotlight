use crate::history::HistoryManager;
use crate::indexer::SearchItem;
use crate::ranking::{compute_score, ScoredItem, rank};
use crate::commands::{CommandRegistry, CommandResult, eval_simple};
use crate::index_engine::IndexEngine;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tauri::State;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

pub struct AppCache {
    pub apps: Mutex<Vec<SearchItem>>,
}

pub struct IndexState(pub Arc<IndexEngine>);
pub struct CommandState(pub CommandRegistry);

/// Result row returned to the UI — extends SearchItem with an optional
/// inline display value (used by command plugins like calc).
#[derive(serde::Serialize, Clone, Debug)]
pub struct SearchResult {
    #[serde(flatten)]
    pub item: SearchItem,
    /// If set, render this inline in the result row instead of the path.
    pub inline_display: Option<String>,
}

impl From<SearchItem> for SearchResult {
    fn from(item: SearchItem) -> Self {
        SearchResult { item, inline_display: None }
    }
}

#[tauri::command]
pub fn search_items(
    query: String,
    state: State<'_, AppCache>,
    history_manager: State<'_, HistoryManager>,
    shortcut_manager: State<'_, crate::shortcuts::ShortcutManager>,
    index_state: State<'_, IndexState>,
    cmd_state: State<'_, CommandState>,
) -> Vec<SearchResult> {
    let query_trimmed = query.trim();

    // ── 1. Explicit command layer (> prefix) — kept for power users ──────────
    if query_trimmed.starts_with('>') {
        return handle_command(query_trimmed, &cmd_state.0);
    }

    // ── 1.1 Handle Keyword Filtering (e.g. app:, file:, folder:) ───────────
    let mut forced_category: Option<&str> = None;
    let mut forced_item_type: Option<crate::indexer::ItemType> = None;
    let mut actual_query = query_trimmed;

    if query_trimmed.starts_with("app:") {
        forced_category = Some("APP");
        actual_query = query_trimmed["app:".len()..].trim();
    } else if query_trimmed.starts_with("file:") {
        forced_item_type = Some(crate::indexer::ItemType::File);
        actual_query = query_trimmed["file:".len()..].trim();
    } else if query_trimmed.starts_with("folder:") {
        forced_item_type = Some(crate::indexer::ItemType::Folder);
        actual_query = query_trimmed["folder:".len()..].trim();
    } else if query_trimmed.starts_with("command:") {
        forced_category = Some("COMMAND");
        actual_query = query_trimmed["command:".len()..].trim();
    }

    let items = state.apps.lock().unwrap();
    let matcher = SkimMatcherV2::default();

    // ── 2. Empty query: show filter suggestions + recents ──────────────────
    // Note: We skip this early return if the `command:` filter is active, 
    // so we can show the full command list even with an empty query.
    if actual_query.is_empty() && forced_category != Some("COMMAND") {
        let history = history_manager.load();
        let mut final_results: Vec<SearchResult> = Vec::new();

        // ── 2a. Inject Filter Suggestions ────────────────────────────────────
        // Only show these if no specific filter is already active
        if forced_category.is_none() && forced_item_type.is_none() {
            let suggestions = vec![
                ("app:", "Search Apps", "monitor"),
                ("file:", "Search Files", "file-text"),
                ("folder:", "Search Folders", "folder"),
                ("command:", "Search Commands", "terminal"),
            ];

            for (prefix, label, icon) in suggestions {
                final_results.push(SearchResult::from(SearchItem {
                    name: label.to_string(),
                    path: prefix.to_string(),
                    icon: Some(icon.to_string()),
                    item_type: crate::indexer::ItemType::File,
                    category: "FILTER".to_string(), // New category for special styling
                }));
            }
        }

        // ── 2b. Add Recents ──────────────────────────────────────────────────
        let mut recents_count = 0;
        for record in history.records.iter() {
            if recents_count >= 5 { break; }

            let item = if record.path.starts_with("COMMAND:") {
                // Commands are generally not filtered by keywords
                if forced_category.is_some() || forced_item_type.is_some() { continue; }

                // Synthetic command item
                let name = if record.path.contains("> health") { "System Health" } 
                          else if record.path.contains("> sys") { "System Action" }
                          else { "Recent Action" };
                Some(crate::indexer::SearchItem {
                    name: format!("⚡ {}", name),
                    path: record.path.clone(),
                    icon: None,
                    item_type: crate::indexer::ItemType::File,
                    category: "RECENT".to_string(),
                })
            } else {
                items.iter().find(|i| i.path == record.path).and_then(|cached| {
                    // Apply filtering to recents too!
                    if let Some(cat) = forced_category {
                        if cached.category != cat { return None; }
                    }
                    if let Some(ref itype) = forced_item_type {
                        if cached.item_type != *itype { return None; }
                    }
                    
                    let mut recent_item = cached.clone();
                    recent_item.category = "RECENT".to_string();
                    Some(recent_item)
                })
            };

            if let Some(res_item) = item {
                final_results.push(SearchResult::from(res_item));
                recents_count += 1;
            }
        }

        return final_results;
    }

    // ── 3. Fuzzy search via in-memory cache ─────────────────────────────────
    let mut scored: Vec<(i64, SearchItem)> = items
        .iter()
        .filter(|item| {
            if let Some(cat) = forced_category {
                if item.category != cat { return false; }
            }
            if let Some(ref itype) = forced_item_type {
                if item.item_type != *itype { return false; }
            }
            true
        })
        .filter_map(|item| {
            let fuzzy = matcher.fuzzy_match(&item.name, actual_query);
            let acronym = acronym_match(&item.name, actual_query);
            match (fuzzy, acronym) {
                (Some(f), Some(a)) => Some((f.max(a), item.clone())),
                (Some(f), None)    => Some((f, item.clone())),
                (None, Some(a))    => Some((a, item.clone())),
                (None, None)       => None,
            }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let base_results: Vec<(SearchItem, i64)> = scored.into_iter().map(|(s, i)| (i, s)).collect();

    // ── 3. Apply ranking ────────────────────────────────────────────────────
    let engine = &index_state.0;
    let scored_items: Vec<ScoredItem> = base_results
        .into_iter()
        .map(|(item, fuzzy)| {
            let (count, last_ts) = engine.get_stats(&item.path);
            let depth = std::path::Path::new(&item.path).components().count();
            let is_app = item.category == "APP";
            let time_score = history_manager.get_time_score(&item.path);
            let score = compute_score(fuzzy, count, last_ts, time_score, depth, is_app);
            ScoredItem::new(item, score)
        })
        .collect();

    let ranked = rank(scored_items)
        .into_iter()
        .map(SearchResult::from)
        .collect::<Vec<_>>();

    // ── 4. Inject Recent items at the top ────────────────────────────────────
    let history = history_manager.load();
    let mut recents: Vec<SearchResult> = Vec::new();

    for record in history.records {
        if recents.len() >= 5 { break; }

        let item = if record.path.starts_with("COMMAND:") {
            // Synthetic command item
            let name = if record.path.contains("> health") { "System Health" } 
                      else if record.path.contains("> sys") { "System Action" }
                      else { "Recent Action" };
            Some(crate::indexer::SearchItem {
                name: format!("⚡ {}", name),
                path: record.path.clone(),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "RECENT".to_string(),
            })
        } else {
            items.iter().find(|i| i.path == record.path).map(|cached| {
                let mut recent_item = cached.clone();
                recent_item.category = "RECENT".to_string();
                recent_item
            })
        };

        if let Some(res_item) = item {
            let match_ok = query_trimmed.is_empty()
                || matcher.fuzzy_match(&res_item.name, query_trimmed).is_some();

            if match_ok {
                recents.push(SearchResult::from(res_item));
            }
        }
    }
    drop(items); // release lock early

    // ── 5. Merge: Recents → Ranked, deduplicated ─────────────────────────────
    let mut final_results: Vec<SearchResult> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    for r in recents {
        if seen_paths.insert(r.item.path.clone()) {
            final_results.push(r);
        }
    }
    for r in ranked {
        if seen_paths.insert(r.item.path.clone()) {
            final_results.push(r);
        }
    }

    let mut file_results: Vec<SearchResult> = final_results.into_iter().take(40).collect();

    // ── 6. Ambient Intent Layer ──────────────────────────────────────────────
    // We only show these if:
    // 1. No specific filter is active (Universal Search)
    // 2. The `command:` filter is explicitly active
    let is_command_filter = forced_category == Some("COMMAND");
    let has_other_filter = (forced_category.is_some() && !is_command_filter) || forced_item_type.is_some();

    let mut command_results: Vec<SearchResult> = if has_other_filter {
        Vec::new()
    } else {
        detect_ambient_intent(actual_query, &shortcut_manager, is_command_filter, &matcher)
    };

    // Combine: command suggestions first (pinned at top of COMMAND section)
    command_results.append(&mut file_results);
    command_results
}

// ── Ambient Intent Detection ──────────────────────────────────────────────────
// Detects math and system keywords without requiring a `>` prefix.

fn detect_ambient_intent(
    query: &str, 
    shortcut_manager: &crate::shortcuts::ShortcutManager,
    force_all: bool,
    matcher: &SkimMatcherV2
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let q = query.trim().to_lowercase();
    let is_empty = q.is_empty();

    // ── Math: detect `<number> <op> <number>` patterns ───────────────────────
    if !is_empty && is_math_expression(query) {
        if let Some(result) = eval_simple(query) {
            let formatted = if result.fract() == 0.0 {
                format!("{}", result as i64)
            } else {
                format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            };
            let display = format!("{} = {}", query.trim(), formatted);
            let synthetic = SearchItem {
                name: display.clone(),
                path: String::new(),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "COMMAND".to_string(),
            };
            results.push(SearchResult { item: synthetic, inline_display: Some(display) });
        }
    }

    // ── System actions: detect power/lock keywords ────────────────────────────
    let sys_actions: &[(&str, &str, &str, &str)] = &[
        ("shutdown",  "Shut Down PC",   "COMMAND:> sys shutdown", "power"),
        ("shut down", "Shut Down PC",   "COMMAND:> sys shutdown", "power"),
        ("restart",   "Restart PC",     "COMMAND:> sys restart",  "refresh-cw"),
        ("reboot",    "Restart PC",     "COMMAND:> sys restart",  "refresh-cw"),
        ("sleep",     "Sleep PC",       "COMMAND:> sys sleep",    "moon"),
        ("hibernate", "Sleep PC",       "COMMAND:> sys sleep",    "moon"),
        ("lock",      "Lock Screen",    "COMMAND:> sys lock",     "lock"),
        ("lock screen","Lock Screen",   "COMMAND:> sys lock",     "lock"),
        ("exit",      "Exit Spotlight", "COMMAND:> sys exit",     "log-out"),
        ("quit",      "Exit Spotlight", "COMMAND:> sys exit",     "log-out"),
    ];

    for (keyword, label, cmd_path, icon) in sys_actions {
        let is_match = if force_all && is_empty { 
            true 
        } else { 
            q == *keyword || q.starts_with(keyword) || matcher.fuzzy_match(keyword, &q).is_some()
        };
        
        if is_match {
            // Avoid duplicate matches (e.g. "shutdown" and "shut down")
            let already_added = results.iter().any(|r: &SearchResult| r.item.path == *cmd_path);
            if !already_added {
                let synthetic = SearchItem {
                    name: label.to_string(),
                    path: cmd_path.to_string(),
                    icon: Some(icon.to_string()),
                    item_type: crate::indexer::ItemType::File,
                    category: "COMMAND".to_string(),
                };
                results.push(SearchResult::from(synthetic));
            }
        }
    }

    // ── Custom Web Shortcuts ──────────────────────────────────────────────────
    let shortcuts = shortcut_manager.get_all();
    for (alias, url) in shortcuts {
        let is_match = if force_all && is_empty { 
            true 
        } else { 
            alias.starts_with(&q) || q.starts_with(&alias) || matcher.fuzzy_match(&alias, &q).is_some()
        };
        
        if is_match {
            let synthetic = SearchItem {
                name: alias.clone(),
                path: format!("COMMAND:{}", url),
                icon: Some("link-2".to_string()),
                item_type: crate::indexer::ItemType::File,
                category: "WEB SHORTCUT".to_string(),
            };
            results.push(SearchResult::from(synthetic));
        }
    }

    // ── URL Detection & "Save Shortcut" ────────────────────────────────────────
    if !is_empty {
        let common_tlds = [".com", ".org", ".net", ".io", ".gov", ".edu", ".me", ".app", ".dev", ".ai"];
        let has_web_tld = common_tlds.iter().any(|tld| q.ends_with(tld));
        let is_web_prefix = q.starts_with("www.") || q.starts_with("http");
        
        let is_url = is_web_prefix || (has_web_tld && !q.contains(' '));

        if is_url {
            // Option 1: Open
            let open_path = if q.starts_with("http") { q.to_string() } else { format!("https://{}", q) };
            results.push(SearchResult::from(SearchItem {
                name: format!("Open {}", q),
                path: format!("COMMAND:{}", open_path),
                icon: Some("globe".to_string()),
                item_type: crate::indexer::ItemType::File,
                category: "WEB".to_string(),
            }));

            // Option 2: Save
            results.push(SearchResult::from(SearchItem {
                name: format!("Save {} as shortcut...", q),
                path: format!("CREATE_SHORTCUT:{}", open_path),
                icon: Some("bookmark".to_string()),
                item_type: crate::indexer::ItemType::File,
                category: "WEB".to_string(),
            }));
        }
    }

    // ── Management: Clear Shortcuts ──────────────────────────────────────────
    let is_clear_match = if force_all && is_empty { true } else { q == "clear shortcuts" || q == "> clear shortcuts" };
    if is_clear_match {
        results.push(SearchResult::from(SearchItem {
            name: "Wipe all saved shortcuts".to_string(),
            path: "CLEAR_SHORTCUTS".to_string(),
            icon: Some("trash-2".to_string()),
            item_type: crate::indexer::ItemType::File,
            category: "COMMAND".to_string(),
        }));
    }

    // ── Currency Conversion ───────────────────────────────────────────────────
    results.append(&mut crate::currency::detect_currency_intent(query));

    results
}

/// Returns true if the query looks like a math expression.
/// Supports: `5 + 5`, `100/4`, `12 * 3.5`, `100 - 20`
fn is_math_expression(query: &str) -> bool {
    let s = query.replace(' ', "");
    // Must contain an operator and start with a digit or minus
    let has_op = s.contains('+') || s.contains('*') || s.contains('/') ||
        (s.contains('-') && s.find('-').map_or(false, |i| i > 0));
    if !has_op { return false; }
    // Must be mostly numeric
    s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '+' || c == '-' || c == '*' || c == '/')
}

// ── Explicit Command handler (> prefix) ──────────────────────────────────────

fn handle_command(query: &str, registry: &CommandRegistry) -> Vec<SearchResult> {
    match registry.handle(query) {
        Some(CommandResult::Display(text)) => {
            let synthetic = SearchItem {
                name: text.clone(),
                path: String::new(),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "COMMAND".to_string(),
            };
            vec![SearchResult { item: synthetic, inline_display: Some(text) }]
        }
        Some(CommandResult::Launch(_, _)) | Some(CommandResult::Silent) => {
            let synthetic = SearchItem {
                name: format!("Run: > {}", query.trim_start_matches('>')),
                path: format!("COMMAND:{}", query),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "COMMAND".to_string(),
            };
            vec![SearchResult::from(synthetic)]
        }
        Some(CommandResult::Error(err)) => {
            let synthetic = SearchItem {
                name: err.clone(),
                path: String::new(),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "COMMAND".to_string(),
            };
            vec![SearchResult { item: synthetic, inline_display: Some(err) }]
        }
        None => {
            let hints = registry.all_hints();
            hints.into_iter().map(|(prefix, desc)| {
                let synthetic = SearchItem {
                    name: format!("> {}  — {}", prefix, desc),
                    path: String::new(),
                    icon: None,
                    item_type: crate::indexer::ItemType::File,
                    category: "COMMAND".to_string(),
                };
                SearchResult { item: synthetic, inline_display: None }
            }).collect()
        }
    }
}

// ── Acronym Matching ─────────────────────────────────────────────────────────

/// Returns a score when `query` matches the initials of words in `name`.
/// e.g. "np" matches "Notepad" (N·o·t·e·p·a·d), "wt" matches "Windows Terminal"
fn acronym_match(name: &str, query: &str) -> Option<i64> {
    let initials: String = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_lowercase();

    let q = query.to_lowercase();
    if initials.starts_with(&q) {
        Some(200 + (q.len() as i64 * 10))
    } else {
        None
    }
}
