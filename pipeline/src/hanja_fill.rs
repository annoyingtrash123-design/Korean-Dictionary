//! Hanja fill: 원어 hanja of 표준국어대사전 (stdict) for core entries that have none.
//!
//! Deterministic dictionary data, never invented: a hanja is copied from a stdict entry only when the
//! evidence below says it is the same word. Native words (stdict has no hanja for them) stay empty.
//! Preference is precision over recall: whenever in doubt the entry is left as it is.
//!
//! Applied while the core pack is built (`build_core`, after every entry is inserted and before the
//! hanja index tables are finished, so `hanja_words` and `hanja_chars.word_count` include the fills).
//! Every filled entry carries `data.hanja_src = "stdict"` and `data.hanja_rule` (the app shows a small
//! "표준" marker next to such hanja).
//!
//! # Rules
//! Candidates are the stdict entries (`kind = 'word'`, not affixes) whose headword is the same after
//! `hw_norm` (spaces, `-`, `^`, `·` removed). A stdict hanja is usable when, with the reading marks
//! `▽ ▼` removed, it is all CJK ideographs and has exactly one ideograph per syllable of the headword
//! (all-hangul headwords only). Hanja that spell only part of the word (홈경기 -> 競技) are dropped.
//!
//! * **krdict, `unique` / `homograph`**: the Korean definitions of the krdict entry are compared with
//!   those of every candidate (native ones included, so a native word whose homograph has hanja can
//!   never be taken over). `score` = best over definition pairs of `(Dice + overlap) / 2` on hangul
//!   character bigrams (text in parentheses and quotes removed). A candidate with a conflicting part of
//!   speech scores 0. The best candidate must have usable hanja, and
//!     - `unique` (the headword has one stdict entry): score >= 0.30;
//!     - `homograph` (several stdict entries): score >= 0.40 and >= 0.15 above the runner-up.
//!   A hanja that another core entry of the same headword already has is never given again.
//! * **kengdic, `kengdic`**: kengdic has only English glosses, which cannot be compared with Korean
//!   definitions, so the rule is structural: exactly one stdict entry with usable hanja, no stdict
//!   entry of the same (or unknown) part of speech without hanja (a native homograph), no other core
//!   entry with that headword (a krdict entry would be compared by definitions instead), at least two
//!   syllables, and a part of speech that does not conflict (kengdic `verb`/`adjective` need a stdict
//!   verb/adjective).
//! * **`derived`**: `X하다 / X되다 / X시키다 / X스럽다 / X롭다 / X스레 / X히 / X적` with no hanja, where the
//!   stem `X` is a core word with exactly one hanja (after the fills above), without a native homograph
//!   of `X` and without a hanja alternative; the result is the stem's hanja plus the native suffix as
//!   krdict writes it (工夫하다; 적 is 的). A stdict entry of the derived word that disagrees vetoes it.
//!
//! Never overwritten: an entry that already has hanja. Never filled: headwords that are affixes
//! (leading/trailing `-`), entries marked with an `origin_note` (stdict calls them native or foreign).

use crate::common::{cjk_chars, hw_norm, is_cjk};
use anyhow::Result;
use rusqlite::{params, Connection, OpenFlags};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub const SRC: &str = "stdict";
/// Minimum definition score when the headword has a single stdict entry.
pub const MIN_UNIQUE: f64 = 0.30;
/// Minimum definition score of the best entry of a homograph group, and its lead over the runner-up.
pub const MIN_HOMOGRAPH: f64 = 0.40;
pub const MIN_MARGIN: f64 = 0.15;

/// Native endings krdict writes after hanja (一任하다), with the parts of speech they form.
const SUFFIXES: [(&str, &str); 8] = [
    ("시키다", "V"),
    ("스럽다", "A"),
    ("하다", "VA"),
    ("되다", "VA"),
    ("롭다", "A"),
    ("스레", "D"),
    ("히", "D"),
    ("적", "NTD"), // 적 = 的
];

