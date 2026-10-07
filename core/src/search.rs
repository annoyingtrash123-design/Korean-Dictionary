//! Search and query logic over the pack databases (see docs/SCOPE.md, "Database contract" and
//! "App behaviour"). Plain Rust over [`Conn`], so it is testable natively.

use crate::deconjugate::{deconjugate, grammar_hints};
use crate::hangul::{has_han, has_hangul, is_han, norm_headword};
use crate::sql::{Conn, Result, Row, SqlError, Val};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub const DEFAULT_LIMIT: usize = 100;
const PREFIX_LIMIT: usize = 50;
const EXACT_LIMIT: usize = 50;

/// Which optional tables a pack contains.
#[derive(Debug, Clone, Default)]
pub struct Caps {
    pub forms: bool,
    pub hanja_words: bool,
    pub entries_fts: bool,
    pub hanja_chars: bool,
    pub sentences: bool,
    pub sentences_fts: bool,
    pub grammar: bool,
    /// `entries_fts` has the short-gloss `head` column (weighted higher).
    pub fts_head: bool,
}

/// An opened pack database.
pub struct PackDb {
    pub id: String,
    pub conn: Conn,
    pub caps: Caps,
}

impl PackDb {
    pub fn new(id: &str, conn: Conn) -> Result<PackDb> {
        let rows = conn.query("SELECT name FROM sqlite_master WHERE type IN ('table','view')", &[])?;
        let names: HashSet<String> = rows.iter().map(|r| r.string(0)).collect();
        if !names.contains("entries") {
            return Err(SqlError(format!("pack '{id}' has no entries table")));
        }
        let caps = Caps {
            forms: names.contains("forms"),
            hanja_words: names.contains("hanja_words"),
            entries_fts: names.contains("entries_fts"),
            hanja_chars: names.contains("hanja_chars"),
            sentences: names.contains("sentences"),
            sentences_fts: names.contains("sentences_fts"),
            grammar: names.contains("grammar"),
            fts_head: names.contains("entries_fts")
                && conn
                    .query("SELECT name FROM pragma_table_info('entries_fts')", &[])
                    .map(|r| r.iter().any(|r| r.string(0) == "head"))
                    .unwrap_or(false),
        };
        Ok(PackDb { id: id.to_string(), conn, caps })
    }

    /// Value of `meta.<key>`, if the table and key exist.
    pub fn meta(&self, key: &str) -> Option<String> {
        self.conn
            .query("SELECT value FROM meta WHERE key = ?1", &[key.into()])
            .ok()
            .and_then(|r| r.first().and_then(|r| r.text(0)))
    }
}

