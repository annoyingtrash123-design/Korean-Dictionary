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
//!   speech scores 0. The best candidate must have usable hanja, and score >= 0.30 when the headword has one
//!   stdict entry (`unique`), or >= 0.40 and >= 0.15 above the runner-up when it has several (`homograph`).
//!   A hanja that another core entry of the same headword already has is never given again.
//! * **kengdic, `kengdic`**: kengdic has only English glosses, which cannot be compared with Korean
//!   definitions, so the rule is structural: exactly one stdict entry with usable hanja, no stdict
//!   entry of the same (or unknown) part of speech without hanja (a native homograph), no other core
//!   entry with that headword (a krdict entry would be compared by definitions instead), at least two
//!   syllables, and a part of speech that does not conflict (kengdic `verb`/`adjective` need a stdict
//!   verb/adjective). Structure alone attaches the only stdict hanja to a *different* word of the same
//!   sound (회의자 懷疑者 "sceptic" vs stdict 會意字), so the fill also needs **English corroboration**:
//!   the kengdic gloss must share a content word (stemmed) with (a) an 우리말샘 entry of the same
//!   headword and the same hanja, or (b) the CC-CEDICT entry of the same hanja word. Without either
//!   source (or without an overlap) nothing is filled. `Fill::evidence` says which one agreed.
//! * **`derived`**: `X하다 / X되다 / X시키다 / X스럽다 / X롭다 / X스레 / X히 / X적` with no hanja, where the
//!   stem `X` is a core word with exactly one hanja (after the fills above), without a native homograph
//!   of `X` and without a hanja alternative; the result is the stem's hanja plus the native suffix as
//!   krdict writes it (工夫하다; 적 is 的). A stdict entry of the derived word that disagrees vetoes it,
//!   and so does a native stdict entry of the same part of speech (타박하다). A stem is unclear when
//!   stdict's best match for the stem's own definitions is not the entry carrying the stem's hanja (구경
//!   is native, krdict's 究竟 is a wrong hanja), and a stem whose hanja was itself filled by the kengdic
//!   rule is never used (재분 才分 -> 재분하다).
//!
//! Never overwritten: an entry that already has hanja. Never filled: headwords that are affixes
//! (leading/trailing `-`), entries marked with an `origin_note` (stdict calls them native or foreign).

use crate::common::{cjk_chars, hw_norm, is_cjk};
use anyhow::Result;
use rusqlite::{params, Connection, OpenFlags};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    /// English glosses (krdict, kengdic).
    pub en: Vec<String>,
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
    /// kengdic rule: which English source agreed (`opendict`, `cedict` or `opendict+cedict`); empty otherwise.
    pub evidence: &'static str,
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
    pub evidence: &'static str,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stats {
    pub krdict_unique: usize,
    pub krdict_homograph: usize,
    pub krdict_derived: usize,
    pub kengdic: usize,
    pub kengdic_derived: usize,
    /// kengdic fills by evidence type (they add up to `kengdic`; `both` = agreed by both sources).
    pub kengdic_opendict: usize,
    pub kengdic_cedict: usize,
    pub kengdic_both: usize,
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

// --- English evidence for the kengdic rule ---------------------------------------------------------

/// English glosses per hanja word from sources that are independent of kengdic.
#[derive(Debug, Default, Clone)]
pub struct Evidence {
    /// 우리말샘: hw_norm -> [(hanja, English glosses)].
    pub opendict: HashMap<String, Vec<(String, Vec<String>)>>,
    /// CC-CEDICT: traditional and simplified spelling -> English glosses.
    pub cedict: HashMap<String, Vec<String>>,
}

