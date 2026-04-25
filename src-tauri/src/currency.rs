use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::indexer::{SearchItem, ItemType};
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

pub fn get_currency_rates() -> Option<HashMap<String, f64>> {
    let path = get_cache_path();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut cached_rates = None;
    let mut needs_refresh = true;

    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(cache) = serde_json::from_str::<CurrencyCache>(&content) {
            cached_rates = Some(cache.rates.clone());
            if now - cache.fetched_at < 3600 { // 1 hour cache
                needs_refresh = false;
            }
        }
    }

    if needs_refresh {
        thread::spawn(move || {
            // Using frankfurter API. Note: it returns rates relative to USD if from=USD.
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
                        let _ = fs::write(&path, serde_json::to_string(&new_cache).unwrap_or_default());
                    }
                }
            }
        });
    }

    cached_rates
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
            
            let synthetic = SearchItem {
                name: display.clone(),
                path: web_search_query,
                icon: None,
                item_type: ItemType::File,
                category: "COMMAND".to_string(),
            };
            results.push(SearchResult {
                item: synthetic,
                inline_display: Some(display),
            });
        }
    }

    results
}
