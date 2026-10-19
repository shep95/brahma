//! The distilled research corpus and a small ranked search over it.
//!
//! `data/corpus.json` is produced by `scripts/distill.py` from the design-research notes the
//! recommender was trained on: one entry per historical pattern, present-state finding,
//! technique or cross-domain convergence, each with a title, a short summary, its domain and
//! keyword tags. Search scores an entry by the query terms it contains (title hits count most,
//! tags next, the summary least) with a length normalisation, so short exact entries beat long
//! vague ones: a plain BM25-style ranking with no tuning data needed.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub domain: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

const CORPUS_JSON: &str = include_str!("../data/corpus.json");

/// Every entry, parsed once. An unreadable corpus (it is checked in; this guards a bad edit)
/// yields no entries rather than a panic.
pub fn entries() -> &'static [Entry] {
    static ALL: OnceLock<Vec<Entry>> = OnceLock::new();
    ALL.get_or_init(|| serde_json::from_str(CORPUS_JSON).unwrap_or_default())
}

/// Lower-case word tokens of at least three letters (two for things like "3d" and "ui").
fn tokens(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !c.is_alphanumeric() && c != '-').filter(|w| w.len() >= 2 && !STOP.contains(w)).map(str::to_owned).collect()
}

const STOP: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "are", "was", "not", "but", "its", "into", "than", "then", "when", "what", "how", "why", "all", "any",
    "has", "have", "had", "can", "will", "you", "your", "our", "out", "one", "two", "per", "via", "use", "used", "uses", "is", "in", "on", "of", "to", "a",
    "an", "as", "at", "by", "or", "it", "be", "so", "no", "if", "do",
];

/// Entries ranked for `query` (free text or a list of tags); at most `limit`, best first, with
/// a score. An empty query returns nothing.
pub fn search(query: &str, limit: usize) -> Vec<(f32, &'static Entry)> {
    let q = tokens(query);
    if q.is_empty() || limit == 0 {
        return Vec::new();
    }
    let all = entries();
    let avg_len = (all.iter().map(|e| e.summary.len()).sum::<usize>() as f32 / all.len().max(1) as f32).max(1.0);
    let mut scored: Vec<(f32, &Entry)> = all
        .iter()
        .filter_map(|e| {
            let title = tokens(&e.title);
            let summary = e.summary.to_lowercase();
            let mut score = 0.0_f32;
            for term in &q {
                let in_title = title.iter().any(|t| t == term || t.starts_with(term.as_str()));
                let in_tags = e.tags.iter().any(|t| t == term);
                let in_domain = e.domain.to_lowercase().contains(term.as_str());
                let count = summary.matches(term.as_str()).count() as f32;
                let tf = count / (count + 1.2 * (0.25 + 0.75 * e.summary.len() as f32 / avg_len));
                score += 3.0 * f32::from(u8::from(in_title)) + 2.0 * f32::from(u8::from(in_tags)) + 1.5 * f32::from(u8::from(in_domain)) + 2.0 * tf;
            }
            (score > 0.0).then_some((score, e))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.1.title.cmp(&b.1.title)));
    scored.truncate(limit);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corpus_loads_and_is_tagged() {
        let all = entries();
        assert!(all.len() >= 100, "{}", all.len());
        assert!(all.iter().all(|e| !e.title.is_empty() && e.summary.len() >= 40));
        assert!(all.iter().filter(|e| e.tags.contains(&"color".to_string())).count() >= 20);
    }

    #[test]
    fn search_ranks_the_obvious_entry_first() {
        let top = search("oklch perceptual colour palette", 5);
        assert!(!top.is_empty());
        assert!(top.iter().take(3).any(|(_, e)| e.title.to_lowercase().contains("oklch")), "{:?}", top.iter().map(|(s, e)| (s, &e.title)).collect::<Vec<_>>());
        let motion = search("spring animation easing", 5);
        assert!(motion.iter().any(|(_, e)| e.tags.contains(&"motion".to_string())));
        assert!(search("", 5).is_empty());
        assert!(search("zzzz qqqq", 5).is_empty());
        assert_eq!(search("color", 3).len(), 3);
    }
}