/// A stdict entry reduced to what the matching needs.
#[derive(Debug, Clone, Default)]
pub struct Std {
    /// Hanja with `▽ ▼` removed; `None` when the entry has none.
    pub hanja: Option<String>,
    pub pos: String,
    pub defs: Vec<String>,
}

/// A core entry that wants hanja.
#[derive(Debug, Clone, Default)]
pub struct Target {
    pub id: i64,
    pub source: String,
    pub headword: String,
    pub pos: String,
    /// krdict: its Korean definitions.
    pub defs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rule {
    Unique,
    Homograph,
    Kengdic,
    Derived,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::Unique => "unique",
            Rule::Homograph => "homograph",
            Rule::Kengdic => "kengdic",
            Rule::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub hanja: String,
    pub rule: Rule,
    /// The stdict definition that was matched (empty when none was used).
    pub stdict_def: String,
    pub score: f64,
}

/// One written fill, for the reports and the QA sample.
#[derive(Debug, Clone)]
pub struct Record {
    pub id: i64,
    pub source: String,
    pub headword: String,
    pub pos: String,
    pub hanja: String,
    pub rule: Rule,
    pub score: f64,
    /// The core entry's definition (krdict Korean, kengdic English gloss).
    pub own_def: String,
    pub stdict_def: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stats {
    pub krdict_unique: usize,
    pub krdict_homograph: usize,
    pub krdict_derived: usize,
    pub kengdic: usize,
    pub kengdic_derived: usize,
}

impl Stats {
    pub fn total(&self) -> usize {
        self.krdict_unique + self.krdict_homograph + self.krdict_derived + self.kengdic + self.kengdic_derived
    }
}

// --- pieces ---------------------------------------------------------------------------------

/// Part-of-speech class: N noun-like, V verb, A adjective, D adverb, T determiner, I interjection, X unknown/other.
pub fn pos_class(pos: &str) -> char {
    match pos {
        "noun" | "pronoun" | "numeral" | "bound noun" => 'N',
        "verb" | "auxiliary verb" => 'V',
        "adjective" | "auxiliary adjective" => 'A',
        "adverb" => 'D',
        "determiner" => 'T',
        "interjection" => 'I',
        _ => 'X',
    }
}

/// Neither side has a known POS, or the classes are equal.
pub fn pos_compatible(a: &str, b: &str) -> bool {
    let (a, b) = (pos_class(a), pos_class(b));
    a == 'X' || b == 'X' || a == b
}

/// `▽ ▼` (reading marks) removed.
pub fn strip_marks(h: &str) -> String {
    h.chars().filter(|c| !matches!(c, '▽' | '▼')).collect()
}

fn is_syllable(c: char) -> bool {
    ('\u{ac00}'..='\u{d7a3}').contains(&c)
}

fn all_syllables(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_syllable)
}

fn all_cjk(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_cjk)
}

/// A stdict hanja that can be copied for a headword: all CJK, one ideograph per syllable.
pub fn usable_hanja(hw_n: &str, hanja: &str) -> bool {
    all_syllables(hw_n) && all_cjk(hanja) && hanja.chars().count() == hw_n.chars().count()
}

fn bigrams(s: &str) -> HashSet<String> {
    let mut depth = 0;
    let letters: Vec<char> = s
        .chars()
        .filter(|&c| match c {
            '(' | '（' | '[' => {
                depth += 1;
                false
            }
            ')' | '）' | ']' => {
                depth = (depth - 1).max(0);
                false
            }
            _ => depth == 0 && is_syllable(c),
        })
        .collect();
    match letters.len() {
        0 => HashSet::new(),
        1 => HashSet::from([letters[0].to_string()]),
        _ => letters.windows(2).map(|w| w.iter().collect()).collect(),
    }
}