impl Evidence {
    /// Read the raw 우리말샘 XML files and the CC-CEDICT file (either may be absent).
    pub fn load(opendict: &[PathBuf], cedict: Option<&Path>) -> Result<Evidence> {
        let mut ev = Evidence::default();
        for p in opendict {
            crate::opendict::parse_file(p, |e| {
                let Some(h) = e.hanja.as_deref().map(strip_marks).filter(|h| all_cjk(h)) else { return Ok(()) };
                let gl = all_glosses(&e.data);
                if !gl.is_empty() {
                    ev.opendict.entry(hw_norm(&e.headword)).or_default().push((h, gl));
                }
                Ok(())
            })?;
        }
        if let Some(p) = cedict {
            crate::cedict::read_lines(p, |l| {
                for k in [l.trad.clone(), l.simp.clone()] {
                    ev.cedict.entry(k).or_default().extend(l.glosses.iter().cloned());
                }
                Ok(())
            })?;
        }
        Ok(ev)
    }

    /// Which sources have an English gloss for `hanja` (the word `n`) that shares a content word with `en`.
    pub fn corroboration(&self, n: &str, hanja: &str, en: &[String]) -> &'static str {
        let keys = gloss_keys(en);
        if keys.is_empty() {
            return "";
        }
        let od = self.opendict.get(n).is_some_and(|v| v.iter().any(|(h, g)| h == hanja && gloss_keys(g).intersection(&keys).next().is_some()));
        let cd = self.cedict.get(hanja).is_some_and(|g| gloss_keys(g).intersection(&keys).next().is_some());
        match (od, cd) {
            (true, true) => "opendict+cedict",
            (true, false) => "opendict",
            (false, true) => "cedict",
            _ => "",
        }
    }
}

fn all_glosses(data: &Value) -> Vec<String> {
    data.get("senses").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|s| s.get("gloss").and_then(|g| g.as_str())).map(String::from).collect()).unwrap_or_default()
}

const EN_STOP: [&str; 40] = [
    "the", "and", "for", "with", "from", "that", "this", "its", "one", "sth", "something", "someone", "person", "thing", "being", "state", "kind", "type", "make", "made", "does", "used", "use", "act", "has", "not", "any", "esp", "see", "also", "variant", "old", "name", "etc", "lit", "abbr", "way", "form", "cl", "sb",
];

/// Content words of English glosses, lowercased, plural/-ing/-ed/-ion stripped and cut to 5 letters
/// (so "conscription" ~ "conscript", "bribery" ~ "bribe"). Words of 3 letters count (tub, oil).
pub fn gloss_keys(v: &[String]) -> HashSet<String> {
    v.iter()
        .flat_map(|g| g.split(|c: char| !c.is_ascii_alphabetic()))
        .map(str::to_ascii_lowercase)
        .filter(|w| w.len() >= 3 && !EN_STOP.contains(&w.as_str()))
        .map(|mut w| {
            for suf in ["ies", "ing", "ion", "ed", "es", "s"] {
                if w.len() > suf.len() + 3 && w.ends_with(suf) {
                    w.truncate(w.len() - suf.len());
                    break;
                }
            }
            w.chars().take(5).collect::<String>()
        })
        .collect()
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
    ok.then(|| Fill { hanja: h.to_string(), rule, stdict_def: best_def(&t.defs, &c.defs), score: best, evidence: "" })
}

/// kengdic-only entry (English glosses only, see the module docs).
pub fn decide_kengdic(t: &Target, cands: &[Std], other_core_hanja: bool, ev: &Evidence) -> Option<Fill> {
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
    let hanja = with[0].hanja.clone()?;
    // a person noun in -자 (者) whose only stdict homograph ends in 子/字 is another word: 추적자 "pursuer" is
    // 追跡者, stdict only has 追跡子 "tracer"; 회의자 "unbeliever" vs 會意字 (both caught in review)
    if person_ja_mismatch(&n, &hanja, &t.en.join("; ")) {
        return None;
    }
    let evidence = ev.corroboration(&n, &hanja, &t.en);
    if evidence.is_empty() {
        return None;
    }
    Some(Fill { hanja, rule: Rule::Kengdic, stdict_def: first_def(&with[0].defs), score: 1.0, evidence })
}

