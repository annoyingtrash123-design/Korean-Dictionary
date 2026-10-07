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
    /// `gloss_terms` (one row per normalised English gloss item, pre-scored) exists.
    pub gloss_terms: bool,
    /// Covering index `entries_hw_rank(hw_norm, rank)` exists.
    pub hw_rank: bool,
    /// `wotd(n, entry_id)` candidate list exists.
    pub wotd: bool,
    /// `hanja_words.rank` exists (covering `(ch, rank, entry_id)` index).
    pub hanja_rank: bool,
    /// `entries_hanja` index exists.
    pub hanja_idx: bool,
    /// `entries.cho` (initial consonants of `hw_norm`) with its `(cho, rank)` index exists.
    pub cho: bool,
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
        let has_index = |name: &str| {
            conn.query("SELECT 1 FROM sqlite_master WHERE type = 'index' AND name = ?1", &[name.into()])
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        };
        let has_hw_rank = has_index("entries_hw_rank");
        let hanja_rank = names.contains("hanja_words")
            && conn
                .query("SELECT name FROM pragma_table_info('hanja_words')", &[])
                .map(|r| r.iter().any(|r| r.string(0) == "rank"))
                .unwrap_or(false);
        let caps = Caps {
            gloss_terms: names.contains("gloss_terms"),
            hw_rank: has_hw_rank,
            wotd: names.contains("wotd"),
            hanja_rank,
            hanja_idx: has_index("entries_hanja"),
            cho: has_index("entries_cho"),
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

/// `INDEXED BY` clause that forces the covering `(hw_norm, rank)` index (the planner would
/// otherwise happily walk `entries_rank` and probe the table for every row).
fn hw_hint(p: &PackDb) -> &'static str {
    if p.caps.hw_rank {
        "INDEXED BY entries_hw_rank"
    } else {
        ""
    }
}

/// Run an `id, rank` query on every pack; the best `limit` as (rank, pack index, id).
fn top_ids(
    packs: &[&PackDb],
    sql: &dyn Fn(&PackDb) -> String,
    params: &[Val],
    limit: usize,
) -> Result<Vec<(i64, usize, i64)>> {
    let mut c = Vec::new();
    for (pi, p) in packs.iter().enumerate() {
        for r in p.conn.query(&sql(p), params)? {
            c.push((r.int(1).unwrap_or(i64::MAX), pi, r.int(0).unwrap_or(0)));
        }
    }
    c.sort();
    c.truncate(limit);
    Ok(c)
}

/// Full result rows for (sort key, pack index, id) candidates, in candidate order.
fn rows_for(packs: &[&PackDb], cands: &[(i64, usize, i64)], via: Option<&'static str>) -> Result<Vec<ResultRow>> {
    let mut by_id: HashMap<(usize, i64), ResultRow> = HashMap::new();
    for (pi, p) in packs.iter().enumerate() {
        let ids: Vec<i64> = cands.iter().filter(|c| c.1 == pi).map(|c| c.2).collect();
        for chunk in ids.chunks(200) {
            let ph = (1..=chunk.len()).map(|i| format!("?{i}")).collect::<Vec<_>>().join(",");
            let sql = format!("SELECT {COLS} FROM entries e WHERE e.id IN ({ph})");
            let params: Vec<Val> = chunk.iter().map(|&i| Val::Int(i)).collect();
            for r in p.conn.query(&sql, &params)? {
                let row = result_row(&p.id, &r, via);
                by_id.insert((pi, row.id), row);
            }
        }
    }
    Ok(cands.iter().filter_map(|c| by_id.remove(&(c.1, c.2))).collect())
}

// ---- search ---------------------------------------------------------------------------------

/// Longest query considered (chars, after normalisation); the rest is ignored.
pub const MAX_QUERY_CHARS: usize = 100;
/// Most FTS tokens used for a text query.
pub const MAX_FTS_TOKENS: usize = 8;

fn is_compat_jamo(c: char) -> bool {
    (0x3130..=0x318F).contains(&(c as u32))
}

