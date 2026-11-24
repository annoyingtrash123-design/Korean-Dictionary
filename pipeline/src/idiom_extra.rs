//! English for 속담 and 성어 that krdict has no entry for (`data/idiom_extra.tsv`; the file header documents
//! the columns): the proverbs of study list `sokdam100` and the 성어 of `qa/frequency/seongeo_review.tsv` that
//! only stdict / 우리말샘 have. Rows ship ONLY once their `review` column says `approved...`: they become core
//! pack entries of source `hanppuri` (kind `proverb` for 속담, `word` for 성어, like krdict), and the study list
//! rows that pointed at the Korean-only entry point at them (`study_idioms.rs`).
//!
//! Real dictionary text (the Korean definition, and examples when the entry has two full sentences) is copied
//! verbatim and keeps its source id; everything written for this file is AI text and carries a flag in the
//! entry JSON so the app labels it: `gloss_ai`, `def_ai`, per example `en_ai` (English, always) and `ko_ai`.

use crate::common::Entry;
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde_json::{json, Value};

pub use crate::grammar_extra::SOURCE;

#[derive(Debug, Clone, PartialEq)]
pub struct IdiomExample {
    pub ko: String,
    /// "stdict" | "opendict" | "ai"
    pub ko_source: String,
    pub en: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdiomItem {
    /// "sokdam" | "seongeo"
    pub kind: String,
    pub headword: String,
    pub hanja: String,
    /// Pack of the Korean source entry: "stdict" | "opendict".
    pub src_pack: String,
    pub src_id: String,
    pub en_gloss: String,
    pub en_def: String,
    pub ko_def: String,
    pub review: String,
    pub examples: Vec<IdiomExample>,
}

pub fn has_hangul(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

/// English text keeps Hangul only inside quotes (a quoted Korean word is fine, a left-over translation is not).
pub fn has_unquoted_hangul(s: &str) -> bool {
    let mut depth = false;
    for c in s.chars() {
        match c {
            '"' | '“' | '”' => depth = !depth,
            c if !depth && ('\u{AC00}'..='\u{D7A3}').contains(&c) => return true,
            _ => {}
        }
    }
    false
}

pub fn parse(tsv: &str) -> Result<Vec<IdiomItem>> {
    let mut out: Vec<IdiomItem> = Vec::new();
    for (i, line) in tsv.lines().enumerate().filter(|(_, l)| !l.starts_with('#') && !l.trim().is_empty()) {
        let n = i + 1;
        let f: Vec<&str> = line.split('\t').collect();
        let want = match f[0] {
            "P" => 9,
            "E" => 5,
            t => bail!("idiom_extra.tsv line {n}: row type {t:?} (expected P or E)"),
        };
        if f.len() != want {
            bail!("idiom_extra.tsv line {n}: {} columns, expected {want}", f.len());
        }
        if f[0] == "P" {
            let (pack, id) = f[4].split_once(':').with_context(|| format!("line {n}: src {:?}", f[4]))?;
            out.push(IdiomItem {
                kind: f[1].to_string(),
                headword: f[2].to_string(),
                hanja: f[3].to_string(),
                src_pack: pack.to_string(),
                src_id: id.to_string(),
                en_gloss: f[5].to_string(),
                en_def: f[6].to_string(),
                ko_def: f[7].to_string(),
                review: f[8].to_string(),
                examples: Vec::new(),
            });
        } else {
            let p = out.last_mut().filter(|p| p.headword == f[1]).with_context(|| format!("line {n}: example of {:?} does not follow its P row", f[1]))?;
            p.examples.push(IdiomExample { ko: f[2].to_string(), ko_source: f[3].to_string(), en: f[4].to_string() });
        }
    }
    Ok(out)
}

pub fn builtin() -> Vec<IdiomItem> {
    parse(include_str!("../data/idiom_extra.tsv")).expect("idiom_extra.tsv parses")
}

/// Only an independent review may set this: the review column then starts with `approved`.
pub fn is_approved(it: &IdiomItem) -> bool {
    it.review.starts_with("approved")
}

/// The approved rows, the only ones that ship.
pub fn approved() -> Vec<IdiomItem> {
    builtin().into_iter().filter(is_approved).collect()
}

/// `ext_id` of the core entry of an item (stable: it names the Korean source entry).
pub fn ext_id(it: &IdiomItem) -> String {
    format!("idiom-{}-{}", it.src_pack, it.src_id)
}

/// The part of an example a headword must appear in: the headword without its last two syllables for a
/// proverb (they inflect: 갚는다 / 갚았다), the whole 성어.
pub fn stem(it: &IdiomItem) -> String {
    let h: Vec<char> = it.headword.chars().filter(|c| *c != ' ').collect();
    let keep = if it.kind == "seongeo" { h.len() } else { h.len().saturating_sub(2).max(2) };
    h[..keep].iter().collect()
}

/// Structural problems of one item (empty when it is fine): the quality gate of the overlay.
pub fn problems(it: &IdiomItem) -> Vec<String> {
    let mut v = Vec::new();
    if !matches!(it.kind.as_str(), "sokdam" | "seongeo") {
        v.push(format!("kind {:?}", it.kind));
    }
    if !has_hangul(&it.headword) || it.headword.contains(['(', ')']) {
        v.push("headword must be plain Hangul".into());
    }
    if it.kind == "seongeo" {
        if it.headword.chars().count() != 4 || it.hanja.chars().count() != 4 || !it.hanja.chars().all(crate::common::is_cjk) {
            v.push("a 성어 needs four syllables and four hanja".into());
        }
        if !it.en_def.contains("lit. ") {
            v.push("a 성어 definition ends with its literal reading '(lit. ...)'".into());
        }
    } else if !it.hanja.is_empty() {
        v.push("a 속담 has no hanja column".into());
    }
    if !matches!(it.src_pack.as_str(), "stdict" | "opendict") || it.src_id.is_empty() || !it.src_id.chars().all(|c| c.is_ascii_digit()) {
        v.push(format!("src {}:{}", it.src_pack, it.src_id));
    }
    let words = |s: &str| s.split_whitespace().count();
    if it.en_gloss.trim().is_empty() || has_hangul(&it.en_gloss) || it.en_gloss.len() > 100 {
        v.push("en_gloss empty, not English or too long".into());
    }
    if words(&it.en_def) < 5 || has_unquoted_hangul(&it.en_def) || !it.en_def.trim_end().ends_with(['.', '!', '?']) || it.en_def.contains(char::is_control) {
        v.push("en_def must be an English sentence".into());
    }
    if !has_hangul(&it.ko_def) || it.ko_def != it.ko_def.trim() {
        v.push("ko_def must be Korean, trimmed".into());
    }
    if !(it.review == "draft" || it.review.starts_with("approved") || it.review.starts_with("rejected: ")) {
        v.push(format!("review {:?} (draft | approved... | rejected: reason)", it.review));
    }
    if !(2..=3).contains(&it.examples.len()) {
        v.push(format!("{} examples (expected 2-3)", it.examples.len()));
    }
    let stem = stem(it);
    for (i, e) in it.examples.iter().enumerate() {
        let tag = format!("example {}", i + 1);
        if !has_hangul(&e.ko) || e.ko.split_whitespace().count() < 3 || !e.ko.trim_end().ends_with(['.', '?', '!']) {
            v.push(format!("{tag}: not a full Korean sentence"));
        }
        if !(e.ko_source == "ai" || e.ko_source == it.src_pack) {
            v.push(format!("{tag}: ko_source {:?} (ai, or the pack of src)", e.ko_source));
        }
        if has_unquoted_hangul(&e.en) || e.en.split_whitespace().count() < 3 || !e.en.trim_end().ends_with(['.', '?', '!']) {
            v.push(format!("{tag}: English translation missing or not English"));
        }
        if !e.ko.replace(' ', "").contains(&stem) {
            v.push(format!("{tag}: does not contain {stem}"));
        }
    }
    v
}

/// The krdict core entry (its `ext_id`) an item duplicates, if any: same headword (spaces ignored) and, for a 성어,
/// the same hanja. The overlay is only for words krdict LACKS, so an approved item for which this returns `Some` is
/// a bug (build and `tests/idiom_extra.rs` both fail on it); such a row is marked `rejected: krdict has this entry`.
pub fn krdict_duplicate(core: &Connection, it: &IdiomItem) -> Result<Option<String>> {
    let kind = if it.kind == "seongeo" { "word" } else { "proverb" };
    let mut q = core.prepare_cached("SELECT ext_id, hanja FROM entries WHERE source = 'krdict' AND kind = ?1 AND replace(headword, ' ', '') = ?2")?;
    let want = crate::common::hw_norm(&it.headword);
    let rows = q.query_map([kind, want.as_str()], |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)))?;
    for r in rows {
        let (ext, hanja) = r?;
        if it.kind != "seongeo" || hanja.as_deref() == Some(it.hanja.as_str()) {
            return Ok(Some(ext.unwrap_or_default()));
        }
    }
    Ok(None)
}