/// The headword ends in 자, the stdict hanja ends in 子 or 字, and the English gloss names a person
/// (one who …, -er/-or/-ist/-ant/-ent/-ee, person/man/woman): the gloss means 者, not the stdict word.
pub fn person_ja_mismatch(headword: &str, hanja: &str, en: &str) -> bool {
    if !headword.ends_with('자') || !(hanja.ends_with('子') || hanja.ends_with('字')) {
        return false;
    }
    let en = en.to_lowercase();
    if en.contains("one who") || en.contains("person") || en.contains(" man") || en.contains("woman") {
        return true;
    }
    en.split(|c: char| !c.is_ascii_alphabetic())
        .filter(|w| w.len() >= 5)
        .any(|w| ["er", "or", "ist", "ant", "ent", "ee"].iter().any(|s| w.ends_with(s)))
}

/// What the core pack says about one stem spelling.
#[derive(Debug, Default, Clone)]
pub struct Stem {
    /// Distinct first hanja (alternatives `/` make a stem ambiguous).
    pub hanja: HashSet<String>,
    /// An entry of this spelling without hanja (a native homograph), or hanja with alternatives.
    pub unclear: bool,
    /// Korean definitions and English glosses of the entries that carry the hanja (the evidence that
    /// a derived word is built on this stem).
    pub defs: Vec<String>,
    pub en: Vec<String>,
}

/// Minimum score for a stdict entry to count as "the stem's own meaning".
pub const MIN_STEM_MATCH: f64 = 0.30;

/// The core's hanja for the stem is not backed by stdict: the stdict entry of the stem's spelling that
/// best matches the stem's definitions (score >= `MIN_STEM_MATCH`) is native or has other hanja. 구경
/// "sightseeing" matches native 구경 (krdict says 究竟), so 구경시키다 must not become 究竟시키다.
pub fn stem_contradicted(stem: &Stem, cands: &[Std]) -> bool {
    let Some(h) = stem.hanja.iter().next() else { return false };
    let best = cands.iter().map(|c| (defs_score(&stem.defs, &c.defs), c)).max_by(|a, b| a.0.total_cmp(&b.0));
    matches!(best, Some((sc, c)) if sc >= MIN_STEM_MATCH && c.hanja.as_deref() != Some(h.as_str()))
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
    // What stdict says about the derived word itself: the same mixed hanja confirms, other hanja vetoes, a
    // native entry of the same POS vetoes.
    let mut confirm = String::new();
    for c in own {
        match &c.hanja {
            Some(h) if *h == hanja => confirm = first_def(&c.defs),
            Some(h) if h.chars().any(is_cjk) => return None,
            // a native entry of the same POS (타박하다): the derived word may well be native
            _ if pos_compatible(&t.pos, &c.pos) && pos_class(&c.pos) != 'X' => return None,
            _ => {}
        }
    }
    if confirm.is_empty() {
        // the derived word must mean something built on the stem: Korean definitions alike, or shared English words
        let ko = defs_score(&t.defs, &stem.defs);
        let en = shared_words(&t.en, &stem.en);
        if ko < MIN_STEM_DEFS && en == 0 {
            return None;
        }
    }
    Some(Fill { hanja, rule: Rule::Derived, stdict_def: confirm, score: 1.0, evidence: "" })
}

/// Minimum definition similarity between a derived word and its stem when stdict does not confirm the derivation.
pub const MIN_STEM_DEFS: f64 = 0.25;

const STOP: [&str; 14] = ["that", "with", "from", "have", "this", "something", "someone", "being", "state", "person", "thing", "make", "made", "does"];

fn word_keys(v: &[String]) -> HashSet<String> {
    v.iter()
        .flat_map(|g| g.split(|c: char| !c.is_ascii_alphabetic()))
        .map(str::to_ascii_lowercase)
        .filter(|w| w.len() >= 4 && !STOP.contains(&w.as_str()))
        .map(|w| w.chars().take(5).collect::<String>())
        .collect()
}

