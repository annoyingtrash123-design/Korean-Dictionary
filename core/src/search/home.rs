//! The Search tab's start screen: each day's picks from the official curriculum (words and
//! grammar of levels 5-6, table `curriculum`) and one idiom or proverb.
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
const MAX_EXAMPLES: usize = 2;
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
    /// "idiom" | "proverb"
    pub kind: String,
    pub gloss: String,
    pub ko_def: Option<String>,
    pub example: HomeExample,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Home {
    pub words: Vec<HomeWord>,
    pub grammar: Vec<HomeGrammar>,
    pub idiom: Option<HomeIdiom>,
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

fn first_items(gloss: &str) -> String {
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
            let one_line = |t: &str| t.split('\n').map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" / ");
            let ko = one_line(ex.get("ko").and_then(|v| v.as_str()).unwrap_or(""));
            let ty = ex.get("type").and_then(|v| v.as_str()).unwrap_or("sentence");
            if ko.is_empty() || ty == "phrase" && !allow_phrases {
                continue;
            }
            let en = ex.get("en").and_then(|v| v.as_str()).map(one_line).filter(|s| !s.is_empty());
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

fn ids(p: &PackDb, sql: &str) -> Result<Vec<i64>> {
    Ok(p.conn.query(sql, &[])?.iter().filter_map(|r| r.int(0)).collect())
}

fn words(p: &PackDb, day: u32) -> Result<Vec<HomeWord>> {
    // ordered by level, then the official list's order: a stable index per item. Only entries krdict
    // levels (not names such as 신라 / 조선, which have none) with a glossed full example sentence.
    let sql = format!(
        "SELECT c.entry_id, e.data FROM curriculum c JOIN entries e ON e.id = c.entry_id \
         WHERE c.kind = 'word' AND c.level IN {LEVELS} AND c.entry_id IS NOT NULL AND e.level IS NOT NULL \
         AND e.gloss IS NOT NULL AND e.gloss != '' AND e.data LIKE '%\"examples\":[{{%' \
         GROUP BY c.entry_id ORDER BY MIN(c.level), MIN(c.rank), c.entry_id"
    );
    let all: Vec<i64> = p
        .conn
        .query(&sql, &[])?
        .iter()
        .filter(|r| !examples_of(&r.string(1), 1, false).is_empty())
        .filter_map(|r| r.int(0))
        .collect();
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
                examples: examples_of(&r.string(7), MAX_EXAMPLES, true),
            });
        }
    }
    Ok(out)
}

fn grammar(p: &PackDb, day: u32) -> Result<Vec<HomeGrammar>> {
    if !p.caps.grammar {
        return Ok(vec![]);
    }
    let sql = format!(
        "SELECT c.grammar_id FROM curriculum c JOIN grammar g ON g.id = c.grammar_id \
         WHERE c.kind = 'grammar' AND c.level IN {LEVELS} AND c.grammar_id IS NOT NULL AND g.summary_en IS NOT NULL AND trim(g.summary_en) != '' \
         GROUP BY c.grammar_id ORDER BY MIN(c.level), MIN(c.rank), c.grammar_id"
    );
    let all = ids(p, &sql)?;
    let mut out = Vec::new();
    for i in day_indices(all.len(), GRAMMAR_SALT, day, GRAMMAR_PER_DAY) {
        let r = p.conn.query(
            &format!(
                "SELECT g.id, g.pattern, g.summary_en, \
                 (SELECT MIN(level) FROM curriculum WHERE kind = 'grammar' AND grammar_id = g.id AND level IN {LEVELS}), \
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
            entry: None,
            examples: vec![],
        };
        if let Some(eid) = r.int(4) {
            let e = p.conn.query("SELECT e.source, e.id, e.homonym, e.pos, e.data, e.gloss FROM entries e WHERE e.id = ?1", &[eid.into()])?;
            if let Some(e) = e.first() {
                g.entry = Some(HomeEntryRef { source: e.string(0), id: e.int(1).unwrap_or(0), homonym: e.int(2), pos: e.text(3) });
                g.examples = examples_of(&e.string(4), MAX_EXAMPLES, true);
                if g.summary_en.is_none() {
                    g.summary_en = e.text(5).filter(|s| !s.trim().is_empty());
                }
            }
        }
        out.push(g);
    }
    Ok(out)
}

/// One slice of the idiom pool: the krdict idioms and proverbs with entry id in `lo..lo + span` that have
/// an English gloss and a usable example (a single line or a dialogue, not a bare phrase). The pool is
/// found a slice at a time (the app calls this in the background and keeps the result per pack
/// version): `entries` has no index on `kind`, so one query over the whole table would hold the engine
/// for seconds on a cold phone. `next` is the `lo` of the following slice, `None` after the last.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IdiomSlice {
    pub ids: Vec<i64>,
    pub next: Option<i64>,
}

pub fn idiom_slice(packs: &[&PackDb], lo: i64, span: i64) -> Result<IdiomSlice> {
    let Some(p) = packs.iter().find(|p| !p.caps.texts && p.id == "core") else { return Ok(IdiomSlice::default()) };
    let hi = lo + span.max(1);
    let rows = p.conn.query(
        "SELECT e.id, e.data FROM entries e WHERE e.id >= ?1 AND e.id < ?2 AND e.source = 'krdict' AND e.kind IN ('idiom', 'proverb') \
         AND e.gloss IS NOT NULL AND e.gloss != '' AND e.data LIKE '%\"examples\":[{%' ORDER BY e.id",
        &[lo.into(), hi.into()],
    )?;
    let ids = rows.iter().filter(|r| !examples_of(&r.string(1), 1, false).is_empty()).filter_map(|r| r.int(0)).collect();
    let max = p.conn.query("SELECT max(id) FROM entries", &[])?.first().and_then(|r| r.int(0)).unwrap_or(0);
    Ok(IdiomSlice { ids, next: (hi <= max).then_some(hi) })
}

