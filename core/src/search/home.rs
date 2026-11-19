//! The Search tab's start screen: each day's picks from the official curriculum (words and
//! grammar of levels 4-6, table `curriculum`), a proverb and a four-character idiom (the frequency-ranked
//! study lists in table `study_item`, most common first).
//!
//! Every pick is a pure function of the day number (days since 2026-01-01 in the user's local
//! calendar, computed by the app): items are walked in a seeded permutation with a fresh
//! permutation per cycle, so every eligible item is shown once before any repeats, and each kind
//! (words, grammar, idioms) has its own independent cycle.

use super::{splitmix64, PackDb};
use crate::sql::Result;
use serde::Serialize;

pub const WORDS_PER_DAY: usize = 3;
pub const GRAMMAR_PER_DAY: usize = 2;
const WORD_SALT: u64 = 0x1001;
const GRAMMAR_SALT: u64 = 0x2002;
const IDIOM_SALT: u64 = 0x3003;
/// Curriculum levels shown on the home page (5-6: the advanced learner's range).
const LEVELS: &str = "(5, 6)";
/// Grammar cards also cover upper-intermediate 4급 (4-6: upper-intermediate to advanced).
const GRAMMAR_LEVELS: &str = "(4, 5, 6)";
const MAX_EXAMPLES: usize = 3;
const MAX_GLOSS_ITEMS: usize = 3;
/// Longer sentences are fine on a phone card only up to a point; shorter ones are preferred.
const LONG_EXAMPLE: usize = 70;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeExample {
    pub ko: String,
    pub en: Option<String>,
    /// `en` is an AI translation (pipeline `en_ai`).
    pub en_ai: bool,
}