/// English words (4+ letters, cut to 5) the two gloss lists share.
pub fn shared_words(a: &[String], b: &[String]) -> usize {
    word_keys(a).intersection(&word_keys(b)).count()
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

fn en_glosses(data: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(data).unwrap_or(Value::Null);
    v.get("senses").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|s| s.get("gloss").and_then(|g| g.as_str())).take(4).map(String::from).collect()).unwrap_or_default()
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
pub fn apply(core: &Connection, stdict: &Path, ev: &Evidence) -> Result<(Stats, Vec<Record>)> {
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
                        let defs = if source == "krdict" { load_defs(&data) } else { Vec::new() };
                        targets.push(Target { id, source, headword, pos: pos.unwrap_or_default(), defs, en: en_glosses(&data) });
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
            _ => {
                stats.kengdic += 1;
                match f.evidence {
                    "opendict" => stats.kengdic_opendict += 1,
                    "cedict" => stats.kengdic_cedict += 1,
                    _ => stats.kengdic_both += 1,
                }
            }
        }
        records.push(Record { id: t.id, source: t.source.clone(), headword: t.headword.clone(), pos: t.pos.clone(), hanja: f.hanja, rule: f.rule, score: f.score, evidence: f.evidence, own_def: if t.defs.is_empty() { t.en.join("; ") } else { first_def(&t.defs) }, stdict_def: f.stdict_def });
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
            decide_kengdic(t, &cands, other, ev)
        };
        if let Some(f) = fill {
            taken.entry(n).or_default().insert(f.hanja.clone());
            push(core, t, f, &mut stats, &mut records, &mut done)?;
        }
    }

    // 2. derived words: stem hanja + native suffix
    let mut stems: HashMap<String, Stem> = HashMap::new();
    {
        let mut st = core.prepare("SELECT hw_norm, hanja, data FROM entries WHERE kind = 'word' AND source IN ('krdict','wikt','kengdic','hanppuri')")?;
        for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?, r.get::<_, String>(2)?)))? {
            let (n, h, data) = r?;
            let s = stems.entry(n).or_default();
            match h {
                None => s.unclear = true,
                // a hanja the kengdic rule filled (this run or an earlier one) is only a guess at the stem
                Some(_) if data.contains("\"hanja_rule\":\"kengdic\"") => s.unclear = true,
                Some(h) if h.contains('/') => s.unclear = true,
                Some(h) => {
                    s.hanja.insert(strip_marks(&h));
                    s.defs.extend(load_defs(&data));
                    s.en.extend(en_glosses(&data));
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
        if !stem.unclear && stem.hanja.len() == 1 && stem_contradicted(stem, sd.get(&stem_hw)?) {
            continue;
        }
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

// --- suspect hanja already in core --------------------------------------------------------------

/// A krdict entry whose own hanja looks wrong: its definition matches a native stdict entry of the
/// same spelling (`native_score` >= 0.6) and the stdict entry carrying that hanja (if any) hardly
/// at all (`hanja_score` < 0.3). For a separate check; nothing is changed.
#[derive(Debug, Clone)]
pub struct Suspect {
    pub headword: String,
    pub hanja: String,
    pub pos: String,
    pub core_def: String,
    pub native_def: String,
    pub native_score: f64,
    /// The stdict entry with the same hanja (empty: stdict has none).
    pub hanja_def: String,
    pub hanja_score: f64,
}

pub const SUSPECT_NATIVE_MIN: f64 = 0.6;
pub const SUSPECT_HANJA_MAX: f64 = 0.3;

/// The reviewer's test, pure: `defs` = the krdict entry's definitions, `hanja` its hanja (first alternative, marks removed).
pub fn suspect_of(defs: &[String], pos: &str, hanja: &str, cands: &[Std]) -> Option<(f64, String, f64, String)> {
    let score = |c: &Std| if pos_compatible(pos, &c.pos) { defs_score(defs, &c.defs) } else { 0.0 };
    let native = cands.iter().filter(|c| c.hanja.is_none()).map(|c| (score(c), c)).max_by(|a, b| a.0.total_cmp(&b.0))?;
    if native.0 < SUSPECT_NATIVE_MIN {
        return None;
    }
    let with = cands.iter().filter(|c| c.hanja.as_deref() == Some(hanja)).map(|c| (score(c), c)).max_by(|a, b| a.0.total_cmp(&b.0));
    let (hs, hd) = with.map_or((0.0, String::new()), |(s, c)| (s, best_def(defs, &c.defs)));
    // the native entry must also beat every entry with other hanja that scores as well as it
    (hs < SUSPECT_HANJA_MAX).then(|| (native.0, best_def(defs, &native.1.defs), hs, hd))
}

/// Scan every krdict entry that has its own hanja (not a fill) with [`suspect_of`]. Call before `apply`.
pub fn suspect_core_hanja(core: &Connection, stdict: &Path) -> Result<Vec<Suspect>> {
    let mut sd = StdLookup::open(stdict)?;
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    {
        let mut st = core.prepare("SELECT headword, hw_norm, hanja, pos, data FROM entries WHERE kind = 'word' AND source = 'krdict' AND hanja IS NOT NULL")?;
        for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?.unwrap_or_default(), r.get::<_, String>(4)?)))? {
            rows.push(r?);
        }
    }
    let mut out = Vec::new();
    for (hw, n, h, pos, data) in rows {
        if data.contains("\"hanja_src\"") || h.contains('/') {
            continue;
        }
        let h = strip_marks(&h);
        if !usable_hanja(&n, &h) {
            continue;
        }
        let defs = load_defs(&data);
        if defs.is_empty() {
            continue;
        }
        if let Some((ns, nd, hs, hd)) = suspect_of(&defs, &pos, &h, sd.get(&n)?) {
            out.push(Suspect { headword: hw, hanja: h, pos, core_def: first_def(&defs), native_def: nd, native_score: ns, hanja_def: hd, hanja_score: hs });
        }
    }
    out.sort_by(|a, b| b.native_score.total_cmp(&a.native_score).then(a.headword.cmp(&b.headword)));
    Ok(out)
}

