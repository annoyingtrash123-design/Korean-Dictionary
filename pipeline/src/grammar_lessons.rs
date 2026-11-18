//! Grammar mini-lessons for the NIKL curriculum patterns (`data/grammar_lessons/<level>.json`, one file
//! per 급, one lesson per curriculum grammar row). Everything in them is AI-written, so a lesson ships
//! in the core pack only after an independent review pass (`content-review` skill) has set its
//! `review.status` to `approved`; drafts stay in the repository. The app labels the whole lesson as AI.
//!
//! A lesson is keyed by the official form (`curriculum.form` without the trailing list punctuation,
//! homonym digits kept: `-거들랑1` and `-거들랑2` are two rows) plus the level.
//!
//! Core pack table `grammar_lesson(level, form, entry_id, grammar_id, body)`: `form` is the curriculum
//! row's own `form`, `entry_id` / `grammar_id` are the row's links (the lesson shows on that entry's
//! page), `body` is the lesson as JSON without its review block. Each `compare` pattern that is a
//! dictionary grammar row gets an `entry` link (`{source, id}`) so the app can open it.

use crate::curriculum::{builtin_grammar, grammar_forms, grammar_key, CurGrammar};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

pub const FILES: [(i64, &str); 6] = [
    (1, include_str!("../data/grammar_lessons/1.json")),
    (2, include_str!("../data/grammar_lessons/2.json")),
    (3, include_str!("../data/grammar_lessons/3.json")),
    (4, include_str!("../data/grammar_lessons/4.json")),
    (5, include_str!("../data/grammar_lessons/5.json")),
    (6, include_str!("../data/grammar_lessons/6.json")),
];

