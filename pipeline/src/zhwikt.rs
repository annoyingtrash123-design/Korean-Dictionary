//! kaikki.org (wiktextract) Chinese JSONL -> `zhwikt` pack entries.
//!
//! The file is multi-GB, so it is **streamed**: lines of one `word` are contiguous in kaikki
//! output, so they are buffered until the word changes, merged per etymology and handed to the
//! callback; nothing else is kept in memory (only the small redirect list). Dropped to keep the
//! pack small: translations, inflection/desc tables, examples, categories, long etymologies.

use crate::cedict::sino_korean;
use crate::common::*;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::BufRead;

const MAX_SENSES: usize = 30;
const MAX_GLOSS: usize = 300;
const MAX_ETYM: usize = 240;
const MAX_PER_KIND: usize = 6;

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect()).unwrap_or_default()
}

/// Pronunciation buckets of an entry (`data.pron`).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Prons {
    pub mandarin: Vec<String>,
    pub middle_chinese: Vec<String>,
    pub cantonese: Vec<String>,
    pub sino_korean: Vec<String>,
    pub sino_vietnamese: Vec<String>,
    pub sino_japanese: Vec<String>,
}

fn push_cap(v: &mut Vec<String>, x: String) {
    if !x.is_empty() && !v.contains(&x) && v.len() < MAX_PER_KIND {
        v.push(x);
    }
}

/// Hangul syllables of a "학 (hak)" style value.
fn hangul_part(v: &str) -> String {
    v.split(|c: char| !is_hangul(c) && c != ' ').next().unwrap_or("").trim().to_string()
}

/// Sort the `sounds[]` of one line into [`Prons`]. Classification uses the lower-cased `tags` and
/// `raw_tags` (wiktextract writes "Middle-Chinese" / "Middle Chinese" / "Sino-Korean" in either);
/// the value is the `zh-pron` / `other` / `pinyin` field (IPA is ignored).
pub fn collect_sounds(v: &Value, p: &mut Prons) {
    for snd in v.get("sounds").and_then(Value::as_array).into_iter().flatten() {
        let mut t = strs(snd, "tags");
        t.extend(strs(snd, "raw_tags"));
        let t = t.join(" ").to_lowercase().replace('-', " ");
        let val = ["zh-pron", "zh_pron", "other", "pinyin", "hangeul", "romanization"]
            .iter()
            .map(|k| clean(s(snd, k)))
            .find(|x| !x.is_empty())
            .unwrap_or_default();
        if val.is_empty() {
            continue;
        }
        if t.contains("sino korean") {
            // any field may hold the hangul ("other": "학교 (hakgyo)")
            let h = ["zh-pron", "zh_pron", "other", "hangeul", "pinyin", "romanization"].iter().map(|k| hangul_part(&clean(s(snd, k)))).find(|x| !x.is_empty());
            push_cap(&mut p.sino_korean, h.unwrap_or_default());
        } else if t.contains("sino vietnamese") {
            push_cap(&mut p.sino_vietnamese, val);
        } else if t.contains("sino japanese") || t.contains("go on") || t.contains("kan on") || t.contains("on yomi") {
            push_cap(&mut p.sino_japanese, val);
        } else if t.contains("middle chinese") {
            push_cap(&mut p.middle_chinese, val);
        } else if t.contains("cantonese") || t.contains("jyutping") {
            push_cap(&mut p.cantonese, val);
        } else if t.contains("mandarin") || t.contains("pinyin") {
            push_cap(&mut p.mandarin, val);
        }
    }
}

fn prons_json(p: &Prons) -> Option<Value> {
    let mut m = Map::new();
    for (k, v) in [
        ("mandarin", &p.mandarin),
        ("middle_chinese", &p.middle_chinese),
        ("cantonese", &p.cantonese),
        ("sino_korean", &p.sino_korean),
        ("sino_vietnamese", &p.sino_vietnamese),
        ("sino_japanese", &p.sino_japanese),
    ] {
        if !v.is_empty() {
            m.insert(k.into(), json!(v));
        }
    }
    (!m.is_empty()).then(|| Value::Object(m))
}

