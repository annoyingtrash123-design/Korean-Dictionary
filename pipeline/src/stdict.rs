//! stdict (표준국어대사전) XML parser.

use crate::common::*;
use crate::xml::{for_each_file, Node};
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

/// Split a trailing 1-3 digit homonym number ("가03" -> ("가", Some(3))).
pub fn split_homonym(word: &str) -> (String, Option<i64>) {
    let digits = word.chars().rev().take_while(|c| c.is_ascii_digit()).count().min(3);
    // do not strip when the word consists only of digits
    if digits > 0 && word.chars().count() > digits {
        let split = word.len() - digits; // ascii digits are 1 byte each
        let n: i64 = word[split..].parse().unwrap_or(0);
        return (word[..split].to_string(), if n == 0 { None } else { Some(n) });
    }
    (word.to_string(), None)
}

/// Remove '-' (morpheme) and '^' (space) markers; keep leading/trailing '-'
/// that denote affixes/endings. '^' becomes a plain space.
pub fn display_word(raw: &str) -> String {
    let raw = raw.trim();
    let lead = raw.starts_with('-');
    let trail = raw.chars().count() > 1 && raw.ends_with('-');
    let core: String = raw.trim_matches('-').replace('-', "").replace('^', " ");
    let core = core.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("{}{}{}", if lead { "-" } else { "" }, core, if trail { "-" } else { "" })
}

fn rel_word(w: &str) -> String {
    display_word(&split_homonym(w).0)
}

fn t(n: &Node, path: &str) -> String {
    clean(&n.t(path))
}

pub fn parse_item(item: &Node) -> Option<Entry> {
    let wi = item.child("word_info")?;
    let (raw, homonym) = split_homonym(&t(wi, "word"));
    let headword = display_word(&raw);
    if headword.is_empty() {
        return None;
    }
    let unit = { let u = t(wi, "word_unit"); if u.is_empty() { "단어".to_string() } else { u } };

    let mut parts: Vec<(String, String)> = Vec::new();
    let mut has_hanja_part = false;
    for ol in wi.kids("original_language_info") {
        let txt = t(ol, "original_language");
        let lt = t(ol, "language_type");
        if txt.is_empty() {
            continue;
        }
        if lt == "한자" {
            has_hanja_part = true;
        }
        parts.push((txt, lt));
    }
    let (mut hanja, mut origin_note) = (None, None);
    if has_hanja_part {
        let cand: String = parts.iter().filter(|(_, lt)| lt == "한자" || lt == "고유어").map(|(t, _)| t.as_str()).collect();
        if has_cjk(&cand) {
            hanja = Some(cand);
        }
    } else if !parts.is_empty() {
        let v: Vec<String> = parts
            .iter()
            .map(|(t, lt)| if !lt.is_empty() && lt != "안 밝힘" && lt != "/(병기)" { format!("{lt}: {t}") } else { t.clone() })
            .collect();
        origin_note = Some(v.join("; "));
    }

    let mut prons: Vec<String> = Vec::new();
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

    let mut related = Vec::new();
    for tag in ["lexical_info", "relation_info"] {
        for li in wi.kids(tag) {
            let w = rel_word(&t(li, "word"));
            if !w.is_empty() {
                related.push(json!({"type": kr_rel(&t(li, "type")), "word": w}));
            }
        }
    }

    let mut senses = Vec::new();
    let mut cats: Vec<String> = Vec::new();
    let mut ko_pos_first: Option<String> = None;
    let multi_pos = wi.kids("pos_info").count() > 1;
    let mut ko_defs = Vec::new();
    for pi in wi.kids("pos_info") {
        let ko_pos = t(pi, "pos");
        if ko_pos_first.is_none() {
            ko_pos_first = Some(ko_pos.clone());
        }
        let sense_pos = kr_pos_or_other(&ko_pos);
        for cp in pi.kids("comm_pattern_info") {
            let pattern = t(cp, "pattern_info/pattern");
            let gram: Vec<String> = cp.find_all("grammar_info/grammar").iter().map(|g| clean(&g.text)).filter(|g| !g.is_empty()).collect();
            let gram = gram.join("; ");
            for si in cp.kids("sense_info") {
                let mut sense = Map::new();
                if multi_pos {
                    sense.insert("pos".into(), sense_pos.into());
                }
                let kd = t(si, "definition");
                if !kd.is_empty() {
                    sense.insert("ko_def".into(), kd.clone().into());
                    ko_defs.push(kd);
                }
                if !gram.is_empty() {
                    sense.insert("note".into(), gram.clone().into());
                }
                if !pattern.is_empty() {
                    sense.insert("pattern".into(), pattern.clone().into());
                }
                let ty = t(si, "type");
                let mut tags: Vec<String> = Vec::new();
                if !ty.is_empty() && ty != "일반어" {
                    tags.push(ty);
                }
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
                for li in si.kids("lexical_info") {
                    let w = rel_word(&t(li, "word"));
                    if !w.is_empty() {
                        rel.push(json!({"type": kr_rel(&t(li, "type")), "word": w}));
                    }
                }
                if !rel.is_empty() {
                    sense.insert("rel".into(), rel.into());
                }
                if !sense.is_empty() {
                    senses.push(Value::Object(sense));
                }
            }
        }
    }

    let ko_pos = ko_pos_first.unwrap_or_default();
    let (pos, mut kind) = match unit.as_str() {
        "구" => ("phrase".to_string(), "phrase".to_string()),
        "속담" => ("proverb".into(), "proverb".into()),
        "관용구" => ("idiom".into(), "idiom".into()),
        _ => {
            let p = kr_pos_or_other(&ko_pos).to_string();
            let k = if p == "phrase" { "phrase" } else { "word" };
            (p, k.to_string())
        }
    };
    if kind == "word" && (ko_pos == "어미" || ko_pos == "조사") {
        kind = "grammar".into();
    }

    let mut data = Map::new();
    data.insert("senses".into(), senses.into());
    if !related.is_empty() {
        data.insert("related".into(), related.into());
    }
    if !cats.is_empty() {
        data.insert("category".into(), cats.join(" / ").into());
    }
    if let Some(o) = origin_note {
        data.insert("origin_note".into(), o.into());
    }
    let o = t(wi, "origin");
    if !o.is_empty() {
        data.insert("etym".into(), o.into());
    }
    let al = t(wi, "allomorph");
    if !al.is_empty() {
        data.insert("allomorph".into(), al.into());
    }

    Some(Entry {
        source: "stdict",
        lang: "ko",
        ext_id: t(item, "target_code"),
        headword,
        homonym,
        hanja,
        pos,
        ko_pos,
        pron: if prons.is_empty() { None } else { Some(prons.join(", ")) },
        level: None,
        kind,
        forms,
        data: Value::Object(data),
        ko_defs,
    })
}

pub fn parse_file(path: &Path, mut cb: impl FnMut(Entry) -> Result<()>) -> Result<()> {
    for_each_file(path, "item", |n| {
        if let Some(e) = parse_item(n) {
            cb(e)?;
        }
        Ok(())
    })
}