// ---- output types (serialised to JS with serde-wasm-bindgen) --------------------------------

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResultRow {
    pub source: String,
    pub id: i64,
    pub headword: String,
    pub hanja: Option<String>,
    pub pos: Option<String>,
    pub level: Option<i64>,
    pub gloss: Option<String>,
    pub kind: String,
    pub pack: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<&'static str>,
    pub rank: i64,
    pub homonym: Option<i64>,
    pub pron: Option<String>,
    pub lang: String,
    pub hw_norm: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HanjaChar {
    pub ch: String,
    pub readings: Option<String>,
    pub meaning_en: Option<String>,
    pub strokes: Option<i64>,
    pub radical: Option<String>,
    pub word_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GrammarRow {
    pub id: i64,
    pub entry_id: Option<i64>,
    pub pattern: String,
    pub category: String,
    pub level: Option<i64>,
    pub summary_en: Option<String>,
    pub sort: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeconjOut {
    pub lemma: String,
    pub rule: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchResult {
    pub mode: &'static str,
    pub rows: Vec<ResultRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanja: Option<Vec<HanjaChar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deconj: Option<Vec<DeconjOut>>,
    #[serde(rename = "grammarHints", skip_serializing_if = "Option::is_none")]
    pub grammar_hints: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Entry {
    pub id: i64,
    pub headword: String,
    pub hw_norm: String,
    pub homonym: Option<i64>,
    pub hanja: Option<String>,
    pub pos: Option<String>,
    pub pron: Option<String>,
    pub source: String,
    pub lang: String,
    pub level: Option<i64>,
    pub rank: i64,
    pub kind: String,
    pub gloss: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SentenceRow {
    pub ko: String,
    pub en: Option<String>,
    pub source: String,
}

// ---- helpers --------------------------------------------------------------------------------

const COLS: &str = "e.source, e.id, e.headword, e.hanja, e.pos, e.level, e.gloss, e.kind, e.rank, e.homonym, e.pron, e.lang, e.hw_norm";

fn opt_text(r: &Row, i: usize) -> Option<String> {
    r.text(i)
}

fn result_row(pack: &str, r: &Row, via: Option<&'static str>) -> ResultRow {
    ResultRow {
        source: r.string(0),
        id: r.int(1).unwrap_or(0),
        headword: r.string(2),
        hanja: opt_text(r, 3),
        pos: opt_text(r, 4),
        level: r.int(5),
        gloss: opt_text(r, 6),
        kind: r.string(7),
        pack: pack.to_string(),
        via,
        rank: r.int(8).unwrap_or(i64::MAX),
        homonym: r.int(9),
        pron: opt_text(r, 10),
        lang: r.string(11),
        hw_norm: r.string(12),
    }
}

fn key(r: &ResultRow) -> (String, i64) {
    (r.pack.clone(), r.id)
}

fn sort_by_rank(v: &mut [ResultRow]) {
    v.sort_by_key(|r| r.rank);
}

/// Append `rows` to `out` unless already present (by pack + id).
fn push_new(out: &mut Vec<ResultRow>, seen: &mut HashSet<(String, i64)>, rows: Vec<ResultRow>) {
    for r in rows {
        if seen.insert(key(&r)) {
            out.push(r);
        }
    }
}

// ---- search ---------------------------------------------------------------------------------

/// Search across the given (already filtered to enabled) packs.
pub fn search(packs: &[&PackDb], query: &str, limit: Option<usize>) -> Result<SearchResult> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).max(1);
    let q = query.trim();
    if q.is_empty() {
        return Ok(SearchResult { mode: "english", rows: vec![], hanja: None, deconj: None, grammar_hints: None });
    }
    if has_hangul(q) {
        search_hangul(packs, q, limit)
    } else if has_han(q) {
        search_hanja(packs, q, limit)
    } else {
        search_english(packs, q, limit)
    }
}

fn search_hangul(packs: &[&PackDb], q_raw: &str, limit: usize) -> Result<SearchResult> {
    let q = norm_headword(q_raw);
    let mut out: Vec<ResultRow> = Vec::new();
    let mut seen: HashSet<(String, i64)> = HashSet::new();

    // 1. exact headword
    let mut exact = Vec::new();
    for p in packs {
        let sql = format!("SELECT {COLS} FROM entries e WHERE e.hw_norm = ?1 ORDER BY e.rank LIMIT {EXACT_LIMIT}");
        for r in p.conn.query(&sql, &[q.as_str().into()])? {
            exact.push(result_row(&p.id, &r, Some("exact")));
        }
    }
    sort_by_rank(&mut exact);
    push_new(&mut out, &mut seen, exact);

    // 2. conjugated / variant forms listed in the pack
    let mut forms = Vec::new();
    for p in packs.iter().filter(|p| p.caps.forms) {
        let sql = format!(
            "SELECT {COLS} FROM forms f JOIN entries e ON e.id = f.entry_id WHERE f.form = ?1 ORDER BY e.rank LIMIT {EXACT_LIMIT}"
        );
        for r in p.conn.query(&sql, &[q.as_str().into()])? {
            forms.push(result_row(&p.id, &r, Some("form")));
        }
    }
    sort_by_rank(&mut forms);
    push_new(&mut out, &mut seen, forms);

    // 3. deconjugation candidates that exist in the database
    let cands = deconjugate(&q);
    let mut deconj_out: Vec<DeconjOut> = Vec::new();
    if !cands.is_empty() {
        let order: HashMap<&str, usize> = cands.iter().enumerate().map(|(i, c)| (c.lemma.as_str(), i)).collect();
        let mut found: Vec<(usize, ResultRow)> = Vec::new();
        for p in packs {
            // chunked IN lists keep us well below SQLite's variable limit
            for chunk in cands.chunks(40) {
                let ph = (1..=chunk.len()).map(|i| format!("?{i}")).collect::<Vec<_>>().join(",");
                let sql = format!("SELECT {COLS} FROM entries e WHERE e.hw_norm IN ({ph}) ORDER BY e.rank");
                let params: Vec<Val> = chunk.iter().map(|c| Val::Text(c.lemma.clone())).collect();
                for r in p.conn.query(&sql, &params)? {
                    let row = result_row(&p.id, &r, Some("deconj"));
                    let idx = order.get(row.hw_norm.as_str()).copied().unwrap_or(usize::MAX);
                    found.push((idx, row));
                }
            }
        }
        found.sort_by_key(|(i, r)| (*i, r.rank));
        let mut matched: HashSet<usize> = HashSet::new();
        for (i, _) in &found {
            matched.insert(*i);
        }
        let mut idxs: Vec<usize> = matched.into_iter().collect();
        idxs.sort();
        for i in idxs {
            if let Some(c) = cands.get(i) {
                deconj_out.push(DeconjOut { lemma: c.lemma.clone(), rule: c.rule.clone() });
            }
        }
        push_new(&mut out, &mut seen, found.into_iter().map(|(_, r)| r).collect());
    }

    // 4. prefix matches
    let upper = format!("{q}{}", char::MAX);
    let mut prefix = Vec::new();
    for p in packs {
        let sql = format!(
            "SELECT {COLS} FROM entries e WHERE e.hw_norm >= ?1 AND e.hw_norm < ?2 ORDER BY e.rank LIMIT {PREFIX_LIMIT}"
        );
        for r in p.conn.query(&sql, &[q.as_str().into(), upper.as_str().into()])? {
            prefix.push(result_row(&p.id, &r, Some("prefix")));
        }
    }
    sort_by_rank(&mut prefix);
    prefix.truncate(PREFIX_LIMIT);
    push_new(&mut out, &mut seen, prefix);

    out.truncate(limit);
    let hints = grammar_hints(&q);
    Ok(SearchResult {
        mode: "hangul",
        rows: out,
        hanja: None,
        deconj: if deconj_out.is_empty() { None } else { Some(deconj_out) },
        grammar_hints: if hints.is_empty() { None } else { Some(hints) },
    })
}

/// FTS5 MATCH expression: every token quoted, the last one a prefix token.
pub fn fts_query(text: &str) -> Option<String> {
    let tokens: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();
    if tokens.is_empty() {
        return None;
    }
    let n = tokens.len();
    let parts: Vec<String> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let quoted = format!("\"{}\"", t.replace('"', "\"\""));
            if i + 1 == n {
                format!("{quoted}*")
            } else {
                quoted
            }
        })
        .collect();
    Some(parts.join(" "))
}

/// Source quality tier for English results: lower is better.
fn quality(r: &ResultRow) -> i64 {
    let src = match r.source.as_str() {
        "krdict" => 0,
        "wikt" => 1,
        "kengdic" => 3,
        _ => 2,
    };
    let kind = if matches!(r.kind.as_str(), "phrase" | "idiom" | "proverb") { 1 } else { 0 };
    src + kind
}

/// How well the entry's short glosses match the query: 0 first gloss is exactly the query,
/// 1 another gloss is, 2 a gloss starts with the query ("thank" → "thankful"), 3 other.
fn gloss_tier(gloss: Option<&str>, q: &str) -> u8 {
    let Some(g) = gloss else { return 3 };
    let mut best = 3;
    for (i, item) in g.split([';', ',']).enumerate() {
        let item = item.trim().to_lowercase();
        let item = item.strip_prefix("to ").unwrap_or(&item);
        if item == q {
            return if i == 0 { 0 } else { 1 };
        }
        if item.starts_with(q) {
            best = best.min(2);
        }
    }
    best
}

const FTS_CANDIDATES: i64 = 300;

fn search_english(packs: &[&PackDb], q: &str, limit: usize) -> Result<SearchResult> {
    let ql: String = q.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    // (tier, quality-adjusted score) per row; rows can come from several queries.
    let mut hits: HashMap<(String, i64), (u8, f64, ResultRow)> = HashMap::new();
    if let Some(m) = fts_query(q) {
        for p in packs.iter().filter(|p| p.caps.entries_fts) {
            let bm = if p.caps.fts_head { "bm25(entries_fts, 10.0, 1.0)" } else { "bm25(entries_fts)" };
            // The exact-token query keeps short common words ("go") from being crowded out
            // by the prefix query's many matches ("good", "gold", …).
            let exact = m.trim_end_matches('*').to_string();
            // (match expression, order): the gloss-only query is ordered by word frequency so
            // common words with many glosses (가다: "go; travel; head for; …") are never cut off.
            let mut exprs = vec![(exact.clone(), "sc, e.rank"), (m.clone(), "sc, e.rank")];
            if p.caps.fts_head {
                exprs.insert(0, (format!("head : ({exact})"), "e.rank"));
            }
            for (expr, order) in exprs {
                let sql = format!(
                    "SELECT {COLS}, {bm} AS sc FROM entries_fts JOIN entries e ON e.id = entries_fts.rowid \
                     WHERE entries_fts MATCH ?1 ORDER BY {order} LIMIT ?2"
                );
                for r in p.conn.query(&sql, &[expr.as_str().into(), FTS_CANDIDATES.into()])? {
                    let score = r.real(13).unwrap_or(0.0);
                    let row = result_row(&p.id, &r, Some("fts"));
                    let tier = gloss_tier(row.gloss.as_deref(), &ql);
                    hits.entry(key(&row)).or_insert((tier, score, row));
                }
            }
        }
    }
    let mut v: Vec<(u8, f64, ResultRow)> = hits.into_values().collect();
    // Gloss matches: best (tier + source) first, then the most frequent word.
    // Everything else: text relevance, nudged down for weaker sources.
    // Gloss matches are keyed on match tier + source quality, so a learner's-dictionary prefix
    // match ("thank" → "thankful") beats an exact match from a noisy source.
    let key = |h: &(u8, f64, ResultRow)| if h.0 < 3 { (0, h.0 as i64 + quality(&h.2)) } else { (1, 0) };
    v.sort_by(|a, b| {
        key(a).cmp(&key(b)).then_with(|| {
            if a.0 < 3 {
                a.2.rank.cmp(&b.2.rank)
            } else {
                let sa = a.1 * (1.0 - 0.15 * quality(&a.2) as f64);
                let sb = b.1 * (1.0 - 0.15 * quality(&b.2) as f64);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal).then(a.2.rank.cmp(&b.2.rank))
            }
        })
    });
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    push_new(&mut out, &mut seen, v.into_iter().map(|h| h.2).collect());
    out.truncate(limit);
    Ok(SearchResult { mode: "english", rows: out, hanja: None, deconj: None, grammar_hints: None })
}

fn search_hanja(packs: &[&PackDb], q: &str, limit: usize) -> Result<SearchResult> {
    let compact: String = q.chars().filter(|c| is_han(*c)).collect();
    let mut chars: Vec<char> = Vec::new();
    for c in compact.chars() {
        if !chars.contains(&c) {
            chars.push(c);
        }
    }
    chars.truncate(12);
    let mut cards = Vec::new();
    for c in &chars {
        if let Some(h) = hanja_char(packs, &c.to_string())? {
            cards.push(h);
        }
    }
    let mut out: Vec<ResultRow> = Vec::new();
    let mut seen: HashSet<(String, i64)> = HashSet::new();
    let n_chars = compact.chars().count();
    if n_chars >= 2 {
        // whole-word hanja: exact, then prefix, then contained
        let like = format!("{}%", compact);
        let steps: [(&str, Val); 3] = [
            ("e.hanja = ?1", compact.as_str().into()),
            ("e.hanja LIKE ?1", like.as_str().into()),
            ("instr(e.hanja, ?1) > 0", compact.as_str().into()),
        ];
        for (cond, param) in steps {
            let mut rows = Vec::new();
            for p in packs {
                let sql = format!("SELECT {COLS} FROM entries e WHERE {cond} ORDER BY e.rank LIMIT {PREFIX_LIMIT}");
                for r in p.conn.query(&sql, &[param.clone()])? {
                    rows.push(result_row(&p.id, &r, Some("hanja")));
                }
            }
            sort_by_rank(&mut rows);
            push_new(&mut out, &mut seen, rows);
        }
    }
    if let Some(first) = chars.first() {
        if n_chars == 1 {
            let rows = words_with_hanja(packs, &first.to_string(), limit, 0)?;
            push_new(&mut out, &mut seen, rows);
        }
    }
    out.truncate(limit);
    Ok(SearchResult {
        mode: "hanja",
        rows: out,
        hanja: if cards.is_empty() { None } else { Some(cards) },
        deconj: None,
        grammar_hints: None,
    })
}

// ---- entry lookups --------------------------------------------------------------------------

fn entry_from(r: &Row, o: usize) -> Entry {
    let data_text = r.string(o + 13);
    let data = serde_json::from_str(&data_text).unwrap_or(serde_json::json!({ "senses": [] }));
    Entry {
        id: r.int(o).unwrap_or(0),
        headword: r.string(o + 1),
        hw_norm: r.string(o + 2),
        homonym: r.int(o + 3),
        hanja: r.text(o + 4),
        pos: r.text(o + 5),
        pron: r.text(o + 6),
        source: r.string(o + 7),
        lang: r.string(o + 8),
        level: r.int(o + 9),
        rank: r.int(o + 10).unwrap_or(i64::MAX),
        kind: r.string(o + 11),
        gloss: r.text(o + 12),
        data,
    }
}

const ENTRY_COLS: &str =
    "e.id, e.headword, e.hw_norm, e.homonym, e.hanja, e.pos, e.pron, e.source, e.lang, e.level, e.rank, e.kind, e.gloss, e.data";

/// All entries whose normalised headword equals `hw` (every enabled pack).
pub fn entries_by_headword(packs: &[&PackDb], hw: &str) -> Result<Vec<Entry>> {
    let n = norm_headword(hw.trim());
    if n.is_empty() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for p in packs {
        let sql = format!("SELECT {ENTRY_COLS} FROM entries e WHERE e.hw_norm = ?1 ORDER BY e.rank, e.homonym, e.id");
        for r in p.conn.query(&sql, &[n.as_str().into()])? {
            out.push(entry_from(&r, 0));
        }
    }
    out.sort_by_key(|e| e.rank);
    Ok(out)
}

/// One entry by (source, id).
pub fn entry(packs: &[&PackDb], source: &str, id: i64) -> Result<Option<Entry>> {
    for p in packs {
        let sql = format!("SELECT {ENTRY_COLS} FROM entries e WHERE e.source = ?1 AND e.id = ?2");
        if let Some(r) = p.conn.query(&sql, &[source.into(), id.into()])?.first() {
            return Ok(Some(entry_from(r, 0)));
        }
    }
    Ok(None)
}

pub fn hanja_char(packs: &[&PackDb], ch: &str) -> Result<Option<HanjaChar>> {
    let ch: String = ch.chars().take(1).collect();
    if ch.is_empty() {
        return Ok(None);
    }
    for p in packs.iter().filter(|p| p.caps.hanja_chars) {
        let sql = "SELECT ch, readings, meaning_en, strokes, radical, word_count FROM hanja_chars WHERE ch = ?1";
        if let Some(r) = p.conn.query(sql, &[ch.as_str().into()])?.first() {
            return Ok(Some(HanjaChar {
                ch: r.string(0),
                readings: r.text(1),
                meaning_en: r.text(2),
                strokes: r.int(3),
                radical: r.text(4),
                word_count: r.int(5),
            }));
        }
    }
    Ok(None)
}

/// Entries containing the hanja character, most important first.
pub fn words_with_hanja(packs: &[&PackDb], ch: &str, limit: usize, offset: usize) -> Result<Vec<ResultRow>> {
    let ch: String = ch.chars().take(1).collect();
    if ch.is_empty() {
        return Ok(vec![]);
    }
    let mut rows = Vec::new();
    for p in packs.iter().filter(|p| p.caps.hanja_words) {
        let sql = format!(
            "SELECT {COLS} FROM hanja_words h JOIN entries e ON e.id = h.entry_id WHERE h.ch = ?1 ORDER BY e.rank, e.id LIMIT ?2"
        );
        for r in p.conn.query(&sql, &[ch.as_str().into(), ((limit + offset) as i64).into()])? {
            rows.push(result_row(&p.id, &r, Some("hanja")));
        }
    }
    sort_by_rank(&mut rows);
    Ok(rows.into_iter().skip(offset).take(limit).collect())
}

/// Example sentences containing `text`. Uses the trigram FTS index (needs >= 3 characters)
/// and falls back to LIKE for shorter input.
pub fn sentences(packs: &[&PackDb], text: &str, limit: usize) -> Result<Vec<SentenceRow>> {
    let t = text.trim();
    if t.is_empty() || limit == 0 {
        return Ok(vec![]);
    }
    let mut out: Vec<SentenceRow> = Vec::new();
    for p in packs.iter().filter(|p| p.caps.sentences) {
        let rows = if t.chars().count() >= 3 && p.caps.sentences_fts {
            let m = format!("\"{}\"", t.replace('"', "\"\""));
            p.conn.query(
                "SELECT s.ko, s.en, s.source FROM sentences_fts f JOIN sentences s ON s.id = f.rowid \
                 WHERE sentences_fts MATCH ?1 ORDER BY length(s.ko), s.id LIMIT ?2",
                &[m.as_str().into(), (limit as i64).into()],
            )?
        } else {
            let esc = t.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            p.conn.query(
                "SELECT ko, en, source FROM sentences WHERE ko LIKE ?1 ESCAPE '\\' ORDER BY length(ko), id LIMIT ?2",
                &[format!("%{esc}%").into(), (limit as i64).into()],
            )?
        };
        for r in rows {
            out.push(SentenceRow { ko: r.string(0), en: r.text(1), source: r.string(2) });
        }
    }
    out.sort_by_key(|s| s.ko.chars().count());
    out.truncate(limit);
    Ok(out)
}

pub fn grammar_list(packs: &[&PackDb]) -> Result<Vec<GrammarRow>> {
    let mut out = Vec::new();
    for p in packs.iter().filter(|p| p.caps.grammar) {
        let rows = p.conn.query(
            "SELECT id, entry_id, pattern, category, level, summary_en, sort FROM grammar ORDER BY sort, id",
            &[],
        )?;
        for r in rows {
            out.push(GrammarRow {
                id: r.int(0).unwrap_or(0),
                entry_id: r.int(1),
                pattern: r.string(2),
                category: r.string(3),
                level: r.int(4),
                summary_en: r.text(5),
                sort: r.int(6),
            });
        }
    }
    Ok(out)
}

// ---- word of the day ------------------------------------------------------------------------

/// Days since 1970-01-01 for "YYYY-MM-DD" (proleptic Gregorian).
fn days_from_date(date: &str) -> Option<i64> {
    let mut it = date.trim().split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Deterministic pick among krdict level 1-2 word entries for the given date ("YYYY-MM-DD").
pub fn word_of_day(packs: &[&PackDb], date: &str) -> Result<Option<ResultRow>> {
    let seed = match days_from_date(date) {
        Some(d) => d as u64,
        None => date.bytes().fold(1469598103934665603u64, |h, b| (h ^ b as u64).wrapping_mul(1099511628211)),
    };
    for p in packs {
        let cond = "e.source = 'krdict' AND e.level IN (1, 2) AND e.kind = 'word' AND e.gloss IS NOT NULL AND e.gloss != ''";
        let n = p
            .conn
            .query(&format!("SELECT count(*) FROM entries e WHERE {cond}"), &[])?
            .first()
            .and_then(|r| r.int(0))
            .unwrap_or(0);
        if n <= 0 {
            continue;
        }
        let idx = (splitmix64(seed) % n as u64) as i64;
        let sql = format!("SELECT {COLS} FROM entries e WHERE {cond} ORDER BY e.id LIMIT 1 OFFSET ?1");
        if let Some(r) = p.conn.query(&sql, &[idx.into()])?.first() {
            return Ok(Some(result_row(&p.id, r, None)));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
