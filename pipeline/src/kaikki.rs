//! kaikki.org (wiktextract) Korean JSONL parser.

use crate::common::*;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

#[derive(Default)]
pub struct Kaikki {
    pub entries: Vec<Entry>,
    /// (form, target) from form-of / alt-of senses
    pub redirects: Vec<(String, String)>,
    /// (ko, en) example sentence pairs
    pub sentences: Vec<(String, String)>,
    /// 훈음 of single Han characters (`word` = one hanja, `pos` character/hanja), merged per character.
    pub hanja: HashMap<char, HanjaHun>,
}

/// 훈음 collected for one hanja character from the Korean Wiktionary entries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HanjaHun {
    /// Korean gloss words in source order (first = primary), e.g. "배울".
    pub hun: Vec<String>,
    /// "hun eum" pairs in source order, e.g. "배울 학".
    pub eumhun: Vec<String>,
    /// Hangul readings (eum) in source order, e.g. "학".
    pub eum: Vec<String>,
}

fn is_syllables(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

/// Split "배울 학" (hun, eum) into `("배울", "학")`: whitespace-separated Hangul words, the last one a
/// single syllable. Trailing romanisation in parentheses ("배울 학 (baeul hak)") is ignored.
pub fn split_eumhun(text: &str) -> Option<(String, String)> {
    let t = text.split('(').next().unwrap_or("").trim().trim_end_matches([',', ';']);
    let toks: Vec<&str> = t.split_whitespace().collect();
    if toks.len() < 2 || !toks.iter().all(|w| is_syllables(w)) {
        return None;
    }
    let eum = *toks.last()?;
    if eum.chars().count() != 1 {
        return None;
    }
    Some((toks[..toks.len() - 1].join(" "), eum.to_string()))
}

/// Hangul words following one of the labels "eumhun" / "hangeul" / "hun-eum" in a head-template
/// expansion such as `學 (eumhun 배울 학 (baeul hak), revised baeul hak)`.
fn eumhun_in_expansion(exp: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for label in ["eumhun", "hun-eum", "hunyeum", "hangeul", "hangul"] {
        let mut rest = exp;
        while let Some(i) = rest.find(label) {
            rest = &rest[i + label.len()..];
            let mut words: Vec<&str> = Vec::new();
            for w in rest.trim_start_matches([' ', ':']).split_whitespace() {
                let bare = w.trim_end_matches([',', ';', ')']);
                if !is_syllables(bare) {
                    break;
                }
                words.push(bare);
                if bare.len() != w.len() {
                    break; // a comma / bracket ends the list
                }
            }
            if let Some(p) = split_eumhun(&words.join(" ")) {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        }
    }
    out
}

/// All (hun, eum) pairs and bare readings of one wiktextract entry of a single Han character.
/// Looks at (1) `{{ko-hanja}}` head-template arguments (1 = hun, 2 = eum, or "hun eum" in one
/// argument), (2) the template expansion text, (3) `forms[]` tagged eumhun / hun-eum, (4) glosses
/// shaped like "학 (hak): learn" (reading only). Pairs are returned in source order.
pub fn extract_hun_eum(v: &Value) -> (Vec<(String, String)>, Vec<String>) {
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut eums: Vec<String> = Vec::new();
    let mut add = |p: (String, String), pairs: &mut Vec<(String, String)>| {
        if !pairs.contains(&p) {
            pairs.push(p);
        }
    };
    for ht in v.get("head_templates").and_then(Value::as_array).into_iter().flatten() {
        let name = s(ht, "name");
        let args = ht.get("args");
        let arg = |k: &str| args.and_then(|a| a.get(k)).and_then(Value::as_str).map(str::trim).unwrap_or("");
        if name.starts_with("ko-hanja") || name.starts_with("ko-hanja-") {
            let (a1, a2) = (arg("1"), arg("2"));
            if is_syllables(a1) && is_syllables(a2) && a2.chars().count() == 1 {
                add((a1.to_string(), a2.to_string()), &mut pairs);
            } else if let Some(p) = split_eumhun(a1) {
                add(p, &mut pairs);
            } else if let Some(p) = split_eumhun(arg("hangeul")) {
                add(p, &mut pairs);
            } else if is_syllables(a1) && a1.chars().count() == 1 {
                eums.push(a1.to_string());
            }
        }
        for p in eumhun_in_expansion(s(ht, "expansion")) {
            add(p, &mut pairs);
        }
    }
    for f in v.get("forms").and_then(Value::as_array).into_iter().flatten() {
        let ft = tags(f);
        if ft.iter().any(|t| t.contains("eumhun") || t.contains("hun-eum") || t == "hangeul") {
            if let Some(p) = split_eumhun(&clean(s(f, "form"))) {
                add(p, &mut pairs);
            }
        }
    }
    for sn in v.get("senses").and_then(Value::as_array).into_iter().flatten() {
        for g in sn.get("glosses").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            let g = g.trim();
            let mut it = g.chars();
            if let (Some(c), Some(' '), Some('(')) = (it.next(), it.next(), it.next()) {
                if is_syllables(&c.to_string()) {
                    eums.push(c.to_string());
                }
            }
        }
    }
    for (_, e) in &pairs {
        eums.insert(0, e.clone());
    }
    let mut seen = HashSet::new();
    eums.retain(|e| seen.insert(e.clone()));
    (pairs, eums)
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn words(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|x| clean(s(x, "word"))).filter(|w| !w.is_empty()).collect())
        .unwrap_or_default()
}

