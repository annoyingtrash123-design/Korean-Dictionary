//! English Wiktionary (kaikki.org English extract) -> Korean translations (`en_ko` table).
//!
//! Each JSON line is one English entry (`word`, `pos`, `senses[]`); the Korean renderings sit in
//! `translations[]` items with `lang_code: "ko"` / `lang: "Korean"`, either at the top level
//! (each item with its own `sense` label) or inside a sense (labelled by the item's `sense`, else
//! the sense's first gloss). The file is several GB: it is streamed line by line, and `fetch`
//! already keeps only the lines that mention Korean.
//!
//! Kept compact: Korean words must be Hangul (parenthesised hanja such as `보고(報告)` is
//! dropped, anything with Latin / Han left is skipped), rows are deduplicated per
//! (term, pos, sense, ko), and senses / words per term are capped.

use crate::common::*;
use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

/// Distinct (pos, sense) groups kept per term.
pub const MAX_SENSES: usize = 12;
/// Korean words kept per (pos, sense).
pub const MAX_KO_PER_SENSE: usize = 6;
/// Rows kept per term.
pub const MAX_ROWS_PER_TERM: usize = 48;
/// Longest English term kept (chars).
pub const MAX_TERM: usize = 60;
/// Longest sense label kept (chars).
pub const MAX_SENSE: usize = 90;

/// One `en_ko` row.
#[derive(Debug, Clone, PartialEq)]
pub struct EnKoRow {
    /// Lookup key: lower-cased, whitespace collapsed.
    pub term_norm: String,
    /// Display form when it differs from `term_norm` (capitals), else `None`.
    pub term: Option<String>,
    pub pos: String,
    pub sense: Option<String>,
    pub ko: String,
    pub roman: Option<String>,
    /// Order within the term (0 = first), source order.
    pub rank: i64,
}

