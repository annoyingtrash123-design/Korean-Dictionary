//! krdict (한국어기초사전) LMF XML parser.

use crate::common::*;
use crate::xml::{for_each_file, Node};
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

fn ex_type(t: &str) -> Option<&'static str> {
    match t {
        "구" => Some("phrase"),
        "문장" => Some("sentence"),
        "대화" => Some("dialogue"),
        _ => None,
    }
}

fn f(n: &Node, att: &str) -> String {
    clean(n.feat(att).unwrap_or(""))
}

pub fn parse_entry(le: &Node) -> Option<Entry> {
    let lemma_el = le.child("Lemma")?;
    let headword = f(lemma_el, "writtenForm");
    if headword.is_empty() {
        return None;
    }
    let mut variants: Vec<String> = Vec::new();
    for lm in le.kids("Lemma") {
        let v = f(lm, "variant");
        if !v.is_empty() && v != headword && !variants.contains(&v) {
            variants.push(v);
        }
    }

    let unit = le.feat("lexicalUnit").unwrap_or("단어");
    let ko_pos = le.feat("partOfSpeech").unwrap_or("").to_string();
    let mut kind = kr_unit_kind(unit).to_string();
    let pos = match kr_unit_pos(unit) {
        Some(p) => p.to_string(),
        None => kr_pos_or_other(&ko_pos).to_string(),
    };
    if kind == "word" && (ko_pos == "어미" || ko_pos == "조사") {
        kind = "grammar".into();
    }
    let homonym = le.feat("homonym_number").and_then(|s| s.trim().parse::<i64>().ok()).filter(|&n| n != 0);
    let level = le.feat("vocabularyLevel").and_then(kr_level);

    let origin = f(le, "origin");
    let (mut hanja, mut origin_note) = (None, None);
    if !origin.is_empty() {
        if has_cjk(&origin) && !origin.chars().any(|c| c.is_ascii_alphabetic()) {
            hanja = Some(origin);
        } else {
            origin_note = Some(origin);
        }
    }

    let mut prons: Vec<String> = Vec::new();
    let mut forms: Vec<String> = Vec::new();
    for wf in le.kids("WordForm") {
        match wf.feat("type").unwrap_or("") {
            "발음" => {
                let p = f(wf, "pronunciation");
                if !p.is_empty() && !prons.contains(&p) {
                    prons.push(p);
                }
            }
            "활용" => {
                let w = f(wf, "writtenForm");
                if !w.is_empty() {
                    forms.push(w);
                }
                for fr in wf.kids("FormRepresentation") {
                    let w2 = f(fr, "writtenForm");
                    if !w2.is_empty() {
                        forms.push(w2);
                    }
                }
            }
            _ => {}
        }
    }
    forms.extend(variants);

    let mut related = Vec::new();
    for rf in le.kids("RelatedForm") {
        let w = f(rf, "writtenForm");
        if !w.is_empty() {
            related.push(json!({"type": kr_rel(rf.feat("type").unwrap_or("")), "word": w}));
        }
    }

    let mut senses = Vec::new();
    let mut ko_defs = Vec::new();
    for s in le.kids("Sense") {
        let mut sense = Map::new();
        let (mut gl, mut df) = (Vec::new(), Vec::new());
        for eq in s.kids("Equivalent") {
            if eq.feat("language") != Some("영어") {
                continue;
            }
            let g = f(eq, "lemma");
            let d = f(eq, "definition");
            // krdict marks untranslatable items with "(no equivalent expression)"
            if !g.is_empty() && !g.to_lowercase().contains("no equivalent") {
                gl.push(g);
            }
            if !d.is_empty() && !(d.len() < 40 && d.to_lowercase().contains("no equivalent")) {
                df.push(d);
            }
        }
        if !gl.is_empty() {
            // grammar entries carry a romanisation (e.g. "-aseo") as their English lemma
            let roman = kind == "grammar" || (gl.len() == 1 && looks_romanized(&gl[0], &headword));
            let key = if roman { "roman" } else { "gloss" };
            sense.insert(key.into(), gl.join("; ").into());
        }
        if !df.is_empty() {
            sense.insert("def".into(), df.join(" ").into());
        }
        let kd = f(s, "definition");
        if !kd.is_empty() {
            sense.insert("ko_def".into(), kd.clone().into());
            ko_defs.push(kd);
        }
        let notes: Vec<String> = [f(s, "annotation"), f(s, "syntacticAnnotation")].into_iter().filter(|n| !n.is_empty()).collect();
        if !notes.is_empty() {
            sense.insert("note".into(), notes.join(" ").into());
        }
        let pat = f(s, "syntacticPattern");
        if !pat.is_empty() {
            sense.insert("pattern".into(), pat.into());
        }
        let mut exs = Vec::new();
        for se in s.kids("SenseExample") {
            let lines: Vec<String> = se
                .kids("feat")
                .filter(|x| x.attr("att") == Some("example"))
                .map(|x| clean(x.attr("val").unwrap_or("")))
                .filter(|x| !x.is_empty())
                .collect();
            if lines.is_empty() {
                continue;
            }
            let mut ex = Map::new();
            ex.insert("ko".into(), lines.join("\n").into());
            if let Some(t) = ex_type(se.feat("type").unwrap_or("")) {
                ex.insert("type".into(), t.into());
            }
            exs.push(Value::Object(ex));
        }
        if !exs.is_empty() {
            sense.insert("examples".into(), exs.into());
        }
        let mut rel = Vec::new();
        for sr in s.kids("SenseRelation") {
            let w = f(sr, "lemma");
            if !w.is_empty() {
                let t = sr.feat("type").unwrap_or("");
                let t = if t.is_empty() { "reference".to_string() } else { kr_rel(t) };
                rel.push(json!({"type": t, "word": w}));
            }
        }
        if !rel.is_empty() {
            sense.insert("rel".into(), rel.into());
        }
        if !sense.is_empty() {
            senses.push(Value::Object(sense));
        }
    }

    let senses_len = senses.len();
    let mut data = Map::new();
    data.insert("senses".into(), senses.into());
    if !related.is_empty() {
        data.insert("related".into(), related.into());
    }
    let cat = f(le, "semanticCategory");
    if !cat.is_empty() {
        data.insert("category".into(), cat.into());
    }
    if let Some(o) = origin_note {
        data.insert("origin_note".into(), o.into());
    }

    // conjugation pointer entry ("배-": "(배고, 배어)→ 배다 1, 배다 2")
    let mut pointer: Option<Pointer> = None;
    if ko_pos == "품사 없음" && kind == "word" && headword.ends_with('-') && !ko_defs.is_empty() && ko_defs.len() == senses_len {
        let mut all = Pointer::default();
        let mut ok = true;
        for d in &ko_defs {
            match parse_pointer(d) {
                Some(p) => {
                    all.forms.extend(p.forms);
                    all.targets.extend(p.targets);
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            pointer = Some(all);
        }
    }

    Some(Entry {
        source: "krdict",
        lang: "en",
        ext_id: le.attr("val").unwrap_or("").to_string(),
        headword,
        homonym,
        hanja,
        pos,
        ko_pos,
        pron: if prons.is_empty() { None } else { Some(prons.join(", ")) },
        level,
        kind,
        forms,
        data: Value::Object(data),
        ko_defs,
        pointer,
    })
}

pub fn parse_file(path: &Path, mut cb: impl FnMut(Entry) -> Result<()>) -> Result<()> {
    for_each_file(path, "LexicalEntry", |n| {
        if let Some(e) = parse_entry(n) {
            cb(e)?;
        }
        Ok(())
    })
}
