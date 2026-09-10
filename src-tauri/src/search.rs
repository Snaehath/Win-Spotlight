use crate::history::HistoryManager;
use crate::indexer::SearchItem;
use crate::ranking::{compute_score, ScoredIndex};
use crate::commands::{CommandRegistry, CommandResult, eval_expression};
use crate::index_engine::IndexEngine;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use tauri::State;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

pub struct AppCache {
    pub apps: Arc<Mutex<Vec<SearchItem>>>,
    pub path_lookup: Arc<Mutex<HashMap<String, usize>>>,
    pub app_indices: Arc<Mutex<Vec<usize>>>,
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

// search
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

    if let Some(stripped) = query_trimmed.strip_prefix("app:") {
        forced_category = Some("APP");
        actual_query = stripped.trim();
    } else if let Some(stripped) = query_trimmed.strip_prefix("file:") {
        forced_item_type = Some(crate::indexer::ItemType::File);
        actual_query = stripped.trim();
    } else if let Some(stripped) = query_trimmed.strip_prefix("folder:") {
        forced_item_type = Some(crate::indexer::ItemType::Folder);
        actual_query = stripped.trim();
    } else if let Some(stripped) = query_trimmed.strip_prefix("command:") {
        forced_category = Some("COMMAND");
        actual_query = stripped.trim();
    }

    let items = state.apps.lock().unwrap();
    let path_lookup = state.path_lookup.lock().unwrap();

    // ── 2. Empty query: show recents ───────────────────────────────────────
    if actual_query.is_empty() && forced_category != Some("COMMAND") {
        return build_recents(
            &history_manager,
            &items,
            &path_lookup,
            forced_category,
            forced_item_type.as_ref(),
            None,
            8,
        );
    }

    let matcher = SkimMatcherV2::default();
    let query_lower = actual_query.to_lowercase();
    let now = crate::ranking::current_timestamp_secs();

    // ── 3. Stage 1: Candidate Retrieval (Tantivy Inverted Index + In-Memory Guarantee) ─
    let mut candidates: Vec<(i64, usize)> = Vec::with_capacity(64);
    let mut candidate_seen: HashSet<usize> = HashSet::with_capacity(64);

    // 3.1 Retrieve candidates from Tantivy inverted index
    let tantivy_candidates = index_state.0.search_candidates(actual_query, 64);
    for (cand_path, tantivy_score) in tantivy_candidates {
        if let Some(&idx) = path_lookup.get(&cand_path) {
            let item = &items[idx];
            if let Some(cat) = forced_category {
                if item.category != cat { continue; }
            }
            if let Some(ref itype) = forced_item_type {
                if item.item_type != *itype { continue; }
            }

            let is_exact = item.normalized_name == query_lower;
            let is_prefix = item.normalized_name.starts_with(&query_lower);
            let mut score = if is_exact {
                1200
            } else if is_prefix {
                700
            } else {
                tantivy_score
            };
            if item.category == "APP" {
                score += 150;
            }
            if candidate_seen.insert(idx) {
                candidates.push((score, idx));
            }
        }
    }

    // 3.2 In-Memory Application Scan: Bounded strictly to installed applications (~100-300 items)
    // Guarantees all installed apps are instantly found even if uncommitted in Tantivy
    let app_indices = state.app_indices.lock().unwrap();
    for &idx in app_indices.iter() {
        if candidate_seen.contains(&idx) {
            continue;
        }
        let item = &items[idx];
        if let Some(cat) = forced_category {
            if item.category != cat { continue; }
        }
        if let Some(ref itype) = forced_item_type {
            if item.item_type != *itype { continue; }
        }

        let is_exact = item.normalized_name == query_lower;
        let is_prefix = item.normalized_name.starts_with(&query_lower);
        let is_acronym = !item.acronym.is_empty() && item.acronym.starts_with(&query_lower);

        let base_score = if is_exact {
            Some(1200)
        } else if is_prefix {
            Some(700)
        } else if is_acronym {
            Some(500)
        } else {
            matcher.fuzzy_match(&item.name, actual_query)
        };

        if let Some(mut s) = base_score {
            s += 150; // APP category priority
            candidate_seen.insert(idx);
            candidates.push((s, idx));
        }
    }

    // ── 4. Stage 2: Bounded Top-K Selection ─────────────────────────────────
    const MAX_CANDIDATES: usize = 48;
    if candidates.len() > MAX_CANDIDATES {
        candidates.select_nth_unstable_by(MAX_CANDIDATES, |a, b| b.0.cmp(&a.0));
        candidates.truncate(MAX_CANDIDATES);
    }