/// Query clean-up: NFKC (full-width Latin, half-width forms, CJK compatibility ideographs,
/// decomposed Hangul), except that compatibility jamo are kept as typed (NFKC would turn them
/// into conjoining jamo that match nothing); Latin lower-cased; at most [`MAX_QUERY_CHARS`].
pub fn normalize_query(query: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let mut out = String::new();
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        out.extend(run.nfkc());
        run.clear();
    };
    for c in query.chars().take(MAX_QUERY_CHARS * 4) {
        if is_compat_jamo(c) {
            flush(&mut run, &mut out);
            out.push(c);
        } else {
            run.push(c);
        }
    }
    flush(&mut run, &mut out);
    // single spaces, lower-case, bounded length
    let collapsed = out.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(MAX_QUERY_CHARS).collect::<String>().trim().to_string()
}

/// True for 2-6 initial consonants and nothing else (ㅎㄱ): a choseong query.
fn choseong_query(q: &str) -> bool {
    const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
    let n = q.chars().count();
    (2..=6).contains(&n) && q.chars().all(|c| CHO.contains(c))
}

/// Search across the given (already filtered to enabled) packs.
pub fn search(packs: &[&PackDb], query: &str, limit: Option<usize>) -> Result<SearchResult> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).max(1);
    let q = normalize_query(query);
    if q.is_empty() {
        return Ok(SearchResult { mode: "english", rows: vec![], hanja: None, deconj: None, grammar_hints: None });
    }
    if has_hangul(&q) {
        if choseong_query(&q) {
            if let Some(r) = search_choseong(packs, &q, limit)? {
                return Ok(r);
            }
        }
        // mixed script ("학교 school"): Hangul tokens in Hangul mode, then English for the rest
        let (ko, latin): (Vec<&str>, Vec<&str>) = q.split(' ').partition(|t| has_hangul(t));
        if !latin.is_empty() && latin.iter().any(|t| t.chars().any(char::is_alphanumeric)) {
            let mut r = search_hangul(packs, &ko.join(" "), limit)?;
            if r.rows.len() < limit {
                let en = search_english(packs, &latin.join(" "), limit)?;
                let mut seen: HashSet<(String, i64)> = r.rows.iter().map(key).collect();
                push_new(&mut r.rows, &mut seen, en.rows);
                r.rows.truncate(limit);
            }
            return Ok(r);
        }
        search_hangul(packs, &q, limit)
    } else if has_han(&q) {
        search_hanja(packs, &q, limit)
    } else {
        search_english(packs, &q, limit)
    }
}

/// Initial-consonant search: headwords whose syllable initials equal `q` (needs `entries.cho`).
/// `None` when no pack has it (the query then falls through to the ordinary Hangul search).
fn search_choseong(packs: &[&PackDb], q: &str, limit: usize) -> Result<Option<SearchResult>> {
    let with: Vec<&PackDb> = packs.iter().copied().filter(|p| p.caps.cho).collect();
    if with.is_empty() {
        return Ok(None);
    }
    let sql = |_: &PackDb| format!("SELECT id, rank FROM entries WHERE cho = ?1 ORDER BY rank LIMIT {PREFIX_LIMIT}");
    let cands = top_ids(&with, &sql, &[q.into()], limit.min(PREFIX_LIMIT))?;
    let rows = rows_for(&with, &cands, Some("prefix"))?;
    Ok(Some(SearchResult { mode: "hangul", rows, hanja: None, deconj: None, grammar_hints: None }))
}