pub const CONTEXT_TAGS: [&str; 7] = ["conversation", "news", "essay", "formal", "literary", "proverb", "collocation"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sentence {
    pub ko: String,
    pub en: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub ko: String,
    pub en: String,
    /// Where the sentence would be heard or read (one of [`CONTEXT_TAGS`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctx: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compare {
    pub pattern: String,
    pub difference: String,
    /// `[0]` uses this lesson's pattern, `[1]` the compared one.
    pub pair: [Sentence; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pitfall {
    pub note: String,
    pub wrong: String,
    pub right: String,
    /// English of `right`.
    pub en: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    /// `draft` | `approved`
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub form: String,
    pub level: i64,
    /// Two to five words naming the pattern's job ("giving a reason"); the lesson's heading, and what
    /// tells apart the list's homonym rows (`-거들랑1` / `-거들랑2`).
    pub label: String,
    pub meaning_en: String,
    pub usage: String,
    pub examples: Vec<Example>,
    pub compare: Vec<Compare>,
    pub pitfalls: Vec<Pitfall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topik_tip: Option<String>,
    pub review: Review,
}

impl Lesson {
    pub fn approved(&self) -> bool {
        self.review.status == "approved"
    }
}

/// The lesson key of a curriculum row: its `form_raw` without the list's trailing punctuation.
pub fn lesson_key(form_raw: &str) -> &str {
    form_raw.trim().trim_end_matches([',', ' '])
}

/// Parse one level file; every lesson must carry that level.
pub fn parse(json: &str, level: i64) -> Result<Vec<Lesson>> {
    let v: Vec<Lesson> = serde_json::from_str(json).with_context(|| format!("grammar_lessons/{level}.json"))?;
    for l in &v {
        anyhow::ensure!(l.level == level, "grammar_lessons/{level}.json: {} has level {}", l.form, l.level);
    }
    Ok(v)
}

pub fn builtin() -> Vec<Lesson> {
    FILES.iter().flat_map(|(lv, s)| parse(s, *lv).expect("grammar lesson files parse")).collect()
}

/// Hangul syllables or jamo.
pub fn has_hangul(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c) || is_jamo(c))
}

fn is_jamo(c: char) -> bool {
    ('\u{3131}'..='\u{318E}').contains(&c)
}

/// The TOPIK section names that English text may use without quotes.
pub const TOPIK_SECTIONS: [&str; 4] = ["듣기", "말하기", "읽기", "쓰기"];

/// Hangul syllables left in an English field once quoted forms, the TOPIK section names and bare jamo
/// (grammar terms such as "ㄹ-stem") are set aside.
fn stray_hangul(s: &str) -> bool {
    let mut t = without_quotes(s);
    for n in TOPIK_SECTIONS {
        t = t.replace(n, "");
    }
    t.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

/// An English field with its quoted spans removed (`“…”`, `"…"`, `‘…’`): quoted Korean forms are
/// allowed in English text, anything else Korean is not.
pub fn without_quotes(s: &str) -> String {
    let mut out = String::new();
    let mut close: Option<char> = None;
    for c in s.chars() {
        match close {
            Some(q) if c == q => close = None,
            Some(_) => {}
            None => match c {
                '“' => close = Some('”'),
                '‘' => close = Some('’'),
                '"' => close = Some('"'),
                _ => out.push(c),
            },
        }
    }
    out
}

fn english_problem(field: &str, s: &str, v: &mut Vec<String>) {
    if s.trim().is_empty() {
        v.push(format!("{field} is empty"));
    } else if stray_hangul(s) {
        v.push(format!("{field} has unquoted Hangul"));
    }
    if s.contains(char::is_control) || s.trim() != s {
        v.push(format!("{field} has stray whitespace or control characters"));
    }
}

fn korean_problem(field: &str, s: &str, v: &mut Vec<String>) {
    if !has_hangul(s) || !s.trim_end().ends_with(['.', '?', '!', '…', '”', '"']) || s.trim() != s || s.contains(char::is_control) {
        v.push(format!("{field} is not a clean Korean sentence"));
    }
}

/// Strings that show a sentence uses the pattern: the last word of each form the list gives for the row
/// (`-어 치우다` -> 치우), with the trailing 다 / 이다 removed and the leading 으 / 이 optional.
fn pattern_cores(g: &CurGrammar) -> Vec<String> {
    let mut v = Vec::new();
    for f in grammar_forms(g) {
        let full = grammar_key(f.rsplit(' ').next().unwrap_or(&f));
        let k = full.trim_end_matches('다').trim_end_matches('이').to_string();
        // the plain ending -다 (and 이다) is all "다": trimming leaves nothing, so match the ending itself
        let k = if k.is_empty() { full.to_string() } else { k };
        if k.is_empty() {
            continue;
        }
        // 으 / 이 are optional, and an ending that starts with a bare jamo (ㄹ망정) shows up written into the
        // previous syllable (가난할망정): match what follows the jamo
        let bare = k.strip_prefix('으').or_else(|| k.strip_prefix('이')).or_else(|| k.strip_prefix(is_jamo)).unwrap_or(&k).to_string();
        // a stem that ends in a vowel contracts with the tense ending (치우다 -> 치웠다): match its first syllables too
        if k.chars().count() >= 2 {
            v.push(k.chars().take(k.chars().count() - 1).collect());
        }
        v.push(k);
        if !bare.is_empty() {
            v.push(bare);
        }
    }
    v.sort();
    v.dedup();
    v
}

/// A homonym row (`-거들랑1` / `-거들랑2`) needs its own label: two lessons of one entry must differ.
pub fn homonym_label_problems(lessons: &[Lesson]) -> Vec<String> {
    let mut seen: HashMap<(i64, String), &str> = HashMap::new();
    let mut out = Vec::new();
    for l in lessons {
        let base = l.form.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        if let Some(other) = seen.insert((l.level, base), &l.label) {
            if other == l.label {
                out.push(format!("{}급 {}: the same label as its homonym row", l.level, l.form));
            }
        }
    }
    out
}

/// Structural and content-shape problems of one lesson (empty when it is fine). `row` is the curriculum
/// row the lesson belongs to.
pub fn problems(l: &Lesson, row: &CurGrammar) -> Vec<String> {
    let mut v = Vec::new();
    if !matches!(l.review.status.as_str(), "draft" | "approved") {
        v.push(format!("review.status {:?}", l.review.status));
    }
    if l.approved() && (l.review.reviewer.as_deref().unwrap_or("").is_empty() || l.review.date.as_deref().unwrap_or("").is_empty()) {
        v.push("an approved lesson names its reviewer and date".into());
    }
    english_problem("label", &l.label, &mut v);
    if !(2..=8).contains(&l.label.split_whitespace().count()) {
        v.push("label is two to eight words".into());
    }
    english_problem("meaning_en", &l.meaning_en, &mut v);
    if l.meaning_en.split_whitespace().count() < 6 {
        v.push("meaning_en too short".into());
    }
    english_problem("usage", &l.usage, &mut v);
    if l.usage.split_whitespace().count() < 8 {
        v.push("usage too short".into());
    }
    if !(4..=6).contains(&l.examples.len()) {
        v.push(format!("{} examples (expected 4-6)", l.examples.len()));
    }
    let mut seen = HashSet::new();
    let cores = pattern_cores(row);
    let mut using = 0;
    for (i, e) in l.examples.iter().enumerate() {
        korean_problem(&format!("example {}", i + 1), &e.ko, &mut v);
        english_problem(&format!("example {} en", i + 1), &e.en, &mut v);
        if e.ko.split_whitespace().count() < 3 {
            v.push(format!("example {} is too short", i + 1));
        }
        if !seen.insert(&e.ko) {
            v.push(format!("example {} repeated", i + 1));
        }
        if let Some(c) = &e.ctx {
            if !CONTEXT_TAGS.contains(&c.as_str()) {
                v.push(format!("example {} ctx {c:?}", i + 1));
            }
        }
        let squashed = grammar_key(&e.ko);
        if cores.iter().any(|c| squashed.contains(c.as_str())) {
            using += 1;
        }
    }
    if using < l.examples.len().min(3) {
        v.push(format!("only {using} examples contain the pattern ({cores:?})"));
    }
    if !l.examples.iter().any(|e| e.ctx.as_deref() == Some("collocation")) {
        v.push("no example is tagged as a common collocation".into());
    }
    if l.examples.iter().filter_map(|e| e.ctx.as_deref()).collect::<HashSet<_>>().len() < 2 {
        v.push("examples need at least two different contexts".into());
    }
    if !(1..=3).contains(&l.compare.len()) {
        v.push(format!("{} compare entries (expected 1-3)", l.compare.len()));
    }
    for (i, c) in l.compare.iter().enumerate() {
        let t = format!("compare {}", i + 1);
        if !has_hangul(&c.pattern) || c.pattern.trim() != c.pattern {
            v.push(format!("{t}: pattern"));
        }
        // the homonym rows of one form (-자면1 / -자면2) may be compared with each other
        if grammar_key(&c.pattern) == grammar_key(&l.form) && !l.form.ends_with(|ch: char| ch.is_ascii_digit()) {
            v.push(format!("{t}: compares the pattern with itself"));
        }
        english_problem(&format!("{t} difference"), &c.difference, &mut v);
        for (j, s) in c.pair.iter().enumerate() {
            korean_problem(&format!("{t} pair {}", j + 1), &s.ko, &mut v);
            english_problem(&format!("{t} pair {} en", j + 1), &s.en, &mut v);
        }
        if c.pair[0].ko == c.pair[1].ko {
            v.push(format!("{t}: the pair is one sentence twice"));
        }
    }
    if !(1..=3).contains(&l.pitfalls.len()) {
        v.push(format!("{} pitfalls (expected 1-3)", l.pitfalls.len()));
    }
    for (i, p) in l.pitfalls.iter().enumerate() {
        let t = format!("pitfall {}", i + 1);
        english_problem(&format!("{t} note"), &p.note, &mut v);
        english_problem(&format!("{t} en"), &p.en, &mut v);
        for (n, s) in [("wrong", &p.wrong), ("right", &p.right)] {
            if !has_hangul(s) || s.trim() != s {
                v.push(format!("{t}: {n} is not Korean"));
            }
        }
        if p.wrong == p.right {
            v.push(format!("{t}: wrong and right are the same"));
        }
    }
    if let Some(t) = &l.topik_tip {
        english_problem("topik_tip", t, &mut v);
    }
    v
}

/// Match each lesson to its curriculum row (form + level); returns the problems of the whole set.
pub fn check_all(lessons: &[Lesson], rows: &[CurGrammar]) -> Vec<String> {
    let by_key: HashMap<(i64, &str), &CurGrammar> = rows.iter().map(|g| ((g.level, lesson_key(&g.form_raw)), g)).collect();
    let mut seen = HashSet::new();
    let mut out = homonym_label_problems(lessons);
    for l in lessons {
        if !seen.insert((l.level, l.form.as_str())) {
            out.push(format!("{}급 {}: two lessons for one row", l.level, l.form));
        }
        match by_key.get(&(l.level, l.form.as_str())) {
            None => out.push(format!("{}급 {}: no such curriculum row", l.level, l.form)),
            Some(row) => out.extend(problems(l, row).into_iter().map(|p| format!("{}급 {}: {p}", l.level, l.form))),
        }
    }
    out
}

/// The body stored in the pack: the lesson without its review block, each compare pattern linked to its
/// dictionary entry where `links` (grammar key -> (source, entry id)) has one.
pub fn body(l: &Lesson, links: &HashMap<String, (String, i64)>) -> Value {
    let mut v = serde_json::to_value(l).expect("lesson serialises");
    let o = v.as_object_mut().expect("object");
    o.remove("review");
    o.remove("level");
    o.remove("form");
    if let Some(cs) = o.get_mut("compare").and_then(Value::as_array_mut) {
        for c in cs {
            let key = grammar_key(c["pattern"].as_str().unwrap_or(""));
            if let Some((source, id)) = links.get(&key) {
                c["entry"] = json!({"source": source, "id": id});
            }
        }
    }
    v
}

/// Create and fill `grammar_lesson` (needs `curriculum`, `grammar` and `entries`): only approved lessons
/// ship. Returns (approved, drafted).
pub fn build_table(conn: &Connection) -> Result<(usize, usize)> {
    build_table_from(conn, &builtin(), &builtin_grammar())
}

/// [`build_table`] for a given set of lessons (tests approve some without touching the data files).
pub fn build_table_from(conn: &Connection, lessons: &[Lesson], rows: &[CurGrammar]) -> Result<(usize, usize)> {
    let problems = check_all(lessons, rows);
    anyhow::ensure!(problems.is_empty(), "grammar lessons: {} problems, first: {}", problems.len(), problems[0]);
    conn.execute_batch(
        "CREATE TABLE grammar_lesson (level INTEGER NOT NULL, form TEXT NOT NULL, entry_id INTEGER, grammar_id INTEGER, body TEXT NOT NULL);",
    )?;
    // compare links: every grammar row's pattern (and each alternative of `A/B`) -> its entry
    let mut links: HashMap<String, (String, i64)> = HashMap::new();
    {
        let mut st = conn.prepare("SELECT g.pattern, e.source, e.id FROM grammar g JOIN entries e ON e.id = g.entry_id ORDER BY g.id DESC")?;
        let mut q = st.query([])?;
        while let Some(r) = q.next()? {
            let (p, s, id): (String, String, i64) = (r.get(0)?, r.get(1)?, r.get(2)?);
            for alt in p.split('/') {
                links.insert(grammar_key(alt), (s.clone(), id));
            }
        }
    }
    type CurRef = (String, Option<i64>, Option<i64>);
    let mut by_key: HashMap<(i64, String), CurRef> = HashMap::new();
    {
        let mut st = conn.prepare("SELECT level, form, entry_id, grammar_id FROM curriculum WHERE kind = 'grammar'")?;
        let mut q = st.query([])?;
        while let Some(r) = q.next()? {
            let form: String = r.get(1)?;
            by_key.insert((r.get(0)?, lesson_key(&form).to_string()), (form, r.get(2)?, r.get(3)?));
        }
    }
    let mut approved = 0;
    for l in lessons.iter().filter(|l| l.approved()) {
        let (form, entry_id, grammar_id) = by_key.get(&(l.level, l.form.clone())).cloned().with_context(|| format!("{} not in the curriculum table", l.form))?;
        conn.prepare_cached("INSERT INTO grammar_lesson(level, form, entry_id, grammar_id, body) VALUES(?,?,?,?,?)")?
            .execute(params![l.level, form, entry_id, grammar_id, body(l, &links).to_string()])?;
        approved += 1;
    }
    conn.execute_batch("CREATE INDEX grammar_lesson_entry ON grammar_lesson(entry_id);")?;
    log::info!("grammar lessons: {approved} approved of {} drafted", lessons.len());
    Ok((approved, lessons.len()))
}