pub fn suspects_tsv(v: &[Suspect]) -> String {
    let mut out = String::from("headword\thanja\tpos\tcore_def\tnative_stdict_def\tnative_score\tstdict_def_with_this_hanja\thanja_score\n");
    for r in v {
        let t = |s: &str| tsv(&crate::common::truncate(s, 160));
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{:.2}\n", r.headword, r.hanja, r.pos, t(&r.core_def), t(&r.native_def), r.native_score, t(&r.hanja_def), r.hanja_score));
    }
    out
}

// --- reports ------------------------------------------------------------------------------------

fn tsv(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

/// All fills as a TSV (`headword, hanja, source, pos, rule, score, own definition, stdict definition`).
pub fn report_tsv(records: &[Record]) -> String {
    let mut out = String::from("headword\thanja\tsource\tpos\trule\tscore\tcore_def\tstdict_def\tevidence\n");
    for r in records {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{:.2}\t{}\t{}\t{}\n", r.headword, r.hanja, r.source, r.pos, r.rule.as_str(), r.score, tsv(&crate::common::truncate(&r.own_def, 160)), tsv(&crate::common::truncate(&r.stdict_def, 160)), r.evidence));
    }
    out
}

fn mix(s: &str) -> u64 {
    // FNV-1a: a stable, seed-free "random" order
    s.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

/// A reproducible sample of `total` fills for the independent reviewer: up to `homograph` homograph picks and
/// `derived` derived ones (as many as exist), the rest unique fills (krdict + kengdic).
pub fn sample_tsv(records: &[Record], total: usize, homograph: usize, derived: usize, seed: &str) -> String {
    let pick = |pred: &dyn Fn(&Record) -> bool, n: usize| -> Vec<&Record> {
        let mut v: Vec<&Record> = records.iter().filter(|r| pred(r)).collect();
        // krdict fills first (there are few), then a stable pseudo-random order
        v.sort_by_key(|r| (r.source != "krdict", mix(&format!("{seed}{}{}", r.headword, r.hanja))));
        v.truncate(n);
        v
    };
    let mut out = String::from("headword\thanja\tsource\tcore_definition\tstdict_definition_used\trule\tevidence\n");
    let mut rows: Vec<&Record> = pick(&|r| r.rule == Rule::Homograph, homograph);
    let derived_rows = pick(&|r| r.rule == Rule::Derived, derived);
    rows.extend(pick(&|r| matches!(r.rule, Rule::Unique | Rule::Kengdic), total.saturating_sub(rows.len() + derived_rows.len())));
    rows.extend(derived_rows);
    for r in rows {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\n", r.headword, r.hanja, r.source, tsv(&crate::common::truncate(&r.own_def, 160)), tsv(&crate::common::truncate(&r.stdict_def, 160)), r.rule.as_str(), r.evidence));
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
        Target { id: 1, source: "krdict".into(), headword: hw.into(), pos: pos.into(), defs: vec![def.into()], en: Vec::new() }
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

    fn evid(od: &[(&str, &str, &str)], cd: &[(&str, &str)]) -> Evidence {
        let mut ev = Evidence::default();
        for (hw, h, g) in od {
            ev.opendict.entry(hw_norm(hw)).or_default().push((h.to_string(), vec![g.to_string()]));
        }
        for (h, g) in cd {
            ev.cedict.entry(h.to_string()).or_default().push(g.to_string());
        }
        ev
    }

    fn kt(hw: &str, en: &str) -> Target {
        Target { id: 2, source: "kengdic".into(), headword: hw.into(), pos: "other".into(), defs: Vec::new(), en: vec![en.into()] }
    }

    #[test]
    fn kengdic_needs_exactly_one_hanja_entry_and_no_native_homograph() {
        let k = kt("환초", "atoll");
        let ev = evid(&[("환초", "環礁", "atoll")], &[]);
        assert_eq!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "고리 모양으로 배열된 산호초.")], false, &ev).unwrap().hanja, "環礁");
        // two hanja readings: ambiguous
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(Some("還草"), "noun", "b")], false, &ev).is_none());
        // a native homograph of the same POS
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "noun", "b")], false, &ev).is_none());
        // kengdic's POS is a guess ("other"): a native homograph of any known POS blocks the fill
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "verb", "b")], false, &ev).is_none());
        // ...but a native affix-like entry of unknown POS does not
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a"), std(None, "other", "b")], false, &ev).is_some());
        let v = Target { pos: "verb".into(), ..k.clone() };
        assert!(decide_kengdic(&v, &[std(Some("環礁"), "noun", "a")], false, &ev).is_none());
        // another core entry of the headword (krdict) or a one-syllable word
        assert!(decide_kengdic(&k, &[std(Some("環礁"), "noun", "a")], true, &ev).is_none());
        let one = Target { headword: "환".into(), ..k };
        assert!(decide_kengdic(&one, &[std(Some("環"), "noun", "a")], false, &evid(&[("환", "環", "atoll")], &[])).is_none());
    }

    #[test]
    fn kengdic_needs_english_corroboration_for_the_same_hanja() {
        // the three reviewed errors: kengdic's word is another word of the same sound than the only stdict entry
        let ev = evid(
            &[("회의자", "會意字", "ideograph"), ("쌍각", "雙脚", "two legs"), ("사례사", "謝禮辭", "thanks")],
            &[("會意字", "associative compound"), ("懷疑者", "sceptic")],
        );
        for (hw, hanja, gloss) in [("회의자", "會意字", "unbeliever"), ("쌍각", "雙脚", "bivlave"), ("사례사", "謝禮辭", "case history")] {
            assert!(decide_kengdic(&kt(hw, gloss), &[std(Some(hanja), "noun", "x")], false, &ev).is_none(), "{hw}");
        }
        // no evidence source at all: nothing is filled
        assert!(decide_kengdic(&kt("환초", "atoll"), &[std(Some("環礁"), "noun", "x")], false, &Evidence::default()).is_none());
        // either source is enough; both are reported; the hanja must be the same one
        let a = [std(Some("척주".replace("척주", "脊柱").as_str()), "noun", "x")];
        let k = kt("척주", "the spinal column");
        assert_eq!(decide_kengdic(&k, &a, false, &evid(&[("척주", "脊柱", "spine; spinal column")], &[])).unwrap().evidence, "opendict");
        assert_eq!(decide_kengdic(&k, &a, false, &evid(&[], &[("脊柱", "spinal column; backbone")])).unwrap().evidence, "cedict");
        assert_eq!(decide_kengdic(&k, &a, false, &evid(&[("척주", "脊柱", "spinal")], &[("脊柱", "spinal column")])).unwrap().evidence, "opendict+cedict");
        assert!(decide_kengdic(&k, &a, false, &evid(&[("척주", "脊椎", "spinal column")], &[])).is_none());
        // a gloss with no shared content word ('canteen' vs court circular) is no evidence
        assert!(decide_kengdic(&kt("조보", "canteen"), &[std(Some("朝報"), "noun", "x")], false, &evid(&[("조보", "朝報", "court gazette")], &[("朝報", "court circular")])).is_none());
    }

    #[test]
    fn gloss_keys_stem_and_skip_stopwords() {
        let k = |s: &str| gloss_keys(&[s.to_string()]);
        assert!(!k("conscription").is_disjoint(&k("to conscript; the draft")));
        assert!(!k("bribery").is_disjoint(&k("bribe")));
        assert!(!k("tub").is_disjoint(&k("a tub, bath")));
        assert!(k("something used for the").is_empty());
    }

    fn stem(h: &[&str], unclear: bool) -> Stem {
        Stem { hanja: h.iter().map(|s| s.to_string()).collect(), unclear, defs: vec!["학문이나 기술을 배우고 익힘.".into()], en: vec!["study".into()] }
    }

    #[test]
    fn derived_words_take_the_stem_hanja_and_the_native_suffix() {
        let mut t = tgt("공부하다", "verb", "학문이나 기술을 배우고 익히다.");
        t.en = vec!["to study".into()];
        let f = decide_derived(&t, &stem(&["工夫"], false), &[]).unwrap();
        assert_eq!((f.hanja.as_str(), f.rule), ("工夫하다", Rule::Derived));
        let sc = |hw: &str, pos: &str, h: &str| {
            let mut t = tgt(hw, pos, "학문이나 기술을 배우고 익힘.");
            t.en = vec!["study".into()];
            decide_derived(&t, &stem(&[h], false), &[]).map(|f| f.hanja)
        };
        assert_eq!(sc("불편스럽다", "adjective", "不便").as_deref(), Some("不便스럽다"));
        assert_eq!(sc("객관적", "noun", "客觀").as_deref(), Some("客觀的"));
        // several hanja for the stem (사기하다), a native homograph, or the wrong POS: nothing
        assert!(decide_derived(&t, &stem(&["士氣", "詐欺"], false), &[]).is_none());
        assert!(decide_derived(&t, &stem(&["工夫"], true), &[]).is_none());
        assert!(sc("공부하다", "noun", "工夫").is_none());
        // 시원하다 is not built on 시원 (始原 "origin"): no shared meaning, no fill
        let mut cool = tgt("시원하다", "adjective", "덥지도 춥지도 않고 적당하게 서늘하다.");
        cool.en = vec!["cool; refreshing".into()];
        let origin = Stem { hanja: HashSet::from(["始原".to_string()]), unclear: false, defs: vec!["사물이나 현상이 시작되는 처음.".into()], en: vec!["origin; beginning".into()] };
        assert!(decide_derived(&cool, &origin, &[]).is_none());
        // a stdict entry of the derived word with other hanja vetoes, a native one too; the same hanja confirms
        assert!(decide_derived(&t, &stem(&["工夫"], false), &[std(Some("功夫하다"), "verb", "x")]).is_none());
        assert!(decide_derived(&t, &stem(&["工夫"], false), &[std(None, "verb", "x")]).is_none());
        assert_eq!(decide_derived(&cool, &origin, &[std(Some("始原하다"), "adjective", "처음이다.")]).unwrap().stdict_def, "처음이다.");
        // 타박하다: stdict has the hanja word 打撲하다 AND a native 타박하다 of the same POS -> no fill
        let mut scold = tgt("타박하다", "verb", "다른 사람의 실수나 결함을 나무라거나 핀잔하다.");
        scold.en = vec!["to find fault".into()];
        let hit = Stem { hanja: HashSet::from(["打撲".to_string()]), unclear: false, defs: vec!["다른 사람의 실수나 결함을 나무라거나 핀잔함.".into()], en: vec!["finding fault".into()] };
        let own = [std(Some("打撲하다"), "verb", "때리고 치다."), std(None, "verb", "남의 실수나 결함을 나무라거나 핀잔하다.")];
        assert!(decide_derived(&scold, &hit, &own).is_none());
    }

    #[test]
    fn a_stem_whose_hanja_stdict_does_not_back_is_contradicted() {
        // 구경 (sightseeing): krdict has 究竟 but stdict's best match for the definition is the native 구경
        let gu = Stem { hanja: HashSet::from(["究竟".to_string()]), unclear: false, defs: vec!["흥미나 관심을 가지고 봄.".into()], en: vec!["sightseeing".into()] };
        let cands = [std(None, "noun", "흥미나 관심을 가지고 봄."), std(Some("究竟"), "noun", "일의 마지막. 또는 마지막에 이르는 상태.")];
        assert!(stem_contradicted(&gu, &cands));
        // the entry that carries the stem's hanja matches best: fine
        let ok = Stem { hanja: HashSet::from(["工夫".to_string()]), unclear: false, defs: vec!["학문이나 기술을 배우고 익힘.".into()], en: vec![] };
        assert!(!stem_contradicted(&ok, &[std(Some("工夫"), "noun", "학문이나 기술을 배우고 익힘."), std(None, "noun", "부엌에서 쓰는 도구.")]));
        // nothing in stdict matches well: no verdict
        assert!(!stem_contradicted(&ok, &[std(None, "noun", "전혀 다른 뜻의 낱말입니다.")]));
    }

    #[test]
    fn suspect_core_hanja_follows_the_reviewers_test() {
        // 타박: krdict says 打撲 with the "find fault" meaning; the native 타박 has that definition, 打撲 means hitting
        let defs = vec!["남의 잘못이나 결점을 나무라거나 핀잔함.".to_string()];
        let c = [std(None, "noun", "남의 잘못이나 결점을 나무라거나 핀잔함."), std(Some("打撲"), "noun", "때리거나 쳐서 몸에 상처를 입힘.")];
        let (ns, _, hs, hd) = suspect_of(&defs, "noun", "打撲", &c).unwrap();
        assert!(ns >= 0.6 && hs < 0.3 && !hd.is_empty());
        // the entry with the hanja matches: not suspect
        let c = [std(None, "noun", "다른 뜻."), std(Some("打撲"), "noun", "남의 잘못이나 결점을 나무라거나 핀잔함.")];
        assert!(suspect_of(&defs, "noun", "打撲", &c).is_none());
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