/// `(Dice + overlap coefficient) / 2` of the hangul character bigrams of two definitions (0..=1).
pub fn def_similarity(a: &str, b: &str) -> f64 {
    let (x, y) = (bigrams(a), bigrams(b));
    if x.is_empty() || y.is_empty() {
        return 0.0;
    }
    let i = x.intersection(&y).count() as f64;
    0.5 * (2.0 * i / (x.len() + y.len()) as f64 + i / x.len().min(y.len()) as f64)
}

/// Best similarity over all definition pairs.
pub fn defs_score(a: &[String], b: &[String]) -> f64 {
    a.iter().flat_map(|x| b.iter().map(move |y| def_similarity(x, y))).fold(0.0, f64::max)
}

fn first_def(d: &[String]) -> String {
    d.first().cloned().unwrap_or_default()
}

fn best_def(own: &[String], cand: &[String]) -> String {
    let mut best = (-1.0, String::new());
    for y in cand {
        let s = own.iter().map(|x| def_similarity(x, y)).fold(0.0, f64::max);
        if s > best.0 {
            best = (s, y.clone());
        }
    }
    best.1
}

// --- decisions (pure) ---------------------------------------------------------------------------

/// krdict entry: `taken` = hanja of the other core entries of this headword.
pub fn decide_krdict(t: &Target, cands: &[Std], taken: &HashSet<String>) -> Option<Fill> {
    let n = hw_norm(&t.headword);
    if cands.is_empty() || t.defs.is_empty() || !all_syllables(&n) {
        return None;
    }
    let mut scored: Vec<(f64, &Std)> = cands.iter().map(|c| (if pos_compatible(&t.pos, &c.pos) { defs_score(&t.defs, &c.defs) } else { 0.0 }, c)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (best, c) = scored[0];
    let h = c.hanja.as_deref().filter(|h| usable_hanja(&n, h))?;
    if taken.contains(h) {
        return None;
    }
    let second = scored.get(1).map_or(0.0, |s| s.0);
    let (rule, ok) = if cands.len() == 1 { (Rule::Unique, best >= MIN_UNIQUE) } else { (Rule::Homograph, best >= MIN_HOMOGRAPH && best - second >= MIN_MARGIN) };
    ok.then(|| Fill { hanja: h.to_string(), rule, stdict_def: best_def(&t.defs, &c.defs), score: best })
}

/// kengdic-only entry (English glosses only, see the module docs).
pub fn decide_kengdic(t: &Target, cands: &[Std], other_core_hanja: bool) -> Option<Fill> {
    let n = hw_norm(&t.headword);
    if other_core_hanja || n.chars().count() < 2 || !all_syllables(&n) {
        return None;
    }
    let with: Vec<&Std> = cands.iter().filter(|c| c.hanja.as_deref().is_some_and(|h| usable_hanja(&n, h)) && pos_compatible(&t.pos, &c.pos)).collect();
    if with.len() != 1 {
        return None;
    }
    // any other stdict entry of the headword (no hanja, or with partial/odd hanja) of a compatible POS = homograph risk
    let risky = cands.iter().filter(|c| !std::ptr::eq(*c, with[0])).any(|c| pos_compatible(&t.pos, &c.pos) && pos_class(&c.pos) != 'X');
    if risky || cands.iter().filter(|c| c.hanja.as_deref().is_some_and(|h| usable_hanja(&n, h))).count() != 1 {
        return None;
    }
    Some(Fill { hanja: with[0].hanja.clone()?, rule: Rule::Kengdic, stdict_def: first_def(&with[0].defs), score: 1.0 })
}

/// What the core pack says about one stem spelling.
#[derive(Debug, Default, Clone)]
pub struct Stem {
    /// Distinct first hanja (alternatives `/` make a stem ambiguous).
    pub hanja: HashSet<String>,
    /// An entry of this spelling without hanja (a native homograph), or hanja with alternatives.
    pub unclear: bool,
}

/// Split `hw_norm` into (stem, suffix, allowed POS classes).
pub fn split_suffix(n: &str) -> Option<(String, &'static str, &'static str)> {
    for (suf, pos) in SUFFIXES {
        if let Some(stem) = n.strip_suffix(suf) {
            if !stem.is_empty() && all_syllables(stem) {
                return Some((stem.to_string(), suf, pos));
            }
        }
    }
    None
}

/// Derived word: stem hanja + native suffix. `stem`: the core's view of the stem; `own`: stdict entries of the derived word itself.
pub fn decide_derived(t: &Target, stem: &Stem, own: &[Std]) -> Option<Fill> {
    let n = hw_norm(&t.headword);
    let (stem_hw, suf, poses) = split_suffix(&n)?;
    if stem.unclear || stem.hanja.len() != 1 || !pos_class_allowed(&t.pos, poses) {
        return None;
    }
    let sh = stem.hanja.iter().next()?;
    if !all_cjk(sh) || sh.chars().count() != stem_hw.chars().count() {
        return None;
    }
    let suffix_out = if suf == "적" { "的" } else { suf };
    let hanja = format!("{sh}{suffix_out}");
    // a stdict entry of the derived word itself must not say otherwise: its hanja (marks removed) is the same,
    // or it is a native word / has none we can compare
    let mut confirm = String::new();
    for c in own {
        if let Some(h) = &c.hanja {
            if *h == hanja {
                confirm = first_def(&c.defs);
            } else if all_cjk(h) || h.chars().any(is_cjk) {
                return None;
            }
        }
    }
    Some(Fill { hanja, rule: Rule::Derived, stdict_def: confirm, score: 1.0 })
}

fn pos_class_allowed(pos: &str, classes: &str) -> bool {
    let c = pos_class(pos);
    c == 'X' || classes.contains(c)
}

/// Native suffix + CJK stem check for a value that is about to be written (also used by the tests).
pub fn is_valid_fill(h: &str) -> bool {
    let core: String = h.chars().take_while(|&c| is_cjk(c)).collect();
    let tail: String = h.chars().skip(core.chars().count()).collect();
    !core.is_empty() && (tail.is_empty() || SUFFIXES.iter().any(|(s, _)| tail == *s || (*s == "적" && tail == "的")) || tail.chars().all(is_cjk))
}

// --- database side ------------------------------------------------------------------------------

fn load_defs(data: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(data).unwrap_or(Value::Null);
    v.get("senses")
        .and_then(|s| s.as_array())
        .map(|a| a.iter().filter_map(|s| s.get("ko_def").and_then(|d| d.as_str())).filter(|d| !d.is_empty()).take(8).map(|d| crate::common::truncate(d, 300)).collect())
        .unwrap_or_default()
}

struct StdLookup {
    conn: Connection,
    cache: HashMap<String, Vec<Std>>,
}

impl StdLookup {
    fn open(path: &Path) -> Result<StdLookup> {
        Ok(StdLookup { conn: Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?, cache: HashMap::new() })
    }

    fn get(&mut self, n: &str) -> Result<&[Std]> {
        if !self.cache.contains_key(n) {
            let mut st = self.conn.prepare_cached("SELECT headword, hanja, pos, data FROM entries WHERE hw_norm = ?1 AND kind = 'word'")?;
            let rows = st.query_map([n], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, String>(3)?)))?;
            let mut v = Vec::new();
            for r in rows {
                let (hw, h, pos, data) = r?;
                if hw.starts_with('-') || hw.ends_with('-') {
                    continue;
                }
                let native_or_foreign = data.contains("\"origin_note\"");
                let hanja = h.map(|h| strip_marks(&h)).filter(|h| !h.is_empty() && !native_or_foreign);
                v.push(Std { hanja, pos: pos.unwrap_or_default(), defs: load_defs(&data) });
            }
            self.cache.insert(n.to_string(), v);
        }
        Ok(&self.cache[n])
    }
}