fn search_hangul(packs: &[&PackDb], q_raw: &str, limit: usize) -> Result<SearchResult> {
    let q = norm_headword(q_raw);
    let mut out: Vec<ResultRow> = Vec::new();
    let mut seen: HashSet<(String, i64)> = HashSet::new();

    // 1. exact headword (ids and ranks from the covering index, then the full rows)
    let exact_sql = |p: &PackDb| {
        format!("SELECT id, rank FROM entries {} WHERE hw_norm = ?1 ORDER BY rank LIMIT {EXACT_LIMIT}", hw_hint(p))
    };
    let cands = top_ids(packs, &exact_sql, &[q.as_str().into()], limit.max(EXACT_LIMIT))?;
    push_new(&mut out, &mut seen, rows_for(packs, &cands, Some("exact"))?);

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

    // 4. prefix matches. A one-syllable query matches thousands of headwords in every pack;
    // reading them all costs hundreds of page reads, so only the core pack is scanned for it
    // (the other packs still contribute exact / form / conjugation matches above).
    let upper = format!("{q}{}", char::MAX);
    let core_only = q.chars().count() == 1 && packs.iter().any(|p| p.id == "core");
    let prefix_packs: Vec<&PackDb> = packs.iter().copied().filter(|p| !core_only || p.id == "core").collect();
    let prefix_sql = |p: &PackDb| {
        format!(
            "SELECT id, rank FROM entries {} WHERE hw_norm >= ?1 AND hw_norm < ?2 ORDER BY rank LIMIT {PREFIX_LIMIT}",
            hw_hint(p)
        )
    };
    // ids are cheap (covering index); full rows cost a page read each, so fetch only as many
    // as still fit under `limit` after dropping what the earlier steps already found.
    let cands = top_ids(&prefix_packs, &prefix_sql, &[q.as_str().into(), upper.as_str().into()], PREFIX_LIMIT)?;
    let need = limit.saturating_sub(out.len());
    let cands: Vec<_> = cands
        .into_iter()
        .filter(|c| !seen.contains(&(prefix_packs[c.1].id.clone(), c.2)))
        .take(need)
        .collect();
    push_new(&mut out, &mut seen, rows_for(&prefix_packs, &cands, Some("prefix"))?);

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
        .take(MAX_FTS_TOKENS)
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
const GLOSS_EXACT_LIMIT: i64 = 60;
const GLOSS_PREFIX_LIMIT: usize = 40;
/// Rows read (in term order) for a gloss-prefix query before ranking them in Rust.
const GLOSS_PREFIX_SCAN: i64 = 2000;
const FTS_LIMIT: i64 = 40;
/// Gloss matches at which the text-relevance (FTS) tier is skipped.
const FTS_SKIP_ABOVE: usize = 30;
const SCORE_UNIT: i64 = 10_000_000;

/// One English candidate. `group` 0 = gloss match (ordered by `key`), 1 = text relevance
/// (ordered by `score`).
struct Hit {
    group: u8,
    key: i64,
    score: f64,
    row: ResultRow,
}

fn search_english(packs: &[&PackDb], q: &str, limit: usize) -> Result<SearchResult> {
    let ql: String = q.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let qt: &str = match ql.strip_prefix("to ") {
        Some(rest) if !rest.trim().is_empty() => rest.trim(),
        _ => &ql,
    };
    let mut hits: HashMap<(String, i64), Hit> = HashMap::new();

    // 1. gloss tiers from `gloss_terms` (pre-scored: ORDER BY score is the final order)
    let mut tiered: Vec<(i64, usize, i64)> = Vec::new(); // (key, pack index, id)
    for (pi, p) in packs.iter().enumerate().filter(|(_, p)| p.caps.gloss_terms) {
        let mut seen_ids: HashSet<i64> = HashSet::new();
        let rows = p.conn.query(
            &format!("SELECT score, entry_id FROM gloss_terms WHERE term = ?1 ORDER BY score LIMIT {GLOSS_EXACT_LIMIT}"),
            &[qt.into()],
        )?;
        for r in rows {
            let id = r.int(1).unwrap_or(0);
            if seen_ids.insert(id) {
                tiered.push((r.int(0).unwrap_or(i64::MAX), pi, id));
            }
        }
        if qt.chars().count() >= 3 {
            let upper = format!("{qt}{}", char::MAX);
            let rows = p.conn.query(
                &format!(
                    "SELECT tier, score, entry_id FROM gloss_terms WHERE term > ?1 AND term < ?2 LIMIT {GLOSS_PREFIX_SCAN}"
                ),
                &[qt.into(), upper.as_str().into()],
            )?;
            // an item that merely starts with the query is tier 2, whatever its position
            let mut pre: HashMap<i64, i64> = HashMap::new();
            for r in rows {
                let (tier, score, id) = (r.int(0).unwrap_or(0), r.int(1).unwrap_or(i64::MAX), r.int(2).unwrap_or(0));
                if seen_ids.contains(&id) {
                    continue;
                }
                let key = score + (2 - tier) * SCORE_UNIT;
                let e = pre.entry(id).or_insert(key);
                *e = (*e).min(key);
            }
            let mut pre: Vec<(i64, i64)> = pre.into_iter().map(|(id, k)| (k, id)).collect();
            pre.sort();
            pre.truncate(GLOSS_PREFIX_LIMIT);
            tiered.extend(pre.into_iter().map(|(k, id)| (k, pi, id)));
        }
    }
    tiered.sort();
    tiered.truncate(limit);
    for (c, row) in tiered.iter().zip(rows_for(packs, &tiered, Some("fts"))?) {
        hits.insert(key(&row), Hit { group: 0, key: c.0, score: 0.0, row });
    }

    // 2. text relevance (definitions, stemmed forms): rowid + bm25 only, then the rows
    let mut fts: Vec<(f64, usize, i64)> = Vec::new();
    // Only when the gloss tiers left room: bm25 ordering reads a docsize row per match (random
    // page reads), and these rows rank below every gloss match anyway. A very short query
    // is left to the gloss tiers unless they found nothing.
    let want_fts = if qt.chars().count() < 3 { hits.is_empty() } else { hits.len() < FTS_SKIP_ABOVE };
    if want_fts && hits.len() < limit {
        if let Some(m) = fts_query(q) {
            // a short last token as a prefix would match a huge part of the index
            let last_len = m.trim_end_matches('*').rsplit('"').nth(1).map_or(0, |t| t.chars().count());
            let expr = if last_len >= 3 { m.clone() } else { m.trim_end_matches('*').to_string() };
            for (pi, p) in packs.iter().enumerate().filter(|(_, p)| p.caps.entries_fts && p.caps.gloss_terms) {
                let bm = if p.caps.fts_head { "bm25(entries_fts, 10.0, 1.0)" } else { "bm25(entries_fts)" };
                let sql = format!(
                    "SELECT rowid, {bm} FROM entries_fts WHERE entries_fts MATCH ?1 ORDER BY 2 LIMIT {FTS_LIMIT}"
                );
                for r in p.conn.query(&sql, &[expr.as_str().into()])? {
                    let id = r.int(0).unwrap_or(0);
                    if !hits.contains_key(&(p.id.clone(), id)) {
                        fts.push((r.real(1).unwrap_or(0.0), pi, id));
                    }
                }
            }
        }
    }
    if !fts.is_empty() {
        let cands: Vec<(i64, usize, i64)> = fts.iter().map(|c| (0, c.1, c.2)).collect();
        let score_of: HashMap<(usize, i64), f64> = fts.iter().map(|c| ((c.1, c.2), c.0)).collect();
        let pack_idx: HashMap<&str, usize> = packs.iter().enumerate().map(|(i, p)| (p.id.as_str(), i)).collect();
        for row in rows_for(packs, &cands, Some("fts"))? {
            let sc = score_of.get(&(pack_idx[row.pack.as_str()], row.id)).copied().unwrap_or(0.0);
            hits.entry(key(&row)).or_insert(Hit { group: 1, key: 0, score: sc, row });
        }
    }

    // 3. packs without `gloss_terms` (older builds): FTS candidates ranked by gloss tier in Rust
    for p in packs.iter().filter(|p| p.caps.entries_fts && !p.caps.gloss_terms) {
        english_legacy(p, q, &ql, &mut hits)?;
    }

    let mut v: Vec<Hit> = hits.into_values().collect();
    // Gloss matches: best (tier + source) first, then the most frequent word.
    // Everything else: text relevance, nudged down for weaker sources.
    v.sort_by(|a, b| {
        (a.group, a.key).cmp(&(b.group, b.key)).then_with(|| {
            if a.group == 0 {
                a.row.rank.cmp(&b.row.rank)
            } else {
                let sa = a.score * (1.0 - 0.15 * quality(&a.row) as f64);
                let sb = b.score * (1.0 - 0.15 * quality(&b.row) as f64);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal).then(a.row.rank.cmp(&b.row.rank))
            }
        })
    });
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    push_new(&mut out, &mut seen, v.into_iter().map(|h| h.row).collect());
    out.truncate(limit);
    Ok(SearchResult { mode: "english", rows: out, hanja: None, deconj: None, grammar_hints: None })
}