#[derive(Debug, Default)]
pub struct EnKo {
    pub rows: Vec<EnKoRow>,
    /// English entries that had at least one usable Korean translation.
    pub entries: i64,
    /// Korean translation items dropped (not Hangul, empty, over a cap, duplicate).
    pub dropped: i64,
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

/// `term_norm`: lower-cased, whitespace collapsed and trimmed.
pub fn norm_term(t: &str) -> String {
    t.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// kaikki `pos` -> the readable label shown in the app.
pub fn en_pos(p: &str) -> &str {
    match p {
        "adj" => "adjective",
        "adv" => "adverb",
        "prep" => "preposition",
        "conj" => "conjunction",
        "intj" => "interjection",
        "pron" => "pronoun",
        "num" => "numeral",
        "det" => "determiner",
        "name" => "proper noun",
        "prep_phrase" => "phrase",
        "" => "other",
        other => other,
    }
}

/// Korean word of a translation item, or `None` when it is not plain Hangul:
/// parenthesised parts (`보고(報告)`, `(을)`) are removed, the rest may hold Hangul syllables,
/// spaces and `-`/`~` only.
pub fn clean_ko_word(w: &str) -> Option<String> {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in w.chars() {
        match c {
            '(' | '（' | '[' => depth += 1,
            ')' | '）' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let out = out.trim_matches(|c: char| c == ',' || c == ';' || c == '/' || c.is_whitespace()).to_string();
    if out.is_empty() || !has_hangul(&out) {
        return None;
    }
    if !out.chars().all(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c) || c == ' ' || c == '-' || c == '~') {
        return None;
    }
    Some(out)
}

fn is_korean(t: &Value) -> bool {
    let lc = s(t, "lang_code");
    if !lc.is_empty() {
        return lc == "ko";
    }
    s(t, "lang") == "Korean"
}

/// Items under Wiktionary's "Translations to be checked" are unverified: skipped.
fn unchecked(t: &Value) -> bool {
    s(t, "sense").trim().to_lowercase().starts_with("translations to be checked")
}

fn sense_label(raw: &str) -> Option<String> {
    let c = clean(raw);
    let c = c.trim().trim_end_matches(['.', ':']).trim();
    if c.is_empty() {
        return None;
    }
    Some(truncate(c, MAX_SENSE))
}

/// Per-term accumulation state while streaming.
#[derive(Default)]
struct TermState {
    rows: usize,
    groups: Vec<(String, Option<String>)>,
    per_group: HashMap<(String, Option<String>), usize>,
    seen: HashSet<(String, Option<String>, String)>,
}

/// Cheap byte pre-filter (used by `fetch` to cut the multi-GB file down, and by [`parse`]):
/// the line mentions `Korean` or `"ko"` somewhere.
pub fn mentions_korean(line: &[u8]) -> bool {
    line.windows(6).any(|w| w == b"Korean") || line.windows(4).any(|w| w == b"\"ko\"")
}

/// Stream the English kaikki JSONL and collect the Korean translation rows.
pub fn parse<R: BufRead>(rd: R) -> Result<EnKo> {
    let mut out = EnKo::default();
    let mut terms: HashMap<String, TermState> = HashMap::new();
    for line in rd.split(b'\n') {
        let line = line?;
        if !mentions_korean(&line) {
            continue;
        }
        let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
        if let Some(lc) = v.get("lang_code").and_then(Value::as_str) {
            if lc != "en" {
                continue;
            }
        }
        let word = clean(s(&v, "word"));
        let term_norm = norm_term(&word);
        if term_norm.is_empty() || term_norm.chars().count() > MAX_TERM {
            continue;
        }
        let pos = en_pos(s(&v, "pos")).to_string();
        // (sense label, translation item) in source order: per-sense first, then top level
        let mut items: Vec<(Option<String>, &Value)> = Vec::new();
        for sn in v.get("senses").and_then(Value::as_array).into_iter().flatten() {
            let gloss = sn.get("glosses").and_then(Value::as_array).and_then(|g| g.first()).and_then(Value::as_str).unwrap_or("");
            for t in sn.get("translations").and_then(Value::as_array).into_iter().flatten().filter(|t| is_korean(t) && !unchecked(t)) {
                let label = sense_label(s(t, "sense")).or_else(|| sense_label(gloss));
                items.push((label, t));
            }
        }
        for t in v.get("translations").and_then(Value::as_array).into_iter().flatten().filter(|t| is_korean(t) && !unchecked(t)) {
            items.push((sense_label(s(t, "sense")), t));
        }
        if items.is_empty() {
            continue;
        }
        let display = Some(word.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|w| *w != term_norm);
        let st = terms.entry(term_norm.clone()).or_default();
        let mut any = false;
        for (sense, t) in items {
            let Some(ko) = clean_ko_word(s(t, "word")) else {
                out.dropped += 1;
                continue;
            };
            let gk = (pos.clone(), sense.clone());
            if st.rows >= MAX_ROWS_PER_TERM || !st.seen.insert((pos.clone(), sense.clone(), ko.clone())) {
                out.dropped += 1;
                continue;
            }
            if !st.groups.contains(&gk) {
                if st.groups.len() >= MAX_SENSES {
                    out.dropped += 1;
                    continue;
                }
                st.groups.push(gk.clone());
            }
            let n = st.per_group.entry(gk).or_default();
            if *n >= MAX_KO_PER_SENSE {
                out.dropped += 1;
                continue;
            }
            *n += 1;
            let roman = Some(clean(s(t, "roman"))).filter(|r| !r.is_empty());
            out.rows.push(EnKoRow {
                term_norm: term_norm.clone(),
                term: display.clone(),
                pos: pos.clone(),
                sense,
                ko,
                roman,
                rank: st.rows as i64,
            });
            st.rows += 1;
            any = true;
        }
        if any {
            out.entries += 1;
        }
    }
    out.rows.sort_by(|a, b| (&a.term_norm, a.rank).cmp(&(&b.term_norm, b.rank)));
    Ok(out)
}

/// Words that do not get a phrase-index row (too common to be useful as a "contains" key).
const STOP: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "at", "by", "for", "with", "and", "or", "as", "is", "be", "it", "its",
    "one's", "oneself", "someone", "someone's", "something", "somebody", "sb", "sth", "from", "into", "up", "out",
];

/// `en_ko_words` rows of a multi-word term: every word after the first (punctuation trimmed,
/// stop words skipped, deduplicated). The first word is served by the `term_norm` prefix range.
pub fn phrase_words(term_norm: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in term_norm.split(' ').skip(1) {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-');
        if w.is_empty() || STOP.contains(&w) || out.iter().any(|x| x == w) {
            continue;
        }
        out.push(w.to_string());
    }
    out
}