fn en_gloss(data: &str) -> String {
    let v: Value = serde_json::from_str(data).unwrap_or(Value::Null);
    v.get("senses").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|s| s.get("gloss").and_then(|g| g.as_str())).take(3).collect::<Vec<_>>().join("; ")).unwrap_or_default()
}

fn write_fill(conn: &Connection, id: i64, f: &Fill) -> Result<()> {
    let data: String = conn.query_row("SELECT data FROM entries WHERE id = ?1", [id], |r| r.get(0))?;
    let mut v: Value = serde_json::from_str(&data)?;
    if let Some(o) = v.as_object_mut() {
        o.insert("hanja_src".into(), SRC.into());
        o.insert("hanja_rule".into(), f.rule.as_str().into());
    }
    let n = conn.execute("UPDATE entries SET hanja = ?1, data = ?2 WHERE id = ?3 AND hanja IS NULL", params![f.hanja, serde_json::to_string(&v)?, id])?;
    if n == 1 {
        for ch in cjk_chars(&f.hanja) {
            conn.prepare_cached("INSERT INTO hanja_words(ch, entry_id) VALUES(?1, ?2)")?.execute(params![ch.to_string(), id])?;
        }
    }
    Ok(())
}

/// Fill the hanja of the (finished, not yet indexed) core pack's krdict / kengdic entries from `stdict`.
pub fn apply(core: &Connection, stdict: &Path) -> Result<(Stats, Vec<Record>)> {
    let mut sd = StdLookup::open(stdict)?;
    let mut stats = Stats::default();
    let mut records: Vec<Record> = Vec::new();

    // headword -> hanja of the core's word entries; entries without hanja that may be filled
    let mut taken: HashMap<String, HashSet<String>> = HashMap::new(); // hw_norm -> hanja (marks removed, first alternative)
    let mut any_core: HashMap<String, usize> = HashMap::new();
    let mut targets: Vec<Target> = Vec::new();
    {
        let mut st = core.prepare("SELECT id, source, headword, hw_norm, pos, hanja, data FROM entries WHERE kind = 'word' AND source IN ('krdict','wikt','kengdic','hanppuri')")?;
        let rows = st.query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, String>(6)?))
        })?;
        for r in rows {
            let (id, source, headword, n, pos, hanja, data) = r?;
            *any_core.entry(n.clone()).or_default() += 1;
            match hanja {
                Some(h) => {
                    taken.entry(n).or_default().insert(strip_marks(h.split('/').next().unwrap_or("")));
                }
                None => {
                    if (source == "krdict" || source == "kengdic") && !headword.starts_with('-') && !headword.ends_with('-') && !data.contains("\"origin_note\"") {
                        let defs = if source == "krdict" { load_defs(&data) } else { vec![en_gloss(&data)] };
                        targets.push(Target { id, source, headword, pos: pos.unwrap_or_default(), defs });
                    }
                }
            }
        }
    }
    let mut krdict_hw: HashSet<String> = HashSet::new();
    {
        let mut st = core.prepare("SELECT hw_norm FROM entries WHERE source IN ('krdict','wikt')")?;
        for r in st.query_map([], |r| r.get::<_, String>(0))? {
            krdict_hw.insert(r?);
        }
    }

    let mut done: HashSet<i64> = HashSet::new();
    let push = |conn: &Connection, t: &Target, f: Fill, stats: &mut Stats, records: &mut Vec<Record>, done: &mut HashSet<i64>| -> Result<()> {
        write_fill(conn, t.id, &f)?;
        done.insert(t.id);
        match (t.source.as_str(), f.rule) {
            ("krdict", Rule::Unique) => stats.krdict_unique += 1,
            ("krdict", Rule::Homograph) => stats.krdict_homograph += 1,
            ("krdict", Rule::Derived) => stats.krdict_derived += 1,
            (_, Rule::Derived) => stats.kengdic_derived += 1,
            _ => stats.kengdic += 1,
        }
        records.push(Record { id: t.id, source: t.source.clone(), headword: t.headword.clone(), pos: t.pos.clone(), hanja: f.hanja, rule: f.rule, score: f.score, own_def: first_def(&t.defs), stdict_def: f.stdict_def });
        Ok(())
    };

    // 1. direct fills
    for t in &targets {
        let n = hw_norm(&t.headword);
        let cands = sd.get(&n)?.to_vec();
        let fill = if t.source == "krdict" {
            let tk = taken.get(&n).cloned().unwrap_or_default();
            decide_krdict(t, &cands, &tk)
        } else {
            let other = taken.contains_key(&n) || krdict_hw.contains(&n) || any_core[&n] > 1;
            decide_kengdic(t, &cands, other)
        };
        if let Some(f) = fill {
            taken.entry(n).or_default().insert(f.hanja.clone());
            push(core, t, f, &mut stats, &mut records, &mut done)?;
        }
    }

    // 2. derived words: stem hanja + native suffix
    let mut stems: HashMap<String, Stem> = HashMap::new();
    {
        let mut st = core.prepare("SELECT hw_norm, hanja FROM entries WHERE kind = 'word' AND source IN ('krdict','wikt','kengdic','hanppuri')")?;
        for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)))? {
            let (n, h) = r?;
            let s = stems.entry(n).or_default();
            match h {
                None => s.unclear = true,
                Some(h) if h.contains('/') => s.unclear = true,
                Some(h) => {
                    s.hanja.insert(strip_marks(&h));
                }
            }
        }
    }
    for t in &targets {
        if done.contains(&t.id) {
            continue;
        }
        let n = hw_norm(&t.headword);
        let Some((stem_hw, _, _)) = split_suffix(&n) else { continue };
        let Some(stem) = stems.get(&stem_hw) else { continue };
        let own = sd.get(&n)?.to_vec();
        if let Some(f) = decide_derived(t, stem, &own) {
            // a one-syllable stem is too ambiguous unless stdict itself spells the derived word that way
            if stem_hw.chars().count() < 2 && f.stdict_def.is_empty() {
                continue;
            }
            if let Some(tk) = taken.get(&n) {
                if tk.contains(&f.hanja) {
                    continue;
                }
            }
            push(core, t, f, &mut stats, &mut records, &mut done)?;
        }
    }
    Ok((stats, records))
}