/// The idiom of the day: day `day`'s pick of the pool (ids from `idiom_slice`, in id order).
fn idiom(p: &PackDb, day: u32, pool: &[i64]) -> Result<Option<HomeIdiom>> {
    for i in day_indices(pool.len(), IDIOM_SALT, day, 1) {
        let r = p.conn.query("SELECT e.source, e.id, e.headword, e.kind, e.gloss, e.data, e.homonym FROM entries e WHERE e.id = ?1", &[pool[i].into()])?;
        let Some(r) = r.first() else { continue };
        let data = r.string(5);
        let Some(example) = examples_of(&data, 1, false).into_iter().next() else { continue };
        let v: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
        let ko_def = v
            .get("senses")
            .and_then(|s| s.as_array())
            .and_then(|s| s.iter().find_map(|s| s.get("ko_def").and_then(|d| d.as_str())))
            .map(String::from);
        return Ok(Some(HomeIdiom {
            source: r.string(0),
            id: r.int(1).unwrap_or(0),
            headword: r.string(2),
            homonym: r.int(6),
            kind: r.string(3),
            gloss: first_items(&r.string(4)),
            ko_def,
            example,
        }));
    }
    Ok(None)
}

/// Home page content for day number `day` (days since 2026-01-01). Without a `curriculum` table
/// (older packs) `words` and `grammar` are empty. `idiom_pool` is the idiom ids found by `idiom_slice`
/// (sorted); while it is empty there is no idiom.
pub fn home(packs: &[&PackDb], day: u32, idiom_pool: &[i64]) -> Result<Home> {
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
        if h.idiom.is_none() && p.id == "core" {
            h.idiom = idiom(p, day, idiom_pool)?;
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

    fn pool(p: &PackDb) -> Vec<i64> {
        let mut out = vec![];
        let mut lo = 0;
        loop {
            // small span: exercises the slicing
            let sl = idiom_slice(&[p], lo, 7).unwrap();
            out.extend(sl.ids);
            match sl.next {
                Some(n) => lo = n,
                None => break out,
            }
        }
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
        ] {
            c.query(
                "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'idiom', NULL, 'krdict', 'en', NULL, ?1, ?3, ?4, ?5)",
                &[id.into(), format!("관용{id}").into(), kind.into(), gloss.into(), data.into()],
            )
            .unwrap();
        }
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
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn no_curriculum_table_still_gives_idiom() {
        let h = home(&[&pack()], 3, &pool(&pack())).unwrap();
        assert!(h.words.is_empty() && h.grammar.is_empty());
        assert!(h.idiom.is_some());
        // eligible: 20 (sentence) and 23 (dialogue, shown on one line); 21 has no example, 22 no gloss.
        // Every day shows one of them, and both appear within two days (one cycle).
        let pk = pack();
        let ids: Vec<i64> = (0..2).map(|d| home(&[&pk], d, &pool(&pk)).unwrap().idiom.unwrap().id).collect();
        assert!(ids.contains(&20) && ids.contains(&23), "{ids:?}");
        let i = (0..2).map(|d| home(&[&pk], d, &pool(&pk)).unwrap().idiom.unwrap()).find(|i| i.id == 20).unwrap();
        assert_eq!((i.kind.as_str(), i.gloss.as_str(), i.ko_def.as_deref()), ("idiom", "to be nosy", Some("참견하다.")));
        let d = (0..2).map(|d| home(&[&pk], d, &pool(&pk)).unwrap().idiom.unwrap()).find(|i| i.id == 23).unwrap();
        assert_eq!(d.example.ko, "A: 응 / B: 아");
    }

    #[test]
    fn picks_from_curriculum_levels_5_6_only() {
        let p = with_curriculum(pack());
        let h = home(&[&p], 0, &pool(&p)).unwrap();
        assert_eq!((h.words.len(), h.grammar.len()), (3, 2));
        for w in &h.words {
            assert!((1..=7).contains(&w.id), "entries 8-10 are not eligible");
            assert!(w.level == 5 || w.level == 6);
            // gloss capped at three items
            assert_eq!(w.gloss.split("; ").count(), 3);
            // sentence first (translated, AI-labelled), then the phrase; dialogue dropped
            assert_eq!(w.examples.len(), 2);
            assert!(w.examples[0].en_ai && w.examples[0].en.is_some());
            assert!(w.examples[1].en.is_none());
            assert!(w.examples.iter().all(|e| !e.ko.contains('\n')));
        }
        for g in &h.grammar {
            assert_eq!(g.level, 6);
            assert!(g.entry.is_some() && !g.examples.is_empty());
        }
        assert_eq!(h, home(&[&p], 0, &pool(&p)).unwrap());
    }

    #[test]
    fn seven_words_all_appear_within_three_days() {
        let p = with_curriculum(pack());
        let mut seen = HashSet::new();
        for d in 0..3 {
            for w in home(&[&p], d, &pool(&p)).unwrap().words {
                seen.insert(w.id);
            }
        }
        // 7 words, 3 a day: 9 slots cover all seven
        assert_eq!(seen.len(), 7);
    }
}