fn tags(v: &Value) -> Vec<String> {
    v.get("tags").and_then(Value::as_array).map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect()).unwrap_or_default()
}

struct Acc {
    entry: Entry,
    poses: Vec<String>,
    sense_pos: Vec<String>, // parallel to data.senses
    senses: Vec<Value>,
    related: Vec<Value>,
    etyms: Vec<String>,
}

pub fn parse<R: BufRead>(rd: R) -> Result<Kaikki> {
    let mut out = Kaikki::default();
    let mut idx: HashMap<(String, i64), usize> = HashMap::new();
    let mut accs: Vec<Acc> = Vec::new();
    let mut seen_sent: HashSet<String> = HashSet::new();

    for line in rd.split(b'\n') {
        let line = line?;
        if line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }
        let Ok(v) = serde_json::from_slice::<Value>(&line) else { continue };
        if let Some(lc) = v.get("lang_code").and_then(Value::as_str) {
            if lc != "ko" {
                continue;
            }
        }
        let word = clean(s(&v, "word"));
        if word.chars().count() == 1 && word.chars().all(is_cjk) {
            // a hanja character entry: only its 훈음 is kept
            let ch = word.chars().next().unwrap();
            let (pairs, eums) = extract_hun_eum(&v);
            if !pairs.is_empty() || !eums.is_empty() {
                let h = out.hanja.entry(ch).or_default();
                for (hun, eum) in pairs {
                    let eh = format!("{hun} {eum}");
                    if !h.eumhun.contains(&eh) {
                        h.eumhun.push(eh);
                    }
                    if !h.hun.contains(&hun) {
                        h.hun.push(hun);
                    }
                }
                for e in eums {
                    if !h.eum.contains(&e) {
                        h.eum.push(e);
                    }
                }
            }
            continue;
        }
        if word.is_empty() || !has_hangul(&word) {
            continue;
        }
        let pos = wikt_pos(s(&v, "pos"));
        let etym_no = v.get("etymology_number").and_then(Value::as_i64).unwrap_or(0);

        let mut senses: Vec<(Value, String)> = Vec::new();
        for sn in v.get("senses").and_then(Value::as_array).into_iter().flatten() {
            let stags = tags(sn);
            // form-of / alt-of: record redirect, skip the sense
            let target = ["form_of", "alt_of"].iter().find_map(|k| {
                sn.get(*k).and_then(Value::as_array).and_then(|a| a.first()).map(|x| clean(s(x, "word"))).filter(|w| !w.is_empty())
            });
            let is_form = target.is_some() || stags.iter().any(|t| t == "form-of" || t == "alt-of");
            if is_form {
                if let Some(t) = target {
                    if t != word && has_hangul(&t) {
                        out.redirects.push((word.clone(), t));
                    }
                }
                continue;
            }
            let glosses: Vec<String> = sn
                .get("glosses")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|g| g.as_str()).map(clean).filter(|g| !g.is_empty()).collect())
                .unwrap_or_default();
            let Some(gloss) = glosses.last().cloned() else { continue };
            let mut sense = Map::new();
            sense.insert("gloss".into(), gloss.into());
            if glosses.len() > 1 {
                sense.insert("parent".into(), glosses[..glosses.len() - 1].join(" > ").into());
            }
            let stags: Vec<String> = stags.into_iter().filter(|t| t != "no-gloss").collect();
            if !stags.is_empty() {
                sense.insert("tags".into(), stags.into());
            }
            let mut exs = Vec::new();
            for ex in sn.get("examples").and_then(Value::as_array).into_iter().flatten() {
                let ko = clean(s(ex, "text"));
                if ko.is_empty() || !has_hangul(&ko) {
                    continue;
                }
                let en = {
                    let e = clean(s(ex, "english"));
                    if e.is_empty() { clean(s(ex, "translation")) } else { e }
                };
                let mut o = Map::new();
                o.insert("ko".into(), ko.clone().into());
                if !en.is_empty() {
                    o.insert("en".into(), en.clone().into());
                    if seen_sent.insert(ko.clone()) {
                        out.sentences.push((ko, en));
                    }
                }
                exs.push(Value::Object(o));
                if exs.len() >= 5 {
                    break;
                }
            }
            if !exs.is_empty() {
                sense.insert("examples".into(), exs.into());
            }
            let mut rel = Vec::new();
            for (k, ty) in [("synonyms", "synonym"), ("antonyms", "antonym")] {
                for w in words(sn, k) {
                    rel.push(json!({"type": ty, "word": w}));
                }
            }
            if !rel.is_empty() {
                sense.insert("rel".into(), rel.into());
            }
            senses.push((Value::Object(sense), pos.to_string()));
        }
        if senses.is_empty() {
            continue;
        }

        let key = (word.clone(), etym_no);
        let i = *idx.entry(key).or_insert_with(|| {
            let kind = kind_for_pos(pos);
            accs.push(Acc {
                entry: Entry {
                    source: "wikt",
                    lang: "en",
                    ext_id: format!("{word}#{etym_no}"),
                    headword: word.clone(),
                    homonym: if etym_no > 0 { Some(etym_no) } else { None },
                    pos: pos.to_string(),
                    kind: kind.to_string(),
                    ..Default::default()
                },
                poses: Vec::new(),
                sense_pos: Vec::new(),
                senses: Vec::new(),
                related: Vec::new(),
                etyms: Vec::new(),
            });
            accs.len() - 1
        });
        let a = &mut accs[i];
        if !a.poses.iter().any(|p| p == pos) {
            a.poses.push(pos.to_string());
        }
        for (sn, p) in senses {
            a.senses.push(sn);
            a.sense_pos.push(p);
        }

        for f in v.get("forms").and_then(Value::as_array).into_iter().flatten() {
            let form = clean(s(f, "form"));
            if form.is_empty() {
                continue;
            }
            let ft = tags(f);
            if ft.iter().any(|t| t == "hanja") {
                if a.entry.hanja.is_none() && has_cjk(&form) {
                    a.entry.hanja = Some(form);
                }
            } else if has_hangul(&form) && form != word && !ft.iter().any(|t| t == "romanization") && !a.entry.forms.contains(&form) {
                a.entry.forms.push(form);
            }
        }
        let et = clean(s(&v, "etymology_text"));
        if !et.is_empty() && !a.etyms.contains(&et) {
            a.etyms.push(et);
        }
        for (k, ty) in [("synonyms", "synonym"), ("antonyms", "antonym"), ("derived", "derived"), ("related", "see also")] {
            for w in words(&v, k) {
                let item = json!({"type": ty, "word": w});
                if !a.related.contains(&item) && a.related.len() < 40 {
                    a.related.push(item);
                }
            }
        }
        if a.entry.pron.is_none() {
            for snd in v.get("sounds").and_then(Value::as_array).into_iter().flatten() {
                let h = clean(s(snd, "hangeul"));
                if !h.is_empty() {
                    a.entry.pron = Some(h);
                    break;
                }
            }
        }
    }

    for mut a in accs {
        let multi = a.poses.len() > 1;
        let mut senses = Vec::new();
        for (mut sn, p) in a.senses.into_iter().zip(a.sense_pos) {
            if multi {
                sn.as_object_mut().unwrap().insert("pos".into(), p.into());
            }
            senses.push(sn);
        }
        let mut data = Map::new();
        data.insert("senses".into(), senses.into());
        if !a.related.is_empty() {
            data.insert("related".into(), a.related.into());
        }
        if !a.etyms.is_empty() {
            data.insert("etym".into(), a.etyms.join("\n").into());
        }
        a.entry.data = Value::Object(data);
        out.entries.push(a.entry);
    }
    Ok(out)
}