// --- reports ------------------------------------------------------------------------------------

fn tsv(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// All fills as a TSV (`headword, hanja, source, pos, rule, score, own definition, stdict definition`).
pub fn report_tsv(records: &[Record]) -> String {
    let mut out = String::from("headword\thanja\tsource\tpos\trule\tscore\tcore_def\tstdict_def\n");
    for r in records {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{}\n", r.headword, r.hanja, r.source, r.pos, r.rule.as_str(), r.score, tsv(&crate::common::truncate(&r.own_def, 160)), tsv(&crate::common::truncate(&r.stdict_def, 160))));
    }
    out
}

fn mix(s: &str) -> u64 {
    // FNV-1a: a stable, seed-free "random" order
    s.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

/// A reproducible sample for the independent reviewer: up to `homograph` homograph picks, `unique`
/// unique fills (krdict + kengdic) and `derived` derived ones.
pub fn sample_tsv(records: &[Record], homograph: usize, unique: usize, derived: usize) -> String {
    let pick = |pred: &dyn Fn(&Record) -> bool, n: usize| -> Vec<&Record> {
        let mut v: Vec<&Record> = records.iter().filter(|r| pred(r)).collect();
        v.sort_by_key(|r| mix(&format!("{}{}", r.headword, r.hanja)));
        v.truncate(n);
        v
    };
    let mut out = String::from("headword\thanja\tsource\tcore_definition\tstdict_definition_used\trule\n");
    let mut rows: Vec<&Record> = pick(&|r| r.rule == Rule::Homograph, homograph);
    rows.extend(pick(&|r| matches!(r.rule, Rule::Unique | Rule::Kengdic), unique));
    rows.extend(pick(&|r| r.rule == Rule::Derived, derived));
    for r in rows {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\n", r.headword, r.hanja, r.source, tsv(&crate::common::truncate(&r.own_def, 160)), tsv(&crate::common::truncate(&r.stdict_def, 160)), r.rule.as_str()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn std(h: Option<&str>, pos: &str, def: &str) -> Std {
        Std { hanja: h.map(String::from), pos: pos.into(), defs: vec![def.into()] }
    }
    fn tgt(hw: &str, pos: &str, def: &str) -> Target {
        Target { id: 1, source: "krdict".into(), headword: hw.into(), pos: pos.into(), defs: vec![def.into()] }
    }

    #[test]
    fn similarity_is_high_for_paraphrases_and_zero_for_unrelated() {
        assert!(def_similarity("자신의 이익을 위해 남을 속임.", "자기의 이익을 위하여 남을 속이는 일.") > 0.3);
        assert_eq!(def_similarity("땅 위로 펼쳐진 무한히 넓은 공간.", "(a)"), 0.0);
        assert!(def_similarity("떡살로 눌러 만든 떡.", "현미경으로 관찰하기 위하여 생체 조직의 일부를 얇게 자른 것.") < 0.15);
        // text in parentheses is ignored
        assert_eq!(def_similarity("범죄(犯罪)가 일어나는 비율", "범죄가 일어나는 비율"), 1.0);
    }

    #[test]
    fn homograph_picks_the_entry_whose_definition_agrees() {
        let c = vec![
            std(Some("士氣"), "noun", "의욕이나 자신감 따위로 충만하여 굽힐 줄 모르는 기세."),
            std(Some("沙器"), "noun", "진흙이나 광물 따위의 가루로 빚어서 구운 희고 매끄러운 그릇."),
            std(Some("詐欺"), "noun", "나쁜 꾀로 남을 속임."),
            std(Some("史記"), "noun", "중국 한나라의 사마천이 엮은 역사책."),
        ];
        let t = tgt("사기", "noun", "진흙이나 광물 등의 가루로 빚어서 구운 희고 매끄러운 그릇. 또는 그 재료로 만든 물건.");
        let f = decide_krdict(&t, &c, &HashSet::new()).unwrap();
        assert_eq!((f.hanja.as_str(), f.rule), ("沙器", Rule::Homograph));
        // when that hanja is already on another krdict entry of the headword it is not given twice
        assert!(decide_krdict(&t, &c, &HashSet::from(["沙器".to_string()])).is_none());
        // no clear match: nothing is filled
        assert!(decide_krdict(&tgt("사기", "noun", "하늘에 떠 있는 별."), &c, &HashSet::new()).is_none());
    }

    #[test]
    fn a_native_homograph_blocks_the_fill() {
        // 나이: the native entry matches best, so the hanja homograph (梛移那移) is never taken
        let c = vec![std(None, "noun", "사람이나 동·식물 따위가 세상에 나서 살아온 햇수."), std(Some("梛移那移"), "noun", "돈이나 물건을 잠시 둘러댐.")];
        assert!(decide_krdict(&tgt("나이", "noun", "사람이나 동물 또는 식물 등이 세상에 나서 살아온 햇수."), &c, &HashSet::new()).is_none());
        // 하늘 has no stdict hanja at all
        assert!(decide_krdict(&tgt("하늘", "noun", "땅 위로 펼쳐진 무한히 넓은 공간."), &[std(None, "noun", "땅 위로 끝없이 펼쳐진 무한대의 공간.")], &HashSet::new()).is_none());
    }

    #[test]
    fn partial_or_mixed_hanja_is_never_copied() {
        let t = tgt("홈경기", "noun", "운동 경기에서, 팀의 본거지에서 하는 경기.");
        assert!(decide_krdict(&t, &[std(Some("競技"), "noun", "운동 경기에서, 팀의 근거지에서 하는 경기.")], &HashSet::new()).is_none());
        let t = tgt("장터", "noun", "많은 사람들이 모여 물건을 사고파는 장이 서는 곳.");
        assert!(decide_krdict(&t, &[std(Some("場터"), "noun", "장이 서는 곳.")], &HashSet::new()).is_none());
        // one unique, matching entry is taken
        let t = tgt("범죄율", "noun", "한 나라나 지역에서 일정 기간 동안 범죄가 일어난 비율.");
        assert_eq!(decide_krdict(&t, &[std(Some("犯罪率"), "noun", "범죄가 일어나는 비율.")], &HashSet::new()).unwrap().rule, Rule::Unique);
        // ...unless the POS conflicts
        assert!(decide_krdict(&t, &[std(Some("犯罪率"), "verb", "범죄가 일어나는 비율.")], &HashSet::new()).is_none());
    }

    #[test]
    fn kengdic_needs_exactly_one_hanja_entry_and_no_native_homograph() {
        let k = Target { id: 2, source: "kengdic".into(), headword: "환초".into(), pos: "other".into(), defs: vec!["atoll".into()] };
        assert_eq!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "고리 모양으로 배열된 산호초.")], false).unwrap().hanja, "環礁");
        // two hanja readings: ambiguous
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(Some("還草"), "noun", "b")], false).is_none());
        // a native homograph of the same POS
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "noun", "b")], false).is_none());
        // kengdic's POS is a guess ("other"): a native homograph of any known POS blocks the fill
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "verb", "b")], false).is_none());
        // ...but a native affix-like entry of unknown POS does not
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "other", "b")], false).is_some());
        let v = Target { pos: "verb".into(), ..k.clone() };
        assert!(decide_kengdic(&v, &[std(Some("環礁"), "noun", "a")], false).is_none());
        // another core entry of the headword (krdict) or a one-syllable word
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a")], true).is_none());
        let one = Target { headword: "환".into(), ..k };
        assert!(decide_kengdic(&one, &[std(Some("環"), "noun", "a")], false).is_none());
    }

    fn stem(h: &[&str], unclear: bool) -> Stem {
        Stem { hanja: h.iter().map(|s| s.to_string()).collect(), unclear }
    }

    #[test]
    fn derived_words_take_the_stem_hanja_and_the_native_suffix() {
        let t = tgt("공부하다", "verb", "학문을 배우다.");
        let f = decide_derived(&t, &stem(&["工夫"], false), &[]).unwrap();
        assert_eq!((f.hanja.as_str(), f.rule), ("工夫하다", Rule::Derived));
        assert_eq!(decide_derived(&tgt("불편스럽다", "adjective", ""), &stem(&["不便"], false), &[]).unwrap().hanja, "不便스럽다");
        assert_eq!(decide_derived(&tgt("객관적", "noun", ""), &stem(&["客觀"], false), &[]).unwrap().hanja, "客觀的");
        // several hanja for the stem (사기하다), a native homograph, or the wrong POS: nothing
        assert!(decide_derived(&tgt("사기하다", "verb", ""), &stem(&["士氣", "詐欺"], false), &[]).is_none());
        assert!(decide_derived(&t, &stem(&["工夫"], true), &[]).is_none());
        assert!(decide_derived(&tgt("공부하다", "noun", ""), &stem(&["工夫"], false), &[]).is_none());
        // a stdict entry of the derived word with other hanja vetoes; the same hanja confirms
        assert!(decide_derived(&t, &stem(&["工夫"], false), &[std(Some("功夫하다"), "verb", "x")]).is_none());
        assert_eq!(decide_derived(&t, &stem(&["工夫"], false), &[std(Some("工夫하다"), "verb", "학문을 배우다.")]).unwrap().stdict_def, "학문을 배우다.");
    }

    #[test]
    fn valid_fills_are_cjk_with_an_allowed_suffix() {
        for ok in ["工夫", "工夫하다", "不便스럽다", "客觀的", "一定히"] {
            assert!(is_valid_fill(ok), "{ok}");
        }
        for bad in ["工夫foo", "ㄱ字쇠", "場터", "", "하다"] {
            assert!(!is_valid_fill(bad), "{bad}");
        }
    }
}
