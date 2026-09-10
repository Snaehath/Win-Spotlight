use std::time::{SystemTime, UNIX_EPOCH};

// ── Composite Ranking Weights ────────────────────────────────────────────────
// Clean separation: Match Quality (70%), Personal History (25%), Context (5%)
const W_MATCH_COMPONENT: f32    = 0.70;
const W_PERSONAL_COMPONENT: f32 = 0.25;
const W_CONTEXT_COMPONENT: f32  = 0.05;

/// Returns the current Unix timestamp in seconds (called once per search query)
pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compute a final composite score for a search result.
///
/// Cleanly separates:
/// 1. Match quality (Exact / Prefix / Acronym / Fuzzy) - normalized without early saturation
/// 2. Personal profile (Log frequency + Exponential 7-day half-life recency)
/// 3. Contextual relevance (Time-of-day affinity + App vs File priority)
pub fn compute_score(
    raw_match_score: i64,
    launch_count: u64,
    last_launched_secs: u64,
    time_score: f32,
    is_app: bool,
    now_secs: u64,
) -> f32 {
    // 1. MATCH SCORE: Scale by 1600 so exact matches (1200) + app boosts (+150)
    // retain gradient rather than saturating prematurely at 1000.
    let match_norm = (raw_match_score as f32 / 1600.0).clamp(0.0, 1.0);

    // 2. PERSONAL SCORE:
    // - Frequency: log10(count+1) scaled 0..1 (caps at ~100 launches)
    let frequency = ((launch_count as f32 + 1.0).log10() / 2.0).clamp(0.0, 1.0);

    // - Recency: True exponential half-life decay (7-day half-life: 2^(-days_ago / 7))
    // 0 days = 1.0, 7 days = 0.5, 14 days = 0.25, 28 days = 0.0625
    let recency = if last_launched_secs == 0 {
        0.0_f32
    } else {
        let secs_ago = now_secs.saturating_sub(last_launched_secs);
        let days_ago = secs_ago as f32 / 86400.0;
        (0.5_f32).powf(days_ago / 7.0)
    };
    let personal_norm = (frequency * 0.60) + (recency * 0.40);

    // 3. CONTEXT SCORE: Time of day pattern + App vs File priority
    let type_bonus: f32 = if is_app { 1.0 } else { 0.35 };
    let context_norm = (time_score * 0.50) + (type_bonus * 0.50);

    // Final blended score
    (match_norm * W_MATCH_COMPONENT)
        + (personal_norm * W_PERSONAL_COMPONENT)
        + (context_norm * W_CONTEXT_COMPONENT)
}

/// Lightweight scored index for zero-clone ranking in hot paths
#[derive(Debug, Copy, Clone)]
pub struct ScoredIndex {
    pub index: usize,
    pub score: f32,
}

impl ScoredIndex {
    pub fn new(index: usize, score: f32) -> Self {
        ScoredIndex { index, score }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_recency_decay() {
        let now = 1_000_000u64;
        let day_secs = 86_400u64;

        // 0 days ago -> recency component 1.0
        let s0 = compute_score(1000, 10, now, 0.5, true, now);
        // 7 days ago -> half-life decay to 0.5
        let s7 = compute_score(1000, 10, now - 7 * day_secs, 0.5, true, now);
        // 14 days ago -> decay to 0.25
        let s14 = compute_score(1000, 10, now - 14 * day_secs, 0.5, true, now);

        assert!(s0 > s7, "Recent items must score higher than 7-day old items");
        assert!(s7 > s14, "7-day old items must score higher than 14-day old items");
    }

    #[test]
    fn test_match_separation_no_saturation() {
        let now = 1_000_000u64;
        let exact = compute_score(1350, 0, 0, 0.0, true, now);
        let prefix = compute_score(850, 0, 0, 0.0, true, now);
        let fuzzy = compute_score(400, 0, 0, 0.0, true, now);

        assert!(exact > prefix, "Exact matches must outscore prefix matches");
        assert!(prefix > fuzzy, "Prefix matches must outscore fuzzy matches");
    }

    #[test]
    fn test_benchmark_ranking_throughput() {
        let now = current_timestamp_secs();
        let start = std::time::Instant::now();
        const CANDIDATE_COUNT: usize = 10_000;

        let mut scored: Vec<ScoredIndex> = (0..CANDIDATE_COUNT)
            .map(|i| {
                let match_score = ((i % 1200) + 100) as i64;
                let score = compute_score(match_score, (i % 50) as u64, now - (i as u64 * 3600), 0.5, i % 2 == 0, now);
                ScoredIndex::new(i, score)
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let elapsed = start.elapsed();

        println!(
            "\n[Benchmark] Ranked and sorted {} candidates in {:?} ({:.3} µs/candidate)",
            CANDIDATE_COUNT,
            elapsed,
            elapsed.as_micros() as f64 / CANDIDATE_COUNT as f64
        );
        assert!(!scored.is_empty());
        assert!(elapsed.as_millis() < 50, "10k candidates must rank in under 50ms");
    }
}
