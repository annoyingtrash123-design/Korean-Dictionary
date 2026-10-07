//! opendict (우리말샘) XML parser.
//!
//! Unlike stdict, every `<item>` is a single sense (one `wordInfo` + one
//! `senseInfo`); items sharing a `group_code` are senses of the same word and
//! are merged (within one file) into one entry.

use crate::common::*;
use crate::stdict::{display_word, origin, rel_word, t};
use crate::xml::{for_each_file, Node};
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

struct Item {
    order: i64,
    word: String,
    unit: String,
    hanja: Option<String>,
    origin_note: Option<String>,
    prons: Vec<String>,
    forms: Vec<String>,
    ko_pos: String,
    sense: Map<String, Value>,
    cats: Vec<String>,
    ext_id: String,
    ko_def: String,
}

fn parse_item(item: &Node) -> Option<(String, Item)> {
    let wi = item.child("wordInfo")?;
    let si = item.child("senseInfo")?;
    let word = display_word(&t(wi, "word"));
    if word.is_empty() {
        return None;
    }
    let (hanja, origin_note) = origin(wi);
    let mut prons = Vec::new();
    for p in wi.find_all("pronunciation_info/pronunciation") {
        let p = clean(&p.text);
        if !p.is_empty() && !prons.contains(&p) {
            prons.push(p);
        }
    }
    let mut forms = Vec::new();
    for path in ["conju_info/conjugation_info/conjugation", "conju_info/abbreviation_info/abbreviation"] {
        for c in wi.find_all(path) {
            let c = clean(&c.text);
            if !c.is_empty() {
                forms.push(c);
            }
        }
    }
    for a in si.find_all("abbreviation_info/abbreviation") {
        let c = clean(&a.text);
        if !c.is_empty() {
            forms.push(c);
        }
    }

    let ko_def = t(si, "definition");
    let mut sense = Map::new();
    let ko_pos = t(si, "pos");
    if !ko_def.is_empty() {
        sense.insert("ko_def".into(), ko_def.clone().into());
    }
    let grammar: Vec<String> = si.find_all("grammar_info/grammar").iter().map(|g| clean(&g.text)).filter(|g| !g.is_empty()).collect();
    if !grammar.is_empty() {
        sense.insert("note".into(), grammar.join("; ").into());
    }
    let pat = t(si, "pattern_info/pattern");
    if !pat.is_empty() {
        sense.insert("pattern".into(), pat.into());
    }
    // labels: sense type (방언/북한어/옛말/...), region, semantic category
    let mut tags: Vec<String> = Vec::new();
    let ty = t(si, "type");
    if !ty.is_empty() && ty != "일반어" {
        tags.push(ty);
    }
    for r in si.find_all("region_info/region") {
        let r = clean(&r.text);
        if !r.is_empty() && !tags.contains(&r) {
            tags.push(r);
        }
    }
    let mut cats = Vec::new();
    for c in si.find_all("cat_info/cat") {
        let ct = clean(&c.text);
        if !ct.is_empty() && ct != "없음" {
            if !cats.contains(&ct) {
                cats.push(ct.clone());
            }
            tags.push(ct);
        }
    }
    if !tags.is_empty() {
        sense.insert("tags".into(), tags.into());
    }
    let mut exs = Vec::new();
    for ei in si.kids("example_info") {
        if ei.child("source").is_some() {
            continue; // cited from copyrighted works -> excluded
        }
        let ex = t(ei, "example");
        if !ex.is_empty() {
            exs.push(json!({"ko": ex}));
        }
    }
    if !exs.is_empty() {
        sense.insert("examples".into(), exs.into());
    }
    let mut rel = Vec::new();
    for ri in si.kids("relation_info") {
        let w = rel_word(&t(ri, "word"));
        if !w.is_empty() {
            rel.push(json!({"type": kr_rel(&t(ri, "type")), "word": w}));
        }
    }
    if !rel.is_empty() {
        sense.insert("rel".into(), rel.into());
    }
    let en: Vec<String> = si
        .kids("translation_info")
        .filter(|x| t(x, "language_type") == "영어")
        .map(|x| clean(&x.t("translation")))
        .filter(|x| !x.is_empty())
        .collect();
    if !en.is_empty() {
        sense.insert("en".into(), en.join("; ").into());
    }

    let group = item.t("group_code");
    let group = if group.is_empty() { format!("t{}", item.t("target_code")) } else { group };
    Some((
        group,
        Item {
            order: item.t("group_order").parse().unwrap_or(0),
            word,
            unit: { let u = t(wi, "word_unit"); u },
            hanja,
            origin_note,
            prons,
            forms,
            ko_pos,
            sense,
            cats,
            ext_id: item.t("target_code"),
            ko_def,
        },
    ))
}