    // ── 5. Stage 3: Personal Composite Ranking ──────────────────────────────
    let mut scored_indices: Vec<ScoredIndex> = candidates
        .into_iter()
        .map(|(match_score, idx)| {
            let item = &items[idx];
            let (count, last_ts) = history_manager.get_launch_stats(&item.path);
            let is_app = item.category == "APP";
            let time_score = history_manager.get_time_score(&item.path);
            let score = compute_score(match_score, count, last_ts, time_score, is_app, now);
            ScoredIndex::new(idx, score)
        })
        .collect();

    scored_indices.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Materialize only the top 10 items
    const TOP_K: usize = 10;
    let ranked: Vec<SearchResult> = scored_indices
        .into_iter()
        .take(TOP_K)
        .map(|s| SearchResult::from(items[s.index].clone()))
        .collect();

    // ── 6. Inject matching Recent items at the top ──────────────────────────
    let recents = build_recents(
        &history_manager,
        &items,
        &path_lookup,
        forced_category,
        forced_item_type.as_ref(),
        Some((&matcher, query_trimmed)),
        4,
    );
    drop(path_lookup);
    drop(items); // release locks early

    // ── 7. Merge: Recents → Ranked, deduplicated ─────────────────────────────
    let mut final_results: Vec<SearchResult> = Vec::with_capacity(TOP_K);
    let mut seen_paths: HashSet<String> = HashSet::with_capacity(TOP_K * 2);

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

    let mut file_results: Vec<SearchResult> = final_results.into_iter().take(TOP_K).collect();

    // ── 8. Ambient Intent Layer ──────────────────────────────────────────────
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