/// The core pack entry of an approved item (source `hanppuri`; kind as krdict has it: `proverb`, or `word` for a 성어).
pub fn to_entry(it: &IdiomItem) -> Entry {
    let src = format!("{}:{}", it.src_pack, it.src_id);
    let sense = json!({
        "gloss": it.en_gloss,
        "gloss_ai": true,
        "def": it.en_def,
        "def_ai": true,
        "ko_def": it.ko_def,
        "ko_def_source": src,
        "examples": it.examples.iter().map(|e| {
            let mut x = json!({"ko": e.ko, "type": "sentence", "en": e.en, "en_ai": true, "ko_source": e.ko_source});
            if e.ko_source == "ai" {
                x["ko_ai"] = Value::Bool(true);
            }
            x
        }).collect::<Vec<_>>(),
    });
    let seongeo = it.kind == "seongeo";
    Entry {
        source: SOURCE,
        lang: "en",
        ext_id: ext_id(it),
        headword: it.headword.clone(),
        homonym: None,
        hanja: seongeo.then(|| it.hanja.clone()),
        pos: if seongeo { "noun" } else { "proverb" }.into(),
        ko_pos: String::new(),
        pron: None,
        level: None,
        kind: if seongeo { "word" } else { "proverb" }.into(),
        forms: Vec::new(),
        data: json!({"senses": [sense]}),
        ko_defs: vec![it.ko_def.clone()],
        pointer: None,
    }
}

