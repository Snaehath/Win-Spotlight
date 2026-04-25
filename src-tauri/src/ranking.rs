use std::time::{SystemTime, UNIX_EPOCH};
use crate::indexer::SearchItem;

/// Weights for the Pro ranking formula
const W_FUZZY: f32     = 0.55; // Fuzzy match quality
const W_FREQUENCY: f32 = 0.15; // How often user launches this
const W_RECENCY: f32   = 0.12; // More recent = higher boost
const W_ADAPTIVE: f32  = 0.10; // Time of day relevance (Phase 3)
const W_TYPE: f32      = 0.05; // Apps score higher than files
const W_DEPTH: f32     = 0.03; // Shorter paths rank higher

/// Returns the current Unix timestamp in seconds
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compute a final composite score for a search result.
///
/// - `fuzzy_score`: raw score from fuzzy-matcher (0..=1000, normalized to 0..=1)
/// - `launch_count`: number of times the user has launched this item
/// - `last_launched_secs`: Unix timestamp of the last launch (0 = never)
/// - `time_score`: score from HistoryManager (0.0 .. 1.0) based on current hour
/// - `path_depth`: number of components in the path (fewer = better)
/// - `is_app`: true if this is an .exe / .lnk
pub fn compute_score(
    fuzzy_score: i64,
    launch_count: u64,
    last_launched_secs: u64,
    time_score: f32,
    path_depth: usize,
    is_app: bool,
) -> f32 {
    // Normalize fuzzy score to 0..1 (typical SkimMatcher output ~0..1000)
    let fuzzy = (fuzzy_score as f32 / 1000.0).clamp(0.0, 1.0);

    // Frequency boost: log10(count+1) scaled 0..1 (caps at ~100 launches)
    let frequency = ((launch_count as f32 + 1.0).log10() / 2.0).clamp(0.0, 1.0);

    // Recency boost: 1 / (days_ago + 1), so "today" = 1.0, "one year ago" ≈ 0.003
    let recency = if last_launched_secs == 0 {
        0.0_f32
    } else {
        let secs_ago = now_secs().saturating_sub(last_launched_secs);
        let days_ago = secs_ago as f32 / 86400.0;
        1.0 / (days_ago + 1.0)
    };

    // Type bonus: Apps are more likely to be the user's intent
    let type_bonus: f32 = if is_app { 1.0 } else { 0.6 };

    // Depth penalty: prefer items near the root of a drive
    let depth_score = (1.0 / (path_depth as f32 + 1.0)).clamp(0.0, 1.0);

    (fuzzy * W_FUZZY)
        + (frequency * W_FREQUENCY)
        + (recency * W_RECENCY)
        + (time_score * W_ADAPTIVE)
        + (type_bonus * W_TYPE)
        + (depth_score * W_DEPTH)
}

/// Scored wrapper around SearchItem – used to sort results before returning to UI
#[derive(Debug)]
pub struct ScoredItem {
    pub item: SearchItem,
    pub score: f32,
}

impl ScoredItem {
    pub fn new(item: SearchItem, score: f32) -> Self {
        ScoredItem { item, score }
    }
}

/// Sort a list of ScoredItems descending by score
pub fn rank(mut scored: Vec<ScoredItem>) -> Vec<SearchItem> {
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|s| s.item).collect()
}
