use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::indexer::SearchItem;
use crate::search::SearchResult;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrencyCache {
    pub fetched_at: u64,
    pub rates: HashMap<String, f64>,
}

fn get_cache_path() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "spotlight", "launcher") {
        let mut p = proj_dirs.data_dir().to_path_buf();
        let _ = fs::create_dir_all(&p);
        p.push("currency_cache.json");
        p
    } else {
        PathBuf::from("currency_cache.json")
    }
}

use std::sync::OnceLock;
use std::sync::Mutex;

static CURRENCY_IN_MEMORY_CACHE: OnceLock<Mutex<Option<CurrencyCache>>> = OnceLock::new();

fn get_in_memory_cache() -> &'static Mutex<Option<CurrencyCache>> {
    CURRENCY_IN_MEMORY_CACHE.get_or_init(|| Mutex::new(None))
}

pub fn get_currency_rates() -> Option<HashMap<String, f64>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cache_lock = get_in_memory_cache();
    let mut cache_guard = cache_lock.lock().unwrap();

    // 1. Return fresh in-memory cache if it exists
    if let Some(ref cache) = *cache_guard {
        if now - cache.fetched_at < 3600 {
            return Some(cache.rates.clone());
        }
    }

    // 2. Otherwise load from disk if memory cache is uninitialized
    let path = get_cache_path();
    let mut disk_cache = None;
    if cache_guard.is_none() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cache) = serde_json::from_str::<CurrencyCache>(&content) {
                disk_cache = Some(cache);
            }
        }
    }

    if let Some(cache) = disk_cache {
        *cache_guard = Some(cache.clone());
        if now - cache.fetched_at < 3600 {
            return Some(cache.rates.clone());
        }
    }

    // 3. Cache is either missing or stale, trigger background refresh
    thread::spawn(move || {
        if let Ok(response) = reqwest::blocking::get("https://api.frankfurter.app/latest?from=USD") {
            if let Ok(json) = response.json::<serde_json::Value>() {
                if let Some(rates_obj) = json.get("rates").and_then(|r| r.as_object()) {
                    let mut map = HashMap::new();
                    map.insert("USD".to_string(), 1.0);
                    for (k, v) in rates_obj {
                        if let Some(f) = v.as_f64() {
                            map.insert(k.clone(), f);
                        }
                    }
                    let new_cache = CurrencyCache {
                        fetched_at: now,
                        rates: map,
                    };
                    if let Ok(serialized) = serde_json::to_string(&new_cache) {
                        let _ = fs::write(&path, serialized);
                    }
                    // Update in-memory cache
                    let cache_lock = get_in_memory_cache();
                    if let Ok(mut guard) = cache_lock.lock() {
                        *guard = Some(new_cache);
                    }
                }
            }
        }
    });

    // Fall back to returning the stale cache rather than blocking or returning None
    cache_guard.as_ref().map(|c| c.rates.clone())
}


fn normalize_currency(s: &str) -> String {
    match s {
        "usd" | "dollar" | "dollars" => "USD".to_string(),
        "eur" | "euro" | "euros" => "EUR".to_string(),
        "gbp" | "pound" | "pounds" => "GBP".to_string(),
        "inr" | "rupee" | "rupees" => "INR".to_string(),
        "jpy" | "yen" => "JPY".to_string(),
        "cad" => "CAD".to_string(),
        "aud" => "AUD".to_string(),
        "kpw" | "krw" | "won" => "KRW".to_string(),
        "chf" | "franc" => "CHF".to_string(),
        "cny" | "rmb" | "yuan" => "CNY".to_string(),
        x if x.len() == 3 && x.chars().all(|c| c.is_ascii_alphabetic()) => x.to_uppercase(),
        _ => "".to_string(),
    }
}

pub fn detect_currency_intent(query: &str) -> Vec<SearchResult> {
    let q_lower = query.trim().to_lowercase();
    let q_replaced = q_lower
        .replace(" to ", " ")
        .replace(" in ", " ")
        .replace("=", " ")
        .replace("$", "usd ")
        .replace("€", "eur ")
        .replace("£", "gbp ")
        .replace("₹", "inr ");

    // Separate numbers from letters so "1usd" becomes "1 usd"
    let mut q = String::new();
    let chars: Vec<char> = q_replaced.chars().collect();
    for i in 0..chars.len() {
        q.push(chars[i]);
        if i + 1 < chars.len() {
            let c1 = chars[i];
            let c2 = chars[i + 1];
            if (c1.is_ascii_digit() && c2.is_alphabetic())
                || (c1.is_alphabetic() && c2.is_ascii_digit())
            {
                q.push(' ');
            }
        }
    }

    let parts: Vec<&str> = q.split_whitespace().collect();
    if parts.len() < 2 || parts.len() > 4 {
        return Vec::new();
    }

    let mut amount = None;
    let mut from_cur = None;
    let mut to_cur = None;

    for p in &parts {
        if amount.is_none() && p.parse::<f64>().is_ok() {
            amount = Some(p.parse::<f64>().unwrap());
        } else {
            let symbol = normalize_currency(p);
            if !symbol.is_empty() {
                if from_cur.is_none() {
                    from_cur = Some(symbol);
                } else if to_cur.is_none() {
                    to_cur = Some(symbol);
                }
            }
        }
    }

    let (a, f) = match (amount, from_cur) {
        (Some(a), Some(f)) => (a, f),
        _ => return Vec::new(),
    };

    let rates = match get_currency_rates() {
        Some(r) => r,
        None => return Vec::new(), // Silently fail if no cache available yet
    };

    let from_rate = match rates.get(&f) {
        Some(r) => *r,
        None => return Vec::new(), // unsupported base currency
    };

    let mut results = Vec::new();

    let targets = if let Some(t) = to_cur {
        vec![t]
    } else {
        vec!["EUR".to_string(), "GBP".to_string(), "INR".to_string(), "JPY".to_string()]
    };

    for target in targets {
        if target == f { continue; }
        if let Some(to_rate) = rates.get(&target) {
            let converted = a * (to_rate / from_rate);
            let formatted_amount = format!("{:.2}", converted);
            
            let display = format!("{} {} = {} {}", a, f, formatted_amount, target);
            let web_search_query = format!("COMMAND:> g {} {} to {}", a, f, target);
            let synthetic = SearchItem::synthetic(display.clone(), web_search_query, "COMMAND");
            results.push(SearchResult {
                item: synthetic,
                inline_display: Some(display),
            });
        }
    }

    results
}