/// The original English search for one pack without `gloss_terms`.
fn english_legacy(p: &PackDb, q: &str, ql: &str, hits: &mut HashMap<(String, i64), Hit>) -> Result<()> {
    let Some(m) = fts_query(q) else { return Ok(()) };
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
            let tier = gloss_tier(row.gloss.as_deref(), ql);
            let hit = if tier < 3 {
                let k = (tier as i64 + quality(&row)) * SCORE_UNIT + row.rank.clamp(0, SCORE_UNIT - 1);
                Hit { group: 0, key: k, score, row }
            } else {
                Hit { group: 1, key: 0, score, row }
            };
            hits.entry(key(&hit.row)).or_insert(hit);
        }
    }
    Ok(())
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
        let upper = format!("{compact}{}", char::MAX);
        let rare = rarest_char(packs, &compact)?;
        for step in 0..3 {
            if step > 0 && out.len() >= PREFIX_LIMIT {
                break;
            }
            let mut rows = Vec::new();
            for p in packs {
                let (sql, params): (String, Vec<Val>) = match step {
                    0 => (
                        format!("SELECT {COLS} FROM entries e WHERE e.hanja = ?1 ORDER BY e.rank LIMIT {PREFIX_LIMIT}"),
                        vec![compact.as_str().into()],
                    ),
                    1 if p.caps.hanja_idx => (
                        format!(
                            "SELECT {COLS} FROM entries e WHERE e.hanja >= ?1 AND e.hanja < ?2 ORDER BY e.rank LIMIT {PREFIX_LIMIT}"
                        ),
                        vec![compact.as_str().into(), upper.as_str().into()],
                    ),
                    1 => (
                        format!("SELECT {COLS} FROM entries e WHERE e.hanja LIKE ?1 ORDER BY e.rank LIMIT {PREFIX_LIMIT}"),
                        vec![format!("{compact}%").into()],
                    ),
                    // contained: walk the rarest character's words in rank order (index-driven)
                    _ if p.caps.hanja_words => (
                        format!(
                            "SELECT {COLS} FROM hanja_words h JOIN entries e ON e.id = h.entry_id \
                             WHERE h.ch = ?1 AND instr(e.hanja, ?2) > 0 ORDER BY h.rank, h.entry_id LIMIT {PREFIX_LIMIT}"
                        ),
                        vec![rare.as_str().into(), compact.as_str().into()],
                    ),
                    _ => (
                        format!("SELECT {COLS} FROM entries e WHERE instr(e.hanja, ?1) > 0 ORDER BY e.rank LIMIT {PREFIX_LIMIT}"),
                        vec![compact.as_str().into()],
                    ),
                };
                // `hanja_rank` packs only: the join above orders by h.rank
                let sql = if step == 2 && p.caps.hanja_words && !p.caps.hanja_rank {
                    sql.replace("h.rank, h.entry_id", "e.rank, e.id")
                } else {
                    sql
                };
                for r in p.conn.query(&sql, &params)? {
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

/// The character of `compact` with the fewest words (cheapest to walk), the first one when unknown.
fn rarest_char(packs: &[&PackDb], compact: &str) -> Result<String> {
    let mut best: Option<(i64, char)> = None;
    for c in compact.chars() {
        let mut n = None;
        for p in packs.iter().filter(|p| p.caps.hanja_chars) {
            if let Some(r) = p.conn.query("SELECT word_count FROM hanja_chars WHERE ch = ?1", &[c.to_string().into()])?.first() {
                n = r.int(0);
                break;
            }
        }
        let n = n.unwrap_or(i64::MAX);
        if best.map_or(true, |(b, _)| n < b) {
            best = Some((n, c));
        }
    }
    Ok(best.map(|b| b.1).or_else(|| compact.chars().next()).unwrap_or(' ').to_string())
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
    let want = limit + offset;
    let hp: Vec<&PackDb> = packs.iter().copied().filter(|p| p.caps.hanja_words).collect();
    // ids and ranks from the covering index of every pack, then only the best rows
    let sql = |p: &PackDb| {
        if p.caps.hanja_rank {
            format!("SELECT entry_id, rank FROM hanja_words WHERE ch = ?1 ORDER BY rank, entry_id LIMIT {want}")
        } else {
            format!(
                "SELECT h.entry_id, e.rank FROM hanja_words h JOIN entries e ON e.id = h.entry_id \
                 WHERE h.ch = ?1 ORDER BY e.rank, e.id LIMIT {want}"
            )
        }
    };
    let cands = top_ids(&hp, &sql, &[ch.as_str().into()], want)?;
    let rows = rows_for(&hp, &cands, Some("hanja"))?;
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
        if p.caps.wotd {
            // precomputed candidate list (n = 0..N-1 in id order): two point lookups
            let n = p
                .conn
                .query("SELECT count(*) FROM wotd", &[])?
                .first()
                .and_then(|r| r.int(0))
                .unwrap_or(0);
            if n <= 0 {
                continue;
            }
            let idx = (splitmix64(seed) % n as u64) as i64;
            let sql = format!("SELECT {COLS} FROM wotd w JOIN entries e ON e.id = w.entry_id WHERE w.n = ?1");
            if let Some(r) = p.conn.query(&sql, &[idx.into()])?.first() {
                return Ok(Some(result_row(&p.id, r, None)));
            }
            continue;
        }
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

/// Background warm-up: step `step` reads one slice of a search index into SQLite's page cache,
/// so that first-time queries don't pay for slow storage reads. Returns false when done.
/// Steps are small (tens of ms cold) so a search queued behind one barely waits.
pub fn warm_step(packs: &[&PackDb], step: usize) -> bool {
    const HW: [&str; 13] = ["", "나", "다", "마", "바", "사", "아", "응", "자", "차", "파", "하", "\u{10FFFF}"];
    const EN: [&str; 7] = ["", "c", "f", "l", "p", "t", "\u{10FFFF}"];
    let mut steps: Vec<(&PackDb, String, String, String)> = Vec::new();
    for p in packs {
        for w in HW.windows(2) {
            steps.push((p, "SELECT count(*) FROM entries INDEXED BY entries_hw_rank WHERE hw_norm >= ?1 AND hw_norm < ?2".into(), w[0].into(), w[1].into()));
        }
        if p.caps.forms {
            for w in [["", "사"], ["사", "\u{10FFFF}"]] {
                steps.push((p, "SELECT count(*) FROM forms INDEXED BY forms_form WHERE form >= ?1 AND form < ?2".into(), w[0].into(), w[1].into()));
            }
        }
        if p.id == "core" {
            for w in EN.windows(2) {
                steps.push((p, "SELECT count(*) FROM gloss_terms WHERE term >= ?1 AND term < ?2".into(), w[0].into(), w[1].into()));
            }
            steps.push((p, "SELECT count(*) FROM hanja_words INDEXED BY hanja_words_ch WHERE ch >= ?1 AND ch < ?2".into(), "".into(), "\u{10FFFF}".into()));
        }
    }
    let Some((p, sql, a, b)) = steps.get(step) else { return false };
    // A missing index (older pack) just means nothing to warm for this step.
    let _ = p.conn.query(sql, &[a.as_str().into(), b.as_str().into()]);
    step + 1 < steps.len()
}