    // ── Math: detect math expression patterns ───────────────────────────────
    if !is_empty && is_math_expression(query) {
        if let Some(result) = eval_expression(query) {
            let formatted = if result.fract() == 0.0 {
                format!("{}", result as i64)
            } else {
                format!("{:.6}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            };
            let display = format!("{} = {}", query.trim(), formatted);
            let synthetic = SearchItem::synthetic(display.clone(), "", "COMMAND");
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
            // Exact match or deliberate prefix (>= 4 chars). Never loosely fuzzy-match power commands!
            q == *keyword || (q.len() >= 4 && keyword.starts_with(&q))
        };
        
        if is_match {
            // Avoid duplicate matches (e.g. "shutdown" and "shut down")
            let already_added = results.iter().any(|r: &SearchResult| r.item.path == *cmd_path);
            if !already_added {
                let synthetic = SearchItem::new(
                    label.to_string(),
                    cmd_path.to_string(),
                    Some(icon.to_string()),
                    crate::indexer::ItemType::File,
                    "COMMAND".to_string(),
                );
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
            let synthetic = SearchItem::new(
                alias.clone(),
                format!("COMMAND:{}", url),
                Some("link-2".to_string()),
                crate::indexer::ItemType::File,
                "WEB SHORTCUT".to_string(),
            );
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
            let open_path = if q.starts_with("http") { q.to_string() } else { format!("https://{}", q) };
            results.push(SearchResult::from(SearchItem::new(
                format!("Open {}", q),
                format!("COMMAND:{}", open_path),
                Some("globe".to_string()),
                crate::indexer::ItemType::File,
                "WEB".to_string(),
            )));
        }
    }

    // ── Management: Clear Shortcuts ──────────────────────────────────────────
    let is_clear_match = if force_all && is_empty { true } else { q == "clear shortcuts" || q == "> clear shortcuts" };
    if is_clear_match {
        results.push(SearchResult::from(SearchItem::new(
            "Wipe all saved shortcuts".to_string(),
            "CLEAR_SHORTCUTS".to_string(),
            Some("trash-2".to_string()),
            crate::indexer::ItemType::File,
            "COMMAND".to_string(),
        )));
    }

    // ── Currency Conversion ───────────────────────────────────────────────────
    results.append(&mut crate::currency::detect_currency_intent(query));

    // ── System HUD Intent ─────────────────────────────────────────────────────
    if q == "sys:" || q == "system" || q == "sys" {
        results.append(&mut crate::system_info::get_system_stats());
    }

    results
}

/// Returns true if the query looks like a math expression.
/// Supports arithmetic, parentheses, power (^), and functions like sqrt, sin, cos, tan, abs, ln, log.
fn is_math_expression(query: &str) -> bool {
    let s = query.replace(' ', "").to_lowercase();
    let has_op = s.contains('+') || s.contains('*') || s.contains('/') || s.contains('^') ||
        s.contains("sqrt") || s.contains("sin") || s.contains("cos") || s.contains("tan") ||
        s.contains("abs") || s.contains("ln") || s.contains("log") ||
        (s.contains('-') && s.find('-').is_some_and(|i| i > 0));
    if !has_op { return false; }
    s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '+' || c == '-' || c == '*' || c == '/' || c == '^' || c == '(' || c == ')' || c.is_ascii_lowercase())
}

// ── Explicit Command handler (> prefix) ──────────────────────────────────────

fn handle_command(query: &str, registry: &CommandRegistry) -> Vec<SearchResult> {
    match registry.handle(query) {
        Some(CommandResult::Display(text)) => {
            let synthetic = SearchItem::synthetic(text.clone(), "", "COMMAND");
            vec![SearchResult { item: synthetic, inline_display: Some(text) }]
        }
        Some(CommandResult::Launch(_, _)) | Some(CommandResult::Silent) => {
            let synthetic = SearchItem::synthetic(
                format!("Run: > {}", query.trim_start_matches('>')),
                format!("COMMAND:{}", query),
                "COMMAND",
            );
            vec![SearchResult::from(synthetic)]
        }
        Some(CommandResult::Error(err)) => {
            let synthetic = SearchItem::synthetic(err.clone(), "", "COMMAND");
            vec![SearchResult { item: synthetic, inline_display: Some(err) }]
        }
        None => {
            let hints = registry.all_hints();
            hints.into_iter().map(|(prefix, desc)| {
                let synthetic = SearchItem::synthetic(
                    format!("> {}  — {}", prefix, desc),
                    "",
                    "COMMAND",
                );
                SearchResult { item: synthetic, inline_display: None }
            }).collect()
        }
    }
}

// ── Recents Helper ──────────────────────────────────────────────────────────

fn build_recents(
    history_manager: &HistoryManager,
    items: &[SearchItem],
    path_lookup: &HashMap<String, usize>,
    forced_category: Option<&str>,
    forced_item_type: Option<&crate::indexer::ItemType>,
    matcher: Option<(&SkimMatcherV2, &str)>,
    limit: usize,
) -> Vec<SearchResult> {
    let history = history_manager.load();
    let mut recents = Vec::new();

    for record in history.records.iter() {
        if recents.len() >= limit {
            break;
        }

        let item = if record.path.starts_with("COMMAND:") {
            if forced_category.is_some() || forced_item_type.is_some() {
                continue;
            }

            let name = if record.path.contains("> health") {
                "System Health"
            } else if record.path.contains("> sys") {
                "System Action"
            } else {
                "Recent Action"
            };

            Some(SearchItem {
                name: format!("⚡ {}", name),
                normalized_name: format!("⚡ {}", name).to_lowercase(),
                acronym: String::new(),
                path: record.path.clone(),
                icon: None,
                item_type: crate::indexer::ItemType::File,
                category: "RECENT".to_string(),
            })
        } else {
            // O(1) index lookup via path_lookup HashMap
            path_lookup.get(&record.path).and_then(|&idx| {
                let cached = &items[idx];
                if let Some(cat) = forced_category {
                    if cached.category != cat {
                        return None;
                    }
                }
                if let Some(itype) = forced_item_type {
                    if cached.item_type != *itype {
                        return None;
                    }
                }
                let mut recent_item = cached.clone();
                recent_item.category = "RECENT".to_string();
                Some(recent_item)
            })
        };

        if let Some(res_item) = item {
            let match_ok = match matcher {
                Some((m, q)) if !q.is_empty() => m.fuzzy_match(&res_item.name, q).is_some(),
                _ => true,
            };

            if match_ok {
                recents.push(SearchResult::from(res_item));
            }
        }
    }

    recents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acronym_generation() {
        let item = SearchItem::new(
            "Windows Terminal".to_string(),
            "C:\\wt.exe".to_string(),
            None,
            crate::indexer::ItemType::App,
            "APP".to_string(),
        );
        assert_eq!(item.acronym, "wt");

        let vs_code = SearchItem::new(
            "Visual Studio Code".to_string(),
            "C:\\Code.exe".to_string(),
            None,
            crate::indexer::ItemType::App,
            "APP".to_string(),
        );
        assert_eq!(vs_code.acronym, "vsc");
    }

    #[test]
    fn test_math_detection() {
        assert!(is_math_expression("45 * 12"));
        assert!(is_math_expression("100 / 4"));
        assert!(is_math_expression("2^8"));
        assert!(is_math_expression("sqrt(144)"));
        assert!(!is_math_expression("chrome"));
        assert!(!is_math_expression("readme.md"));
    }

    #[test]
    fn test_folder_clean_name() {
        let item = SearchItem::new(
            "spotlight-win".to_string(),
            "D:\\Projects\\spotlight-win".to_string(),
            None,
            crate::indexer::ItemType::Folder,
            "FOLDER".to_string(),
        );
        assert_eq!(item.name, "spotlight-win");
        assert_eq!(item.normalized_name, "spotlight-win");
        assert!(item.normalized_name.starts_with("spotlight"));
    }

    #[test]
    fn test_system_command_safeguard_invariants() {
        let matcher = SkimMatcherV2::default();
        let temp_file = std::env::temp_dir().join("test_spotlight_shortcuts_1.json");
        let shortcut_manager = crate::shortcuts::ShortcutManager::with_path(temp_file);

        // Deliberate commands must trigger
        let res_shutdown = detect_ambient_intent("shutdown", &shortcut_manager, false, &matcher);
        assert!(res_shutdown.iter().any(|r| r.item.name.contains("Shut Down")));

        let res_restart = detect_ambient_intent("restart", &shortcut_manager, false, &matcher);
        assert!(res_restart.iter().any(|r| r.item.name.contains("Restart")));

        // Accidental or under-length prefixes (< 4 chars) must NOT trigger destructive actions
        let res_accidental = detect_ambient_intent("shu", &shortcut_manager, false, &matcher);
        assert!(!res_accidental.iter().any(|r| r.item.name.contains("Shut Down")));

        let res_res = detect_ambient_intent("res", &shortcut_manager, false, &matcher);
        assert!(!res_res.iter().any(|r| r.item.name.contains("Restart")));
    }

    #[test]
    fn test_url_single_focused_action() {
        let matcher = SkimMatcherV2::default();
        let temp_file = std::env::temp_dir().join("test_spotlight_shortcuts_2.json");
        let shortcut_manager = crate::shortcuts::ShortcutManager::with_path(temp_file);

        let results = detect_ambient_intent("github.com", &shortcut_manager, false, &matcher);
        let url_actions: Vec<_> = results.iter().filter(|r| r.item.category == "WEB").collect();
        
        // Invariant: Exactly one primary "Open" action must be presented
        assert_eq!(url_actions.len(), 1, "URL typing must yield exactly 1 focused action");
        assert!(url_actions[0].item.name.starts_with("Open github.com"));
    }

    #[test]
    fn test_tantivy_indexes_and_retrieves_files_and_folders() {
        let temp_dir = std::env::temp_dir().join(format!("spotlight_tantivy_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let engine = crate::index_engine::IndexEngine::open(&temp_dir).unwrap();
        let items = vec![
            SearchItem::new("Quarterly Report".to_string(), "C:\\Docs\\report.pdf".to_string(), None, crate::indexer::ItemType::File, "DOC".to_string()),
            SearchItem::new("Vacation Photos".to_string(), "C:\\Pictures\\Vacation".to_string(), None, crate::indexer::ItemType::Folder, "FOLDER".to_string()),
            SearchItem::new("Intro Video".to_string(), "C:\\Videos\\intro.mp4".to_string(), None, crate::indexer::ItemType::File, "VID".to_string()),
            SearchItem::new("aquaFlow".to_string(), "D:\\DevelopmentSide\\React Native\\aquaFlow".to_string(), None, crate::indexer::ItemType::Folder, "FOLDER".to_string()),
        ];
        engine.bulk_add(&items).unwrap();
        assert_eq!(engine.reader.searcher().num_docs(), 4);

        let aqua_cand = engine.search_candidates("aqua", 5);
        assert_eq!(aqua_cand.len(), 1, "aqua should match aquaFlow via prefix!");
        assert_eq!(aqua_cand[0].0, "D:\\DevelopmentSide\\React Native\\aquaFlow");

        let report_cand = engine.search_candidates("report", 5);
        assert_eq!(report_cand.len(), 1);
        assert_eq!(report_cand[0].0, "C:\\Docs\\report.pdf");

        let folder_cand = engine.search_candidates("vacation", 5);
        assert_eq!(folder_cand.len(), 1);
        assert_eq!(folder_cand[0].0, "C:\\Pictures\\Vacation");

        let video_cand = engine.search_candidates("intro", 5);
        assert_eq!(video_cand.len(), 1);
        assert_eq!(video_cand[0].0, "C:\\Videos\\intro.mp4");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