/// A related word of an entry (synonym, antonym, see also...).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeRel {
    #[serde(rename = "type")]
    pub kind: String,
    pub word: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeWord {
    pub source: String,
    pub id: i64,
    pub headword: String,
    pub homonym: Option<i64>,
    pub hanja: Option<String>,
    pub pos: Option<String>,
    /// Official curriculum level (5 or 6).
    pub level: i64,
    pub gloss: String,
    /// Korean definition of the first sense that has one.
    pub ko_def: Option<String>,
    pub related: Vec<HomeRel>,
    pub examples: Vec<HomeExample>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeEntryRef {
    pub source: String,
    pub id: i64,
    pub homonym: Option<i64>,
    pub pos: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeGrammar {
    pub id: i64,
    pub pattern: String,
    /// Official curriculum level (5 or 6).
    pub level: i64,
    pub summary_en: Option<String>,
    pub ko_def: Option<String>,
    pub related: Vec<HomeRel>,
    /// The dictionary entry of the pattern (for opening it), when linked.
    pub entry: Option<HomeEntryRef>,
    pub examples: Vec<HomeExample>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HomeIdiom {
    pub source: String,
    pub id: i64,
    pub headword: String,
    pub homonym: Option<i64>,
    /// The entry's part of speech as the dictionary has it (what a saved item is keyed on).
    pub pos: Option<String>,
    /// "proverb" (속담) | "seongeo" (성어, four-character idiom) | "idiom" (관용구, old packs only)
    pub kind: String,
    pub gloss: String,
    pub ko_def: Option<String>,
    pub related: Vec<HomeRel>,
    pub examples: Vec<HomeExample>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Home {
    pub words: Vec<HomeWord>,
    pub grammar: Vec<HomeGrammar>,
    /// The proverb of the day, then the four-character idiom of the day (one card on old packs).
    pub idioms: Vec<HomeIdiom>,
}

/// The seeded permutation of `0..n` for `cycle` (Fisher-Yates).
fn permutation(n: usize, salt: u64, cycle: u64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    let mut state = splitmix64(salt ^ splitmix64(cycle.wrapping_add(1)));
    for i in (1..n).rev() {
        state = splitmix64(state);
        v.swap(i, (state % (i as u64 + 1)) as usize);
    }
    v
}

/// The `count` item indices (0..n) for stream positions `start..start+count` of a kind: position
/// `s` is item `permutation(cycle = s / n)[s % n]`. An item already taken in this call (a day that
/// straddles two cycles can meet the same item twice) is skipped, so the result has no repeats.
pub fn pick_indices(n: usize, salt: u64, start: u64, count: usize) -> Vec<usize> {
    if n == 0 {
        return vec![];
    }
    let want = count.min(n);
    let mut out: Vec<usize> = Vec::with_capacity(want);
    let mut cache: Option<(u64, Vec<usize>)> = None;
    let mut s = start;
    while out.len() < want {
        let cycle = s / n as u64;
        if cache.as_ref().is_none_or(|(c, _)| *c != cycle) {
            cache = Some((cycle, permutation(n, salt, cycle)));
        }
        let i = cache.as_ref().unwrap().1[(s % n as u64) as usize];
        if !out.contains(&i) {
            out.push(i);
        }
        s += 1;
    }
    out
}

/// Day `day`'s picks for a kind whose stream moves `per_day` items a day.
fn day_indices(n: usize, salt: u64, day: u32, per_day: usize) -> Vec<usize> {
    pick_indices(n, salt, day as u64 * per_day as u64, per_day)
}

pub(super) fn first_items(gloss: &str) -> String {
    let items: Vec<&str> = gloss.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
    items[..items.len().min(MAX_GLOSS_ITEMS)].join("; ")
}

/// Up to `max` example sentences of an entry's data: full sentences before phrases, translated before
/// untranslated, short before long; dialogues (several lines) are left out.
fn examples_of(data: &str, max: usize, allow_phrases: bool) -> Vec<HomeExample> {
    let v: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    let mut found: Vec<(u8, bool, usize, HomeExample)> = Vec::new();
    for s in v.get("senses").and_then(|v| v.as_array()).into_iter().flatten() {
        for ex in s.get("examples").and_then(|v| v.as_array()).into_iter().flatten() {
            // a dialogue keeps one turn per line, exactly as krdict spells it
            let lines = |t: &str| t.split('\n').map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n");
            let ko = lines(ex.get("ko").and_then(|v| v.as_str()).unwrap_or(""));
            let ty = ex.get("type").and_then(|v| v.as_str()).unwrap_or("sentence");
            if ko.is_empty() || ty == "phrase" && !allow_phrases {
                continue;
            }
            let en = ex.get("en").and_then(|v| v.as_str()).map(lines).filter(|s| !s.is_empty());
            let en_ai = en.is_some() && ex.get("en_ai").and_then(|v| v.as_bool()).unwrap_or(false);
            let len = ko.chars().count();
            let class = match ty { "sentence" => 0, "dialogue" => 1, _ => 2 } + if len > LONG_EXAMPLE { 3 } else { 0 };
            found.push((class, en.is_none(), len, HomeExample { ko, en, en_ai }));
        }
    }
    // stable: equal keys keep the dictionary's order
    found.sort_by_key(|(c, no_en, len, _)| (*c, *no_en, *len));
    let mut out: Vec<HomeExample> = Vec::new();
    for (_, _, _, ex) in found {
        if out.len() < max && !out.iter().any(|o| o.ko == ex.ko) {
            out.push(ex);
        }
    }
    out
}

/// The Korean definition of the first sense that has one.
fn ko_def_of(data: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    v.get("senses")?.as_array()?.iter().find_map(|s| s.get("ko_def").and_then(|d| d.as_str())).map(String::from)
}

const MAX_RELATED: usize = 8;

/// Related words of an entry (sense-level synonyms / antonyms / see also first, then its `related` list), no repeats.
fn related_of(data: &str) -> Vec<HomeRel> {
    let v: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    let mut out: Vec<HomeRel> = Vec::new();
    let mut add = |r: &serde_json::Value| {
        let (Some(kind), Some(word)) = (r.get("type").and_then(|t| t.as_str()), r.get("word").and_then(|w| w.as_str())) else { return };
        if !word.trim().is_empty() && out.len() < MAX_RELATED && !out.iter().any(|o| o.word == word) {
            out.push(HomeRel { kind: kind.to_string(), word: word.trim().to_string() });
        }
    };
    for s in v.get("senses").and_then(|s| s.as_array()).into_iter().flatten() {
        for r in s.get("rel").and_then(|r| r.as_array()).into_iter().flatten() {
            add(r);
        }
    }
    for r in v.get("related").and_then(|r| r.as_array()).into_iter().flatten() {
        add(r);
    }
    out
}

fn ids(p: &PackDb, sql: &str) -> Result<Vec<i64>> {
    Ok(p.conn.query(sql, &[])?.iter().filter_map(|r| r.int(0)).collect())
}

/// The word-card rule, shared with the pipeline (which stores it as `curriculum.home_ok`): a krdict
/// level (names such as 신라 / 조선 have none), an English gloss and a full example sentence.
pub fn word_home_ok(level: Option<i64>, gloss: Option<&str>, data: &str) -> bool {
    level.is_some() && gloss.is_some_and(|g| !g.is_empty()) && data.contains("\"examples\":[{") && !examples_of(data, 1, false).is_empty()
}

/// Candidate word ids of the Home page, in stable order (by level, then the official list's order),
/// checked at runtime: reads every candidate entry's JSON (about 2 s on the real pack; packs
/// without `curriculum.home_ok` only).
fn word_pool_runtime(p: &PackDb) -> Result<Vec<i64>> {
    let sql = format!(
        "SELECT c.entry_id, e.data, e.level, e.gloss FROM curriculum c JOIN entries e ON e.id = c.entry_id \
         WHERE c.kind = 'word' AND c.level IN {LEVELS} AND c.entry_id IS NOT NULL \
         GROUP BY c.entry_id ORDER BY MIN(c.level), MIN(c.rank), c.entry_id"
    );
    Ok(p.conn.query(&sql, &[])?.iter().filter(|r| word_home_ok(r.int(2), r.text(3).as_deref(), &r.string(1))).filter_map(|r| r.int(0)).collect())
}

/// The same pool from the precomputed flag: no entry is read.
fn word_pool_flagged(p: &PackDb) -> Result<Vec<i64>> {
    ids(
        p,
        &format!(
            "SELECT entry_id FROM curriculum WHERE kind = 'word' AND level IN {LEVELS} AND entry_id IS NOT NULL AND home_ok = 1 \
             GROUP BY entry_id ORDER BY MIN(level), MIN(rank), entry_id"
        ),
    )
}

fn words(p: &PackDb, day: u32) -> Result<Vec<HomeWord>> {
    let all = if p.caps.curriculum_home_ok { word_pool_flagged(p)? } else { word_pool_runtime(p)? };
    let mut out = Vec::new();
    for i in day_indices(all.len(), WORD_SALT, day, WORDS_PER_DAY) {
        let r = p.conn.query(
            &format!(
                "SELECT e.source, e.id, e.headword, e.homonym, e.hanja, e.pos, e.gloss, e.data, \
                 (SELECT MIN(level) FROM curriculum WHERE kind = 'word' AND entry_id = e.id AND level IN {LEVELS}) \
                 FROM entries e WHERE e.id = ?1"
            ),
            &[all[i].into()],
        )?;
        if let Some(r) = r.first() {
            out.push(HomeWord {
                source: r.string(0),
                id: r.int(1).unwrap_or(0),
                headword: r.string(2),
                homonym: r.int(3),
                hanja: r.text(4),
                pos: r.text(5),
                level: r.int(8).unwrap_or(5),
                gloss: first_items(&r.string(6)),
                ko_def: ko_def_of(&r.string(7)),
                related: related_of(&r.string(7)),
                examples: examples_of(&r.string(7), MAX_EXAMPLES, true),
            });
        }
    }
    Ok(out)
}

/// Grammar patterns of the Home page in stable order: those with an English summary.
fn grammar_pool(p: &PackDb) -> Result<Vec<i64>> {
    let sql = if p.caps.curriculum_home_ok {
        format!(
            "SELECT grammar_id FROM curriculum WHERE kind = 'grammar' AND level IN {GRAMMAR_LEVELS} AND grammar_id IS NOT NULL AND home_ok = 1 \
             GROUP BY grammar_id ORDER BY MIN(level), MIN(rank), grammar_id"
        )
    } else {
        format!(
            "SELECT c.grammar_id FROM curriculum c JOIN grammar g ON g.id = c.grammar_id \
             WHERE c.kind = 'grammar' AND c.level IN {GRAMMAR_LEVELS} AND c.grammar_id IS NOT NULL AND g.summary_en IS NOT NULL AND trim(g.summary_en) != '' \
             GROUP BY c.grammar_id ORDER BY MIN(c.level), MIN(c.rank), c.grammar_id"
        )
    };
    ids(p, &sql)
}

fn grammar(p: &PackDb, day: u32) -> Result<Vec<HomeGrammar>> {
    if !p.caps.grammar {
        return Ok(vec![]);
    }
    let all = grammar_pool(p)?;
    let mut out = Vec::new();
    for i in day_indices(all.len(), GRAMMAR_SALT, day, GRAMMAR_PER_DAY) {
        let r = p.conn.query(
            &format!(
                "SELECT g.id, g.pattern, g.summary_en, \
                 (SELECT MIN(level) FROM curriculum WHERE kind = 'grammar' AND grammar_id = g.id AND level IN {GRAMMAR_LEVELS}), \
                 COALESCE((SELECT entry_id FROM curriculum WHERE kind = 'grammar' AND grammar_id = g.id AND entry_id IS NOT NULL ORDER BY level, rank LIMIT 1), g.entry_id) \
                 FROM grammar g WHERE g.id = ?1"
            ),
            &[all[i].into()],
        )?;
        let Some(r) = r.first() else { continue };
        let mut g = HomeGrammar {
            id: r.int(0).unwrap_or(0),
            pattern: r.string(1),
            level: r.int(3).unwrap_or(5),
            summary_en: r.text(2).filter(|s| !s.trim().is_empty()),
            ko_def: None,
            related: vec![],
            entry: None,
            examples: vec![],
        };
        if let Some(eid) = r.int(4) {
            let e = p.conn.query("SELECT e.source, e.id, e.homonym, e.pos, e.data, e.gloss FROM entries e WHERE e.id = ?1", &[eid.into()])?;
            if let Some(e) = e.first() {
                g.entry = Some(HomeEntryRef { source: e.string(0), id: e.int(1).unwrap_or(0), homonym: e.int(2), pos: e.text(3) });
                g.examples = examples_of(&e.string(4), MAX_EXAMPLES, true);
                g.ko_def = ko_def_of(&e.string(4));
                g.related = related_of(&e.string(4));
                if g.summary_en.is_none() {
                    g.summary_en = e.text(5).filter(|s| !s.trim().is_empty());
                }
            }
        }
        out.push(g);
    }
    Ok(out)
}

/// A headword that is a frame with a slot (말이 ∼지, … 하다), not a fixed expression.
fn has_placeholder(hw: &str) -> bool {
    hw.contains(['∼', '~', '…', '～'])
}

/// The entry has a full example sentence (not only phrases or dialogues).
fn has_sentence(data: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    v.get("senses").and_then(|s| s.as_array()).into_iter().flatten().any(|s| {
        s.get("examples").and_then(|e| e.as_array()).into_iter().flatten().any(|ex| {
            ex.get("type").and_then(|t| t.as_str()) == Some("sentence") && ex.get("ko").and_then(|k| k.as_str()).is_some_and(|k| !k.trim().is_empty())
        })
    })
}

/// The idiom-card rule on an entry's headword and data, shared with the pipeline (`study_item.home_ok`, which
/// also requires a krdict entry of the core pack with an English gloss): a full example sentence and a fixed
/// expression (no slot).
pub fn idiom_home_ok(headword: &str, data: &str) -> bool {
    !has_placeholder(headword) && has_sentence(data)
}

/// Entries fit for an idiom card: an English gloss, a full example sentence (a dialogue does not
/// count) and a fixed expression (no ∼ slot). Rows are (id, headword, data, kind); ids stay in row order.
fn eligible(rows: &[crate::sql::Row]) -> Vec<(i64, String)> {
    let mut out: Vec<(i64, String)> = Vec::new();
    for r in rows {
        let Some(id) = r.int(0) else { continue };
        if idiom_home_ok(&r.string(1), &r.string(2)) && !out.iter().any(|(i, _)| *i == id) {
            out.push((id, r.string(3)));
        }
    }
    out
}

/// The study lists of the two idiom cards, in page order: (lists, card kind). The first list with an
/// eligible item is used (`seongeo300` only for packs built before `seongeo_learner` existed).
const IDIOM_LISTS: [(&[&str], &str, &str); 2] = [(&["sokdam_all"], "sokdam100", "proverb"), (&["seongeo_learner", "seongeo300"], "seongeo300", "seongeo")];

/// One study list's pool: its krdict (core) entries with an English gloss, in rank order. `home_ok` rows of
/// `also` that point at a reviewed `hanppuri` idiom entry (`ext_id` `idiom-...`: the 속담/성어 krdict lacks, which are
/// not in the pool's own list) follow, so those items can be the day's card too.
fn study_pool(p: &PackDb, list: &str, also: &str, kind: &str) -> Result<Vec<(i64, String)>> {
    if p.caps.study_home_ok {
        // eligibility is precomputed (`study_item.home_ok`): no entry is read
        let rows = p.conn.query(
            "SELECT entry_id FROM study_item WHERE pack = 'core' AND home_ok = 1 AND (list = ?1 OR (list = ?2 AND ext_id LIKE 'idiom-%')) ORDER BY (list = ?1) DESC, rank",
            &[list.into(), also.into()],
        )?;
        let mut out: Vec<(i64, String)> = Vec::new();
        for id in rows.iter().filter_map(|r| r.int(0)) {
            if !out.iter().any(|(i, _)| *i == id) {
                out.push((id, kind.to_string()));
            }
        }
        return Ok(out);
    }
    study_pool_runtime(p, list, kind)
}

fn study_pool_runtime(p: &PackDb, list: &str, kind: &str) -> Result<Vec<(i64, String)>> {
    let rows = p.conn.query(
        "SELECT s.entry_id, e.headword, e.data, ?2 FROM study_item s JOIN entries e ON e.id = s.entry_id \
         WHERE s.list = ?1 AND s.pack = 'core' AND e.source IN ('krdict', 'hanppuri') AND e.gloss IS NOT NULL AND trim(e.gloss) != '' \
         ORDER BY s.rank",
        &[list.into(), kind.into()],
    )?;
    Ok(eligible(&rows))
}

/// The card for pool item `i` (an index into `pool`), when there is one.
fn idiom_at(p: &PackDb, pool: &[(i64, String)], i: Option<usize>) -> Result<Option<HomeIdiom>> {
    if let Some(i) = i {
        let (id, kind) = &pool[i];
        let r = p.conn.query("SELECT e.source, e.id, e.headword, e.pos, e.gloss, e.data, e.homonym FROM entries e WHERE e.id = ?1", &[(*id).into()])?;
        let Some(r) = r.first() else { return Ok(None) };
        let data = r.string(5);
        return Ok(Some(HomeIdiom {
            source: r.string(0),
            id: r.int(1).unwrap_or(0),
            headword: r.string(2),
            homonym: r.int(6),
            pos: r.text(3),
            kind: kind.clone(),
            gloss: first_items(&r.string(4)),
            ko_def: ko_def_of(&data),
            related: related_of(&data),
            examples: examples_of(&data, MAX_EXAMPLES, false),
        }));
    }
    Ok(None)
}

/// The day's idiom cards: a proverb (속담, list `sokdam_all`) then a four-character idiom (성어, list
/// `seongeo_learner`). Each list is walked in rank order, most common first: day n shows the n-th eligible item,
/// wrapping around after the last. Packs without `study_item` (or with nothing eligible in it) fall back
/// to one card from krdict's idioms and proverbs on a seeded cycle, which needs the partial index
/// `entries_kind` (a scan would hold the engine for seconds: no card then).
fn idioms(p: &PackDb, day: u32) -> Result<Vec<HomeIdiom>> {
    let mut out = Vec::new();
    if p.caps.study_item {
        for (lists, also, kind) in IDIOM_LISTS {
            let mut pool = Vec::new();
            for list in lists {
                pool = study_pool(p, list, also, kind)?;
                if !pool.is_empty() {
                    break;
                }
            }
            let i = (!pool.is_empty()).then(|| day as usize % pool.len());
            out.extend(idiom_at(p, &pool, i)?);
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    if !p.caps.idiom_idx {
        return Ok(out);
    }
    let rows = p.conn.query(
        "SELECT e.id, e.headword, e.data, e.kind FROM entries e WHERE e.kind IN ('idiom', 'proverb') AND e.source = 'krdict' \
         AND e.gloss IS NOT NULL AND e.gloss != '' AND e.data LIKE '%\"type\":\"sentence\"%' ORDER BY e.id",
        &[],
    )?;
    let pool = eligible(&rows);
    out.extend(idiom_at(p, &pool, day_indices(pool.len(), IDIOM_SALT, day, 1).first().copied())?);
    Ok(out)
}

/// Home page content for day number `day` (days since 2026-01-01). Without a `curriculum` table
/// (older packs) `words` and `grammar` are empty; without the `entries_kind` index there is no idiom.
pub fn home(packs: &[&PackDb], day: u32) -> Result<Home> {
    let mut h = Home::default();
    for p in packs {
        // krdict lives in the core pack
        if p.caps.texts || !(p.id == "core" || p.caps.curriculum) {
            continue;
        }
        if p.caps.curriculum && h.words.is_empty() && h.grammar.is_empty() {
            h.words = words(p, day)?;
            h.grammar = grammar(p, day)?;
        }
        if h.idioms.is_empty() && p.id == "core" {
            h.idioms = idioms(p, day)?;
        }
    }
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;
    use std::collections::HashSet;

    #[test]
    fn permutations_cover_everything_and_differ() {
        let n = 25;
        for cycle in 0..3u64 {
            let p = permutation(n, WORD_SALT, cycle);
            assert_eq!(p.iter().copied().collect::<HashSet<_>>().len(), n);
        }
        assert_ne!(permutation(n, WORD_SALT, 0), permutation(n, WORD_SALT, 1));
        // independent kinds: different salts, different orders
        assert_ne!(permutation(n, WORD_SALT, 0), permutation(n, GRAMMAR_SALT, 0));
    }

    #[test]
    fn deterministic_and_no_repeat_within_a_day() {
        for d in 0..60u32 {
            let a = day_indices(10, WORD_SALT, d, 3);
            assert_eq!(a, day_indices(10, WORD_SALT, d, 3));
            assert_eq!(a.iter().collect::<HashSet<_>>().len(), 3, "day {d}");
        }
        assert_eq!(day_indices(2, WORD_SALT, 5, 3).len(), 2);
        assert!(day_indices(0, WORD_SALT, 5, 3).is_empty());
    }

    #[test]
    fn days_walk_the_whole_pool_before_repeating() {
        // 9 items, 3 a day: days 0..3 show all nine exactly once, then a new order starts
        let mut seen = HashSet::new();
        for d in 0..3 {
            for i in day_indices(9, WORD_SALT, d, 3) {
                assert!(seen.insert(i), "item {i} repeated on day {d}");
            }
        }
        assert_eq!(seen.len(), 9);
        // a single pick a day over 11 days: all 11 once
        let one: HashSet<usize> = (0..11).flat_map(|d| day_indices(11, IDIOM_SALT, d, 1)).collect();
        assert_eq!(one.len(), 11);
    }

    fn pack() -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER, hanja TEXT, pos TEXT, pron TEXT, \
             source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER, rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);
             CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL, category TEXT NOT NULL, level INTEGER, summary_en TEXT, sort INTEGER);",
        )
        .unwrap();
        for i in 1..=8i64 {
            let data = serde_json::json!({"senses": [{"examples": [
                {"ko": format!("구절 {i}"), "type": "phrase"},
                {"ko": format!("문장 {i}입니다."), "en": format!("Sentence {i}."), "type": "sentence", "en_ai": true},
                {"ko": "A: 안녕\nB: 응", "type": "dialogue"}]}]})
            .to_string();
            let gloss = if i == 8 { "".to_string() } else { format!("word {i}; thing; stuff; more") };
            c.query(
                "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'noun', NULL, 'krdict', 'en', 3, ?1, 'word', ?3, ?4)",
                &[i.into(), format!("단어{i}").into(), gloss.into(), data.into()],
            )
            .unwrap();
        }
        // 9: phrases only; 10: no krdict level (a name); both are in the curriculum but never picked
        c.exec(r#"INSERT INTO entries VALUES (9, '단어9', '단어9', NULL, NULL, 'noun', NULL, 'krdict', 'en', 3, 9, 'word', 'nine', '{"senses":[{"examples":[{"ko":"구절","type":"phrase"}]}]}'),
                  (10, '단어10', '단어10', NULL, NULL, 'noun', NULL, 'krdict', 'en', NULL, 10, 'word', 'ten', '{"senses":[{"examples":[{"ko":"문장입니다.","type":"sentence"}]}]}')"#).unwrap();
        for (id, kind, gloss, data) in [
            (20i64, "idiom", "to be nosy", r#"{"senses":[{"ko_def":"참견하다.","examples":[{"ko":"그는 남의 일에 코를 들이민다.","type":"sentence"}]}]}"#),
            (21, "proverb", "a proverb", r#"{"senses":[{"examples":[]}]}"#),
            (22, "idiom", "", r#"{"senses":[{"examples":[{"ko":"x","type":"sentence"}]}]}"#),
            (23, "idiom", "dialogue only", r#"{"senses":[{"examples":[{"ko":"A: 응\nB: 아","type":"dialogue"}]}]}"#),
            (24, "idiom", "although said", r#"{"senses":[{"examples":[{"ko":"말이 사장이지 겨우 번다.","type":"sentence"}]}]}"#),
        ] {
            c.query(
                "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'idiom', NULL, 'krdict', 'en', NULL, ?1, ?3, ?4, ?5)",
                &[id.into(), if id == 24 { "말이 ∼지".to_string() } else { format!("관용{id}") }.into(), kind.into(), gloss.into(), data.into()],
            )
            .unwrap();
        }
        c.exec("CREATE INDEX entries_kind ON entries(kind) WHERE kind IN ('idiom', 'proverb')").unwrap();
        PackDb::new("core", c).unwrap()
    }

    fn with_curriculum(p: PackDb) -> PackDb {
        let c = p.conn;
        c.exec("CREATE TABLE curriculum (kind TEXT NOT NULL, level INTEGER NOT NULL, entry_id INTEGER, grammar_id INTEGER, form TEXT NOT NULL, rank INTEGER NOT NULL)").unwrap();
        for i in 1..=10i64 {
            // 8 has no gloss, 9 only a phrase, 10 no level: not eligible
            c.query("INSERT INTO curriculum VALUES ('word', ?1, ?2, NULL, 'w', ?2)", &[(5 + i % 2).into(), i.into()]).unwrap();
        }
        // a level 4 row and an unmatched row are skipped
        c.exec("INSERT INTO curriculum VALUES ('word', 4, 1, NULL, 'low', 1), ('word', 5, NULL, NULL, 'unmatched', 99)").unwrap();
        for g in 1..=3i64 {
            c.query("INSERT INTO grammar VALUES (?1, ?1, ?2, 'Expressions', 5, ?3, ?1)", &[g.into(), format!("-패턴{g}").into(), format!("summary {g}").into()]).unwrap();
            c.query("INSERT INTO curriculum VALUES ('grammar', 6, ?1, ?1, 'g', ?1)", &[g.into()]).unwrap();
        }
        // grammar also takes 4급 (id 4); a 3급 pattern (id 5) is skipped
        for (g, lvl) in [(4i64, 4i64), (5, 3)] {
            c.query("INSERT INTO grammar VALUES (?1, ?1, ?2, 'Expressions', ?3, ?4, ?1)", &[g.into(), format!("-패턴{g}").into(), lvl.into(), format!("summary {g}").into()]).unwrap();
            c.query("INSERT INTO curriculum VALUES ('grammar', ?2, ?1, ?1, 'g', ?1)", &[g.into(), lvl.into()]).unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn no_curriculum_table_still_gives_idiom() {
        let pk = pack();
        let h = home(&[&pk], 3).unwrap();
        assert!(h.words.is_empty() && h.grammar.is_empty());
        // eligible: only 20 (a sentence example); 21 has no example, 22 no gloss, 23 only a dialogue, 24 is a frame (말이 ∼지)
        for d in 0..10 {
            let v = home(&[&pk], d).unwrap().idioms;
            assert_eq!(v.len(), 1, "day {d}: one card from the old pool");
            let i = &v[0];
            assert_eq!((i.id, i.kind.as_str(), i.gloss.as_str(), i.ko_def.as_deref()), (20, "idiom", "to be nosy", Some("참견하다.")), "day {d}");
        }
    }

    #[test]
    fn no_idiom_without_the_kind_index() {
        let pk = pack();
        pk.conn.exec("DROP INDEX entries_kind").unwrap();
        let pk = PackDb::new("core", pk.conn).unwrap();
        assert!(home(&[&pk], 3).unwrap().idioms.is_empty());
    }

    #[test]
    fn related_words_from_senses_and_related_list() {
        let d = r#"{"senses":[{"rel":[{"type":"synonym","word":"가"},{"type":"antonym","word":"나"}]},{"rel":[{"type":"synonym","word":"가"}]}],"related":[{"type":"derived","word":"다"},{"type":"variant","word":" "}]}"#;
        let r: Vec<(String, String)> = related_of(d).into_iter().map(|r| (r.kind, r.word)).collect();
        assert_eq!(r, vec![("synonym".into(), "가".into()), ("antonym".into(), "나".into()), ("derived".into(), "다".into())]);
        assert!(related_of("{}").is_empty());
        assert_eq!(ko_def_of(r#"{"senses":[{},{"ko_def":"뜻."}]}"#).as_deref(), Some("뜻."));
    }

    #[test]
    fn dialogue_examples_keep_one_turn_per_line() {
        let ex = examples_of(r#"{"senses":[{"examples":[{"ko":"가: 안녕.\n나: 응.참새","en":"A: Hi.\nB: Yes.","type":"dialogue"},{"ko":"문장.","type":"sentence"}]}]}"#, 2, false);
        assert_eq!(ex[0].ko, "문장.");
        assert_eq!(ex[1].ko, "가: 안녕.\n나: 응.참새");
        assert_eq!(ex[1].en.as_deref(), Some("A: Hi.\nB: Yes."));
    }

    #[test]
    fn picks_words_from_levels_5_6_and_grammar_from_4_6() {
        let p = with_curriculum(pack());
        let h = home(&[&p], 0).unwrap();
        assert_eq!((h.words.len(), h.grammar.len()), (3, 2));
        for w in &h.words {
            assert!((1..=7).contains(&w.id), "entries 8-10 are not eligible");
            assert!(w.level == 5 || w.level == 6);
            // gloss capped at three items
            assert_eq!(w.gloss.split("; ").count(), 3);
            // sentence first (translated, AI-labelled), then the dialogue (one turn per line), then phrases
            assert_eq!(w.examples.len(), 3, "sentence, dialogue, phrase");
            assert!(w.examples[0].en_ai && w.examples[0].en.is_some());
            assert!(w.examples[1].en.is_none());
            assert_eq!(w.examples[1].ko, "A: 안녕\nB: 응");
        }
        for g in &h.grammar {
            assert!(g.level == 4 || g.level == 6, "grammar covers 4-6급, not 3급");
            assert!(g.entry.is_some() && !g.examples.is_empty());
        }
        assert_eq!(h, home(&[&p], 0).unwrap());
        // 4급 grammar comes up; 3급 never does
        let mut lv = HashSet::new();
        for d in 0..60 {
            for g in home(&[&p], d).unwrap().grammar {
                lv.insert(g.level);
            }
        }
        assert_eq!(lv, HashSet::from([4, 6]));
    }

    #[test]
    fn seven_words_all_appear_within_three_days() {
        let p = with_curriculum(pack());
        let mut seen = HashSet::new();
        for d in 0..3 {
            for w in home(&[&p], d).unwrap().words {
                seen.insert(w.id);
            }
        }
        // 7 words, 3 a day: 9 slots cover all seven
        assert_eq!(seen.len(), 7);
    }

    /// Adds the study lists: (list, rank, kind, entry id, pack).
    fn with_study_items(p: PackDb, rows: &[(&str, i64, &str, i64, &str)]) -> PackDb {
        let c = p.conn;
        c.exec("CREATE TABLE study_item (list TEXT NOT NULL, rank INTEGER NOT NULL, kind TEXT NOT NULL, title TEXT NOT NULL, entry_id INTEGER, pack TEXT NOT NULL, ext_id TEXT NOT NULL, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, gloss TEXT NOT NULL, PRIMARY KEY (list, rank))").unwrap();
        for (list, rank, kind, eid, pack) in rows {
            c.query(
                "INSERT INTO study_item VALUES (?1, ?2, ?3, 't', ?4, ?5, '1', 'h', 'h', 'g')",
                &[(*list).into(), (*rank).into(), (*kind).into(), (*eid).into(), (*pack).into()],
            )
            .unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    fn study_pack() -> PackDb {
        let pk = pack();
        let ok = r#"{"senses":[{"ko_def":"뜻.","examples":[{"ko":"아주 쉬운 일이었다.","en":"It was easy.","type":"sentence"}]}]}"#;
        for (id, hw, kind, pos, gloss, data) in [
            (30i64, "누워서 떡 먹기", "proverb", "proverb", "a piece of cake", ok),
            (31, "설상가상", "word", "noun", "adding insult to injury", ok),
            (32, "말이 ∼지", "idiom", "idiom", "although said", ok),
            (33, "금상첨화", "word", "noun", "", ok),
            (34, "자포자기", "word", "noun", "desperation", r#"{"senses":[{"examples":[{"ko":"A: 응\nB: 아","type":"dialogue"}]}]}"#),
            (35, "다른목록", "word", "noun", "other list", ok),
            (36, "가는 날이 장날", "proverb", "proverb", "bad timing", ok),
            (37, "우유부단", "word", "noun", "indecisiveness", ok),
        ] {
            pk.conn
                .query(
                    "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, ?4, NULL, 'krdict', 'en', NULL, ?1, ?3, ?5, ?6)",
                    &[id.into(), hw.into(), kind.into(), pos.into(), gloss.into(), data.into()],
                )
                .unwrap();
        }
        with_study_items(
            pk,
            &[
                ("sokdam_all", 1, "sokdam", 30, "core"),
                ("sokdam_all", 2, "sokdam", 32, "core"),     // a frame with a ∼ slot
                ("sokdam_all", 3, "sokdam", 36, "core"),
                ("seongeo_learner", 1, "seongeo", 31, "core"),
                ("seongeo_learner", 2, "seongeo", 33, "core"),    // no English gloss
                ("seongeo_learner", 3, "seongeo", 34, "core"),    // only a dialogue
                ("seongeo_learner", 4, "seongeo", 20, "stdict"),  // entry in another pack
                ("seongeo_learner", 5, "seongeo", 37, "core"),
                ("seongeo100", 1, "seongeo", 35, "core"),    // not one of the two lists
                ("seongeo300", 1, "seongeo", 35, "core"),    // only for packs without seongeo_learner
            ],
        )
    }

    #[test]
    fn idiom_cards_walk_the_study_lists_in_rank_order() {
        let pk = study_pack();
        let day = |d: u32| home(&[&pk], d).unwrap().idioms.iter().map(|i| (i.id, i.kind.clone())).collect::<Vec<_>>();
        let (p, s) = ("proverb".to_string(), "seongeo".to_string());
        // a proverb then a 성어; day n is the n-th eligible item of each list, wrapping around after the last
        assert_eq!(day(0), vec![(30, p.clone()), (31, s.clone())]);
        assert_eq!(day(1), vec![(36, p.clone()), (37, s.clone())]);
        assert_eq!(day(2), vec![(30, p.clone()), (31, s.clone())]);
        assert_eq!(day(5), vec![(36, p), (37, s)]);
        let h = home(&[&pk], 0).unwrap();
        let (a, b) = (&h.idioms[0], &h.idioms[1]);
        assert_eq!((a.headword.as_str(), a.pos.as_deref(), a.gloss.as_str()), ("누워서 떡 먹기", Some("proverb"), "a piece of cake"));
        assert_eq!((b.headword.as_str(), b.pos.as_deref(), b.ko_def.as_deref()), ("설상가상", Some("noun"), Some("뜻.")));
        assert_eq!(a.examples.len(), 1);
        assert_eq!(h, home(&[&pk], 0).unwrap(), "the same picks for the same day");
    }

    #[test]
    fn older_packs_take_the_seongeo_card_from_seongeo300() {
        let pk = study_pack();
        pk.conn.query("DELETE FROM study_item WHERE list = 'seongeo_learner'", &[]).unwrap();
        let v = home(&[&pk], 0).unwrap().idioms;
        assert_eq!(v.iter().map(|i| i.id).collect::<Vec<_>>(), vec![30, 35]);
    }

    #[test]
    fn empty_study_pool_falls_back_to_krdict_idioms() {
        // the table is there but nothing in it is eligible: the old pool (with the frame filter) still gives a card
        let pk = with_study_items(pack(), &[("sokdam_all", 1, "sokdam", 21, "core")]);
        for d in 0..5 {
            let v = home(&[&pk], d).unwrap().idioms;
            assert_eq!(v.iter().map(|i| i.id).collect::<Vec<_>>(), vec![20], "day {d}");
        }
    }

    /// Adds the precomputed flags exactly as the pipeline does (`pipeline/src/home_ok.rs`), through the shared predicates.
    fn flag(p: PackDb) -> PackDb {
        let c = p.conn;
        c.exec("ALTER TABLE curriculum ADD COLUMN home_ok INTEGER NOT NULL DEFAULT 0").unwrap();
        for r in c.query("SELECT c.rowid, e.level, e.gloss, e.data FROM curriculum c JOIN entries e ON e.id = c.entry_id WHERE c.kind = 'word'", &[]).unwrap() {
            if word_home_ok(r.int(1), r.text(2).as_deref(), &r.string(3)) {
                c.query("UPDATE curriculum SET home_ok = 1 WHERE rowid = ?1", &[r.int(0).unwrap().into()]).unwrap();
            }
        }
        c.exec("UPDATE curriculum SET home_ok = 1 WHERE kind = 'grammar' AND grammar_id IN (SELECT id FROM grammar WHERE summary_en IS NOT NULL AND trim(summary_en) != '')").unwrap();
        c.exec("ALTER TABLE study_item ADD COLUMN home_ok INTEGER NOT NULL DEFAULT 0").unwrap();
        let rows = c
            .query(
                "SELECT s.list, s.rank, e.headword, e.data FROM study_item s JOIN entries e ON e.id = s.entry_id \
                 WHERE s.pack = 'core' AND e.source IN ('krdict', 'hanppuri') AND e.gloss IS NOT NULL AND trim(e.gloss) != ''",
                &[],
            )
            .unwrap();
        for r in rows {
            if idiom_home_ok(&r.string(2), &r.string(3)) {
                c.query("UPDATE study_item SET home_ok = 1 WHERE list = ?1 AND rank = ?2", &[r.string(0).into(), r.int(1).unwrap().into()]).unwrap();
            }
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn flagged_selection_is_identical_to_the_runtime_rules() {
        // curriculum + study lists in one pack; the flagged copy must pick exactly what the runtime rules pick, day after day
        let build = || {
            let p = with_curriculum(study_pack());
            // a grammar row whose summary is blank (never eligible), and a duplicate list row
            p.conn.exec("INSERT INTO grammar VALUES (9, 1, '-빈칸', 'Expressions', 5, '  ', 9); INSERT INTO curriculum VALUES ('grammar', 5, 1, 9, 'g', 20)").unwrap();
            p.conn.exec("INSERT INTO study_item VALUES ('sokdam_all', 4, 'sokdam', 't', 30, 'core', '1', 'h', 'h', 'g')").unwrap();
            PackDb::new("core", p.conn).unwrap()
        };
        let (old, new) = (build(), flag(build()));
        assert!(!old.caps.curriculum_home_ok && !old.caps.study_home_ok);
        assert!(new.caps.curriculum_home_ok && new.caps.study_home_ok);
        assert_eq!(word_pool_runtime(&old).unwrap(), word_pool_flagged(&new).unwrap());
        assert_eq!(word_pool_flagged(&new).unwrap().len(), 7);
        assert_eq!(grammar_pool(&old).unwrap(), grammar_pool(&new).unwrap());
        assert_eq!(grammar_pool(&new).unwrap().len(), 4);
        let h = home(&[&new], 0).unwrap();
        assert_eq!((h.words.len(), h.grammar.len(), h.idioms.len()), (3, 2, 2));
        for d in 0..400 {
            assert_eq!(home(&[&old], d).unwrap(), home(&[&new], d).unwrap(), "day {d}");
        }
    }

    #[test]
    fn reviewed_idioms_of_other_lists_join_the_pool() {
        // 속담/성어 that krdict lacks have a reviewed `hanppuri` core entry; the study rows of sokdam100 / seongeo300
        // that point at it (ext_id `idiom-...`) follow the lists the cards walk
        let p = study_pack();
        let ok = r#"{"senses":[{"ko_def":"뜻.","examples":[{"ko":"백문이 불여일견이다.","en":"Seeing is believing.","type":"sentence"}]}]}"#;
        for (id, hw, kind) in [(40i64, "백문이 불여일견", "proverb"), (41, "만장일치", "word")] {
            p.conn
                .query(
                    "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'noun', NULL, 'hanppuri', 'en', NULL, ?1, ?3, 'g', ?4)",
                    &[id.into(), hw.into(), kind.into(), ok.into()],
                )
                .unwrap();
        }
        p.conn.exec("INSERT INTO study_item VALUES ('sokdam100', 7, 'sokdam', 't', 40, 'core', 'idiom-stdict-1', 'h', 'h', 'g'), ('seongeo300', 7, 'seongeo', 't', 41, 'core', 'idiom-stdict-2', 'h', 'h', 'g')").unwrap();
        let p = flag(with_curriculum(PackDb::new("core", p.conn).unwrap()));
        assert_eq!(study_pool(&p, "sokdam_all", "sokdam100", "proverb").unwrap().iter().map(|x| x.0).collect::<Vec<_>>(), vec![30, 36, 40]);
        assert_eq!(study_pool(&p, "seongeo_learner", "seongeo300", "seongeo").unwrap().iter().map(|x| x.0).collect::<Vec<_>>(), vec![31, 37, 41]);
        // the krdict rows of the other lists (35, ext_id '1') stay out
        let h = home(&[&p], 2).unwrap();
        assert_eq!(h.idioms.iter().map(|i| i.id).collect::<Vec<_>>(), vec![40, 41]);
        assert_eq!(h.idioms[0].source, "hanppuri");
    }

    /// Real packs (`$KDICT_PACKS`, else `pipeline/out`): the flagged pools equal the runtime ones, and both timings.
    /// `KDICT_PACKS=pipeline/out cargo test --release -p kdict-core --lib home_flags_match_runtime -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn home_flags_match_runtime() {
        let dir = std::env::var_os("KDICT_PACKS").map(std::path::PathBuf::from).unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../pipeline/out"));
        let path = dir.join("core.sqlite");
        if !path.exists() {
            eprintln!("skipping: no {}", path.display());
            return;
        }
        let open = || PackDb::new("core", Conn::open_path(path.to_str().unwrap(), true).unwrap()).unwrap();
        let p = open();
        assert!(p.caps.curriculum_home_ok && p.caps.study_home_ok, "pack lacks the home_ok columns");
        let t = std::time::Instant::now();
        let old = word_pool_runtime(&p).unwrap();
        let t_old = t.elapsed();
        let t = std::time::Instant::now();
        let new = word_pool_flagged(&p).unwrap();
        eprintln!("word pool: {} ids; runtime {t_old:?}, flagged {:?}", new.len(), t.elapsed());
        assert_eq!(old, new);
        for (lists, _, kind) in IDIOM_LISTS {
            for l in lists {
                assert_eq!(study_pool_runtime(&p, l, kind).unwrap(), study_pool(&p, l, "none", kind).unwrap(), "{l}");
            }
        }
        let mut q = open();
        q.caps.curriculum_home_ok = false;
        q.caps.study_home_ok = false;
        assert_eq!(grammar_pool(&p).unwrap(), grammar_pool(&q).unwrap());
        for d in (0..3000).step_by(7) {
            assert_eq!(home(&[&p], d).unwrap().idioms, home(&[&q], d).unwrap().idioms, "day {d}");
        }
        let t = std::time::Instant::now();
        let h = home(&[&open()], 100).unwrap();
        eprintln!("home (cold, flagged): {:?}; {} words {} grammar {} idioms", t.elapsed(), h.words.len(), h.grammar.len(), h.idioms.len());
        let t = std::time::Instant::now();
        let _ = home(&[&q], 100).unwrap();
        eprintln!("home (warm, runtime): {:?}", t.elapsed());
    }

    #[test]
    fn placeholder_headwords() {
        for hw in ["말이 ∼지", "~ 하다", "… 노릇", "～기"] {
            assert!(has_placeholder(hw), "{hw}");
        }
        assert!(!has_placeholder("누워서 떡 먹기"));
    }
}