fn merge(mut items: Vec<Item>) -> Option<Entry> {
    items.sort_by_key(|i| i.order);
    // senses without a definition carry no information for this pack
    items.retain(|i| !i.ko_def.is_empty());
    let first = items.first()?;
    let mut poses: Vec<&'static str> = Vec::new();
    for i in &items {
        let p = kr_pos_or_other(&i.ko_pos);
        if !poses.contains(&p) {
            poses.push(p);
        }
    }
    let multi = poses.len() > 1;
    let (pos, kind) = match first.unit.as_str() {
        "구" => ("phrase".to_string(), "phrase".to_string()),
        "속담" => ("proverb".into(), "proverb".into()),
        "관용구" => ("idiom".into(), "idiom".into()),
        _ => {
            let p = kr_pos_or_other(&first.ko_pos).to_string();
            let k = if matches!(first.ko_pos.as_str(), "어미" | "조사") { "grammar" } else { "word" };
            (p, k.to_string())
        }
    };
    let mut prons: Vec<String> = Vec::new();
    let mut forms: Vec<String> = Vec::new();
    let mut cats: Vec<String> = Vec::new();
    let mut hanja = None;
    let mut origin_note = None;
    let mut senses = Vec::new();
    for i in &items {
        for p in &i.prons {
            if !prons.contains(p) {
                prons.push(p.clone());
            }
        }
        for f in &i.forms {
            if !forms.contains(f) {
                forms.push(f.clone());
            }
        }
        for c in &i.cats {
            if !cats.contains(c) {
                cats.push(c.clone());
            }
        }
        if hanja.is_none() {
            hanja = i.hanja.clone();
        }
        if origin_note.is_none() {
            origin_note = i.origin_note.clone();
        }
        let mut s = i.sense.clone();
        if multi {
            s.insert("pos".into(), kr_pos_or_other(&i.ko_pos).into());
        }
        senses.push(Value::Object(s));
    }
    let mut data = Map::new();
    data.insert("senses".into(), senses.into());
    if !cats.is_empty() {
        data.insert("category".into(), cats.join(" / ").into());
    }
    if let Some(o) = origin_note {
        data.insert("origin_note".into(), o.into());
    }
    let ko_defs = items.iter().map(|i| i.ko_def.clone()).collect();
    Some(Entry {
        source: "opendict",
        lang: "ko",
        ext_id: first.ext_id.clone(),
        headword: first.word.clone(),
        homonym: None,
        hanja,
        pos,
        ko_pos: first.ko_pos.clone(),
        pron: if prons.is_empty() { None } else { Some(prons.join(", ")) },
        level: None,
        kind,
        forms,
        data: Value::Object(data),
        ko_defs,
    })
}

/// Parse one file; entries (merged per group_code) are emitted at the end of the file.
pub fn parse_file(path: &Path, mut cb: impl FnMut(Entry) -> Result<()>) -> Result<()> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<Item>> = HashMap::new();
    for_each_file(path, "item", |n| {
        if let Some((g, it)) = parse_item(n) {
            if !groups.contains_key(&g) {
                order.push(g.clone());
            }
            groups.entry(g).or_default().push(it);
        }
        Ok(())
    })?;
    for g in order {
        if let Some(e) = groups.remove(&g).and_then(merge) {
            cb(e)?;
        }
    }
    Ok(())
}