/// Chinese-specific POS names not covered by [`wikt_pos`].
fn zh_pos(p: &str) -> &'static str {
    match p {
        "character" | "hanzi" | "han" | "symbol" => "other",
        "conj" | "prep" | "postp" => wikt_pos(p),
        other => wikt_pos(other),
    }
}

struct Acc {
    entry: Entry,
    poses: Vec<&'static str>,
    senses: Vec<(Value, &'static str)>,
    prons: Prons,
    etym: Vec<String>,
    classical: bool,
}

/// Merge the buffered lines of one word. Entries go to `cb`; form-of redirects to `redirects`.
fn flush(
    word: &str,
    lines: &[Value],
    sino: &HashMap<char, String>,
    seq: &mut usize,
    redirects: &mut Vec<(String, String)>,
    cb: &mut impl FnMut(Entry, i64) -> Result<()>,
) -> Result<()> {
    let mut accs: Vec<(i64, Acc)> = Vec::new();
    for v in lines {
        if let Some(lc) = v.get("lang_code").and_then(Value::as_str) {
            if lc != "zh" {
                continue;
            }
        }
        let pos = zh_pos(s(v, "pos"));
        let etym_no = v.get("etymology_number").and_then(Value::as_i64).unwrap_or(0);
        let mut senses: Vec<(Value, &'static str)> = Vec::new();
        for sn in v.get("senses").and_then(Value::as_array).into_iter().flatten() {
            let target = ["form_of", "alt_of"].iter().find_map(|k| {
                sn.get(*k).and_then(Value::as_array).and_then(|a| a.first()).map(|x| clean(s(x, "word"))).filter(|w| !w.is_empty())
            });
            let mut tags = strs(sn, "tags");
            tags.extend(strs(sn, "raw_tags"));
            if target.is_some() || tags.iter().any(|t| t == "form-of" || t == "alt-of") {
                if let Some(t) = target {
                    if t != word && has_cjk(&t) {
                        redirects.push((word.to_string(), t));
                    }
                }
                continue;
            }
            let glosses: Vec<String> = strs(sn, "glosses").iter().map(|g| clean(g)).filter(|g| !g.is_empty()).collect();
            let Some(gloss) = glosses.last().cloned() else { continue };
            let mut o = Map::new();
            o.insert("gloss".into(), truncate(&gloss, MAX_GLOSS).into());
            if glosses.len() > 1 {
                o.insert("parent".into(), truncate(&glosses[..glosses.len() - 1].join(" > "), 120).into());
            }
            tags.retain(|t| t != "no-gloss");
            tags.dedup();
            tags.truncate(6);
            if !tags.is_empty() {
                o.insert("tags".into(), tags.into());
            }
            senses.push((Value::Object(o), pos));
        }
        if senses.is_empty() {
            continue;
        }
        let i = match accs.iter().position(|(n, _)| *n == etym_no) {
            Some(i) => i,
            None => {
                accs.push((
                    etym_no,
                    Acc {
                        entry: Entry {
                            source: "zhwikt",
                            lang: "en",
                            ext_id: format!("{word}#{etym_no}"),
                            headword: word.to_string(),
                            hanja: Some(word.to_string()),
                            homonym: (etym_no > 0).then_some(etym_no),
                            pos: pos.to_string(),
                            kind: kind_for_pos(pos).to_string(),
                            ..Default::default()
                        },
                        poses: Vec::new(),
                        senses: Vec::new(),
                        prons: Prons::default(),
                        etym: Vec::new(),
                        classical: false,
                    },
                ));
                accs.len() - 1
            }
        };
        let a = &mut accs[i].1;
        if !a.poses.contains(&pos) {
            a.poses.push(pos);
        }
        for (sn, p) in senses {
            if a.senses.len() < MAX_SENSES {
                let low = sn.get("tags").map(|t| t.to_string().to_lowercase()).unwrap_or_default();
                if low.contains("classical") || low.contains("literary") || low.contains("archaic") {
                    a.classical = true;
                }
                a.senses.push((sn, p));
            }
        }
        collect_sounds(v, &mut a.prons);
        for f in v.get("forms").and_then(Value::as_array).into_iter().flatten() {
            let form = clean(s(f, "form"));
            if form.is_empty() || form == word || !has_cjk(&form) {
                continue;
            }
            let ft = strs(f, "tags");
            if !a.entry.forms.contains(&form) {
                a.entry.forms.push(form.clone());
            }
            let slot = if ft.iter().any(|t| t.eq_ignore_ascii_case("simplified")) {
                Some("simplified")
            } else if ft.iter().any(|t| t.eq_ignore_ascii_case("traditional")) {
                Some("traditional")
            } else {
                None
            };
            if let Some(k) = slot {
                a.entry.data = {
                    let mut d = a.entry.data.as_object().cloned().unwrap_or_default();
                    d.entry(k).or_insert(form.into());
                    Value::Object(d)
                };
            }
        }
        let et = clean(s(v, "etymology_text"));
        if !et.is_empty() && a.etym.len() < 2 && !a.etym.contains(&et) {
            a.etym.push(truncate(&et, MAX_ETYM));
        }
    }
    for (_, mut a) in accs {
        let multi = a.poses.len() > 1;
        let senses: Vec<Value> = a
            .senses
            .into_iter()
            .map(|(mut sn, p)| {
                if multi {
                    sn.as_object_mut().unwrap().insert("pos".into(), p.into());
                }
                sn
            })
            .collect();
        let mut data = a.entry.data.as_object().cloned().unwrap_or_default();
        data.insert("senses".into(), senses.into());
        if let Some(p) = prons_json(&a.prons) {
            data.insert("pron".into(), p);
        }
        if !a.etym.is_empty() {
            data.insert("etym".into(), a.etym.join("\n").into());
        }
        if a.classical {
            data.insert("classical".into(), true.into());
        }
        a.entry.pron = a.prons.sino_korean.first().cloned().or_else(|| sino_korean(word, sino));
        a.entry.data = Value::Object(data);
        let r = crate::cedict::rank(word, *seq);
        *seq += 1;
        cb(a.entry, r)?;
    }
    Ok(())
}

/// Stream a kaikki Chinese JSONL. `cb(entry, rank)` per merged entry; returns the form-of
/// redirects `(form, target)` between Han words (simplified -> traditional, variants).
pub fn parse<R: BufRead>(rd: R, sino: &HashMap<char, String>, mut cb: impl FnMut(Entry, i64) -> Result<()>) -> Result<Vec<(String, String)>> {
    let mut redirects = Vec::new();
    let mut seq = 0usize;
    let mut cur = String::new();
    let mut lines: Vec<Value> = Vec::new();
    for line in rd.split(b'\n') {
        let line = line?;
        if line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }
        let Ok(mut v) = serde_json::from_slice::<Value>(&line) else { continue };
        let word = clean(s(&v, "word"));
        if word.is_empty() || !has_cjk(&word) {
            continue;
        }
        // drop what the pack never keeps before buffering
        if let Some(o) = v.as_object_mut() {
            for k in ["translations", "descendants", "derived", "related", "synonyms", "antonyms", "categories", "topics", "head_templates", "inflection_templates"] {
                o.remove(k);
            }
            if let Some(Value::Array(sn)) = o.get_mut("senses") {
                for x in sn {
                    if let Some(so) = x.as_object_mut() {
                        for k in ["examples", "categories", "topics", "synonyms", "antonyms", "hypernyms", "hyponyms", "coordinate_terms"] {
                            so.remove(k);
                        }
                    }
                }
            }
        }
        if word != cur {
            if !cur.is_empty() {
                flush(&cur, &lines, sino, &mut seq, &mut redirects, &mut cb)?;
            }
            cur = word;
            lines.clear();
        }
        lines.push(v);
    }
    if !cur.is_empty() {
        flush(&cur, &lines, sino, &mut seq, &mut redirects, &mut cb)?;
    }
    Ok(redirects)
}