/// Check an item against its Korean source entry in the (built) stdict / opendict pack: the definition and every
/// non-AI example are copied verbatim. `Ok(false)` when the pack has no such entry (partial local builds).
pub fn check_against_pack(pack: &Connection, it: &IdiomItem) -> Result<bool> {
    let data: Option<String> = pack
        .query_row("SELECT data FROM entries WHERE source = ?1 AND ext_id = ?2 LIMIT 1", [it.src_pack.as_str(), it.src_id.as_str()], |r| r.get(0))
        .ok();
    let Some(data) = data else { return Ok(false) };
    let v: Value = serde_json::from_str(&data)?;
    let senses = v.get("senses").and_then(Value::as_array).cloned().unwrap_or_default();
    let first = senses.first().and_then(|s| s.get("ko_def")).and_then(Value::as_str).unwrap_or_default();
    if first != it.ko_def {
        bail!("{} ({}:{}): ko_def is not the pack's first definition\n  file: {}\n  pack: {first}", it.headword, it.src_pack, it.src_id, it.ko_def);
    }
    let exs: Vec<&str> = senses.iter().flat_map(|s| s.get("examples").and_then(Value::as_array).into_iter().flatten()).filter_map(|x| x.get("ko").and_then(Value::as_str)).map(str::trim).collect();
    for e in it.examples.iter().filter(|e| e.ko_source != "ai") {
        if !exs.contains(&e.ko.as_str()) {
            bail!("{} ({}:{}): example is not in the pack entry: {}", it.headword, it.src_pack, it.src_id, e.ko);
        }
    }
    Ok(true)
}
