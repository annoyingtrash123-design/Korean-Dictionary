//! Word lists for the Games tab: random words of a TOPIK level, details for given headwords
//! (recent lookups, Reader vocabulary), and nouns to continue a 끝말잇기 word chain.

use super::{norm_headword, PackDb};
use crate::sql::{Result, Row};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GameWord {
    pub source: String,
    pub id: i64,
    pub headword: String,
    pub hanja: Option<String>,
    pub pos: Option<String>,
    pub level: Option<i64>,
    pub gloss: String,
    /// Korean example sentences (krdict "sentence" examples, no dialogues), for Fill the gap.
    pub examples: Vec<String>,
}

const COLS: &str = "e.source, e.id, e.headword, e.hanja, e.pos, e.level, e.gloss, e.data";
/// Content words with a short English gloss: what a quiz can ask about.
const QUIZ_POS: &str = "('noun','verb','adjective','adverb')";
const MAX_EXAMPLES: usize = 4;

fn word(r: &Row) -> GameWord {
    let data: serde_json::Value = serde_json::from_str(&r.string(7)).unwrap_or_default();
    let mut examples = Vec::new();
    for s in data.get("senses").and_then(|v| v.as_array()).into_iter().flatten() {
        for ex in s.get("examples").and_then(|v| v.as_array()).into_iter().flatten() {
            let ko = ex.get("ko").and_then(|v| v.as_str()).unwrap_or("").trim();
            let sentence = ex.get("type").and_then(|v| v.as_str()).is_none_or(|t| t == "sentence");
            if sentence && !ko.is_empty() && !ko.contains('\n') && examples.len() < MAX_EXAMPLES {
                examples.push(ko.to_string());
            }
        }
    }
    GameWord {
        source: r.string(0),
        id: r.int(1).unwrap_or(0),
        headword: r.string(2),
        hanja: r.text(3),
        pos: r.text(4),
        level: r.int(5),
        gloss: r.string(6),
        examples,
    }
}

/// `limit` random quiz words of a krdict level (1 초급, 2 중급, 3 고급).
pub fn game_words(packs: &[&PackDb], level: i64, limit: usize) -> Result<Vec<GameWord>> {
    for p in packs {
        let sql = format!(
            "SELECT {COLS} FROM entries e WHERE e.source = 'krdict' AND e.kind = 'word' AND e.level = ?1 \
             AND e.pos IN {QUIZ_POS} AND e.gloss IS NOT NULL AND e.gloss != '' AND length(e.gloss) <= 60 \
             ORDER BY random() LIMIT ?2"
        );
        let rows = p.conn.query(&sql, &[level.into(), (limit as i64).into()])?;
        if !rows.is_empty() {
            return Ok(dedupe(rows.iter().map(word).collect()));
        }
    }
    Ok(vec![])
}

/// The best quiz entry for each headword (krdict with a level first, then any English gloss), in
/// order; headwords with no English gloss are left out.
pub fn game_words_for(packs: &[&PackDb], headwords: &[String]) -> Result<Vec<GameWord>> {
    let mut out = Vec::new();
    for hw in headwords {
        let n = norm_headword(hw.trim());
        if n.is_empty() {
            continue;
        }
        let mut best: Option<(i64, GameWord)> = None;
        for p in packs {
            let sql = format!(
                "SELECT {COLS}, e.rank FROM entries e WHERE e.hw_norm = ?1 AND e.kind = 'word' \
                 AND e.gloss IS NOT NULL AND e.gloss != '' AND e.lang = 'en'"
            );
            for r in p.conn.query(&sql, &[n.as_str().into()])? {
                let w = word(&r);
                let score = if w.source == "krdict" { 0 } else { 1_000_000 } + if w.level.is_some() { 0 } else { 100_000 } + r.int(8).unwrap_or(0);
                if best.as_ref().is_none_or(|(s, _)| score < *s) {
                    best = Some((score, w));
                }
            }
        }
        if let Some((_, w)) = best {
            out.push(w);
        }
    }
    Ok(dedupe(out))
}

/// Nouns (two syllables or more) starting with any of `starts`, common words first, skipping
/// `used` headwords: the app's next move in 끝말잇기.
pub fn chain_candidates(packs: &[&PackDb], starts: &[String], used: &[String], limit: usize) -> Result<Vec<GameWord>> {
    let used: HashSet<String> = used.iter().map(|u| norm_headword(u)).collect();
    let mut out: Vec<(i64, GameWord)> = Vec::new();
    for s in starts {
        let Some(hi) = next_prefix(s) else { continue };
        for p in packs {
            let sql = format!(
                "SELECT {COLS}, e.rank FROM entries e WHERE e.hw_norm >= ?1 AND e.hw_norm < ?2 AND e.pos = 'noun' \
                 AND e.kind = 'word' AND e.lang = 'en' AND e.gloss IS NOT NULL AND e.gloss != '' AND length(e.hw_norm) >= 2 \
                 ORDER BY (e.level IS NULL), e.rank LIMIT 60"
            );
            for r in p.conn.query(&sql, &[s.as_str().into(), hi.as_str().into()])? {
                let w = word(&r);
                let hwn = norm_headword(&w.headword);
                // plain Hangul nouns only: no spaces, hyphens or numbered homonym marks
                if used.contains(&hwn) || !hwn.chars().all(|c| ('가'..='힣').contains(&c)) {
                    continue;
                }
                let score = if w.level.is_some() { 0 } else { 1_000_000 } + r.int(8).unwrap_or(0);
                out.push((score, w));
            }
        }
    }
    out.sort_by_key(|(s, _)| *s);
    let mut words = dedupe(out.into_iter().map(|(_, w)| w).collect());
    words.truncate(limit);
    Ok(words)
}

/// Is `word` a noun in any installed dictionary? Returns its entry (with an English gloss when
/// one exists) so the game can show what the player's word means.
pub fn chain_check(packs: &[&PackDb], word_in: &str) -> Result<Option<GameWord>> {
    let n = norm_headword(word_in.trim());
    if n.chars().count() < 2 {
        return Ok(None);
    }
    let mut best: Option<(i64, GameWord)> = None;
    for p in packs {
        let sql = format!("SELECT {COLS}, e.rank, e.lang FROM entries e WHERE e.hw_norm = ?1 AND e.pos = 'noun' AND e.kind = 'word'");
        for r in p.conn.query(&sql, &[n.as_str().into()])? {
            let w = word(&r);
            let score = if r.string(9) == "en" && !w.gloss.is_empty() { 0 } else { 1_000_000 } + r.int(8).unwrap_or(0);
            if best.as_ref().is_none_or(|(s, _)| score < *s) {
                best = Some((score, w));
            }
        }
    }
    Ok(best.map(|(_, w)| w))
}

/// Exclusive upper bound for a prefix range: the prefix with its last character bumped.
fn next_prefix(s: &str) -> Option<String> {
    let mut cs: Vec<char> = s.chars().collect();
    let last = cs.pop()?;
    cs.push(char::from_u32(last as u32 + 1)?);
    Some(cs.into_iter().collect())
}

/// One word per headword (the first, i.e. best-ranked).
fn dedupe(words: Vec<GameWord>) -> Vec<GameWord> {
    let mut seen = HashSet::new();
    words.into_iter().filter(|w| seen.insert(norm_headword(&w.headword))).collect()
}
