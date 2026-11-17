//! Reviewed overlay of grammar patterns that krdict has no row for (`data/grammar_extra.tsv`; the file
//! header documents the columns). Every pattern of the NIKL curriculum list must resolve to an entry
//! (`curriculum.rs`); the ones krdict lacks are added to the core pack as source `hanppuri` entries
//! (kind `grammar`) plus rows of the `grammar` table.
//!
//! Real dictionary text (definition or example copied from stdict / 우리말샘) is preferred and keeps
//! its source id; anything written for this file is AI text and carries a flag in the entry JSON so
//! the app labels it: `def_ai` (English summary, always), `ko_def_ai`, and per example `en_ai`
//! (English, always) and `ko_ai`.

use crate::common::Entry;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub const SOURCE: &str = "hanppuri";

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraExample {
    pub ko: String,
    /// "stdict" | "opendict" | "ai"
    pub ko_source: String,
    pub en: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraPattern {
    pub pattern: String,
    /// Grammar table category ("Connective endings", ...).
    pub category: &'static str,
    pub nikl_level: i64,
    pub ko_def: String,
    /// "stdict:<ext_id>" | "opendict:<ext_id>" | "ai"
    pub ko_def_source: String,
    pub en_summary: String,
    pub forms: Vec<String>,
    pub examples: Vec<ExtraExample>,
}

pub fn category_of(name: &str) -> Option<&'static str> {
    Some(match name {
        "connective ending" => "Connective endings",
        "final ending" => "Final endings",
        "expression" => "Expressions",
        "particle" => "Particles",
        _ => return None,
    })
}

fn valid_def_source(s: &str) -> bool {
    s == "ai" || ["stdict:", "opendict:"].iter().any(|p| s.strip_prefix(p).is_some_and(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())))
}

pub fn parse(tsv: &str) -> Result<Vec<ExtraPattern>> {
    let mut out: Vec<ExtraPattern> = Vec::new();
    for (i, line) in tsv.lines().enumerate().filter(|(_, l)| !l.starts_with('#') && !l.trim().is_empty()) {
        let n = i + 1;
        let f: Vec<&str> = line.split('\t').collect();
        let (min, max) = match f[0] {
            "P" => (7, 8),
            "E" => (5, 5),
            t => bail!("grammar_extra.tsv line {n}: row type {t:?} (expected P or E)"),
        };
        if !(min..=max).contains(&f.len()) {
            bail!("grammar_extra.tsv line {n}: {} columns, expected {min}..={max}", f.len());
        }
        match f[0] {
            "P" => {
                let category = category_of(f[2]).with_context(|| format!("line {n}: unknown category {:?}", f[2]))?;
                out.push(ExtraPattern {
                    pattern: f[1].to_string(),
                    category,
                    nikl_level: f[3].parse().with_context(|| format!("line {n}: level"))?,
                    ko_def: f[4].to_string(),
                    ko_def_source: f[5].to_string(),
                    en_summary: f[6].to_string(),
                    forms: f.get(7).copied().unwrap_or("").split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect(),
                    examples: Vec::new(),
                });
            }
            "E" => {
                let p = out.last_mut().filter(|p| p.pattern == f[1]).with_context(|| format!("line {n}: example of {:?} does not follow its P row", f[1]))?;
                p.examples.push(ExtraExample { ko: f[2].to_string(), ko_source: f[3].to_string(), en: f[4].to_string() });
            }
            _ => unreachable!("row type checked above"),
        }
    }
    Ok(out)
}

pub fn builtin() -> Vec<ExtraPattern> {
    parse(include_str!("../data/grammar_extra.tsv")).expect("grammar_extra.tsv parses")
}

/// Structural problems of one pattern (empty when it is fine): the quality gate of the overlay.
pub fn problems(p: &ExtraPattern) -> Vec<String> {
    let mut v = Vec::new();
    let has_hangul = |s: &str| s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c));
    if !has_hangul(&p.pattern) {
        v.push("pattern has no Hangul".into());
    }
    if !(1..=6).contains(&p.nikl_level) {
        v.push(format!("level {} outside 1-6", p.nikl_level));
    }
    if !has_hangul(&p.ko_def) || !p.ko_def.trim_end().ends_with('.') {
        v.push("ko_def must be Korean and end with a period".into());
    }
    if !valid_def_source(&p.ko_def_source) {
        v.push(format!("ko_def_source {:?}", p.ko_def_source));
    }
    if p.en_summary.split_whitespace().count() < 5 || p.en_summary.contains(char::is_control) {
        v.push("en_summary too short".into());
    }
    if !(2..=3).contains(&p.examples.len()) {
        v.push(format!("{} examples (expected 2-3)", p.examples.len()));
    }
    for (i, e) in p.examples.iter().enumerate() {
        let tag = format!("example {}", i + 1);
        if !has_hangul(&e.ko) || e.ko.split_whitespace().count() < 3 || !e.ko.trim_end().ends_with(['.', '?', '!']) {
            v.push(format!("{tag}: not a full Korean sentence"));
        }
        if !matches!(e.ko_source.as_str(), "stdict" | "opendict" | "ai") {
            v.push(format!("{tag}: ko_source {:?}", e.ko_source));
        }
        if has_hangul(&e.en) || e.en.split_whitespace().count() < 3 || !e.en.trim_end().ends_with(['.', '?', '!']) {
            v.push(format!("{tag}: English translation missing or not English"));
        }
        // the example must actually use the pattern or one of its forms (hyphens and spaces ignored)
        let squash = |s: &str| s.chars().filter(|c| !matches!(c, '-' | ' ' | '\u{a0}')).collect::<String>();
        let ko = squash(&e.ko);
        if !std::iter::once(&p.pattern).chain(&p.forms).any(|f| ko.contains(&squash(f))) {
            v.push(format!("{tag}: does not contain the pattern"));
        }
    }
    v
}

fn pos_of(category: &str) -> &'static str {
    match category {
        "Particles" => "particle",
        "Expressions" => "expression",
        _ => "ending",
    }
}

/// The `entries` row of a pattern (core pack, source `hanppuri`, kind `grammar`, language `en`:
/// English summary first, the Korean definition shown with it like krdict's grammar entries).
pub fn to_entry(p: &ExtraPattern, ordinal: usize) -> Entry {
    let ai_def = p.ko_def_source == "ai";
    let mut sense = json!({
        "def": p.en_summary,
        "def_ai": true,
        "ko_def": p.ko_def,
        "ko_def_source": p.ko_def_source,
        "examples": p.examples.iter().map(|e| {
            let mut x = json!({"ko": e.ko, "type": "sentence", "en": e.en, "en_ai": true, "ko_source": e.ko_source});
            if e.ko_source == "ai" {
                x["ko_ai"] = Value::Bool(true);
            }
            x
        }).collect::<Vec<_>>(),
    });
    if ai_def {
        sense["ko_def_ai"] = Value::Bool(true);
    }
    Entry {
        source: SOURCE,
        lang: "en",
        ext_id: format!("grammar-extra-{}", ordinal + 1),
        headword: p.pattern.clone(),
        homonym: None,
        hanja: None,
        pos: pos_of(p.category).into(),
        ko_pos: String::new(),
        pron: None,
        level: Some((p.nikl_level + 1) / 2),
        kind: "grammar".into(),
        forms: p.forms.clone(),
        data: json!({"senses": [sense]}),
        ko_defs: vec![p.ko_def.clone()],
        pointer: None,
    }
}
