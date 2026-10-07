//! kengdic TSV parser (id, surface, hanja, gloss, level, created, source).

use crate::common::*;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::io::BufRead;

fn edge_start(c: char) -> bool {
    c.is_whitespace() || ",;:.-–—/\\|\"'`~*+=".contains(c)
}
fn edge_end(c: char) -> bool {
    c.is_whitespace() || ",;:-–—/\\|\"'`~*+=".contains(c)
}

pub fn clean_gloss(g: &str) -> String {
    let g: String = g.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    let g = clean(&g);
    let g = g.trim_start_matches(edge_start).trim_end_matches(edge_end);
    // fix "a ,b" spacing artefacts
    let mut out = String::with_capacity(g.len());
    let mut pending_ws = false;
    for c in g.chars() {
        if c == ' ' {
            pending_ws = true;
            continue;
        }
        if pending_ws && c != ',' {
            out.push(' ');
        }
        pending_ws = false;
        out.push(c);
    }
    out
}

const PROPER: &[&str] = &[
    "Korea", "Korean", "Koreans", "Seoul", "Pacific", "Atlantic", "Japan", "Japanese", "China", "Chinese", "English", "America", "American",
    "Asia", "Asian", "Europe", "European", "Buddha", "Buddhist", "Buddhism", "God", "Christ", "Christian", "Christianity", "Confucian",
    "Confucius", "Catholic", "Islam", "Muslim", "Jesus", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December",
    "Russia", "Russian", "France", "French", "Germany", "German", "England", "British", "Britain", "India", "Indian", "Mongol", "Mongolia",
    "Manchu", "Manchuria", "Pyongyang", "Busan", "Jeju", "Silla", "Goryeo", "Joseon", "Baekje", "Goguryeo", "Hanguk", "Hangeul", "Hangul",
    "Latin", "Greek", "Sanskrit", "Taoism", "Taoist", "Shinto", "Africa", "African", "Australia", "Canada", "Mr", "Mrs", "Ms", "Dr",
];

/// Lowercase the first letter of each "; "-separated segment when it is just sentence-initial
/// capitalisation (heuristic: first word is Capitalised + lowercase rest, not a known proper noun).
pub fn normalize_case(g: &str) -> String {
    g.split("; ").map(lower_first).collect::<Vec<_>>().join("; ")
}

fn lower_first(seg: &str) -> String {
    let first_word: String = seg.chars().take_while(|c| c.is_ascii_alphabetic() || *c == '\'').collect();
    let mut cs = first_word.chars();
    let Some(c0) = cs.next() else { return seg.to_string() };
    let rest: String = cs.collect();
    if !c0.is_ascii_uppercase() || PROPER.contains(&first_word.as_str()) || first_word.contains('\'') && first_word.starts_with("I'") {
        return seg.to_string();
    }
    let rest_lower = rest.chars().all(|c| c.is_ascii_lowercase());
    // keep "I", acronyms (TV, DNA) and mixed-case words
    if first_word == "I" || !rest_lower || (rest.is_empty() && first_word != "A") {
        return seg.to_string();
    }
    format!("{}{}", c0.to_ascii_lowercase(), &seg[1..])
}

fn is_junk(g: &str) -> bool {
    let l = g.to_lowercase();
    l.contains("adds no meaning") || l.starts_with("vst +") || l.starts_with("vs +")
}

fn guess_pos(surface: &str, gloss: &str) -> &'static str {
    let low = gloss.to_lowercase();
    if surface.ends_with('다') && !surface.contains(' ') {
        if low.starts_with("to ") {
            return "verb";
        }
        if low.starts_with("be ") {
            return "adjective";
        }
    }
    "other"
}

fn level_ord(l: &str) -> Option<u8> {
    match l {
        "A" => Some(0),
        "B" => Some(1),
        "C" => Some(2),
        "D" => Some(3),
        _ => None,
    }
}

pub struct Kengdic {
    pub entries: Vec<Entry>,
    /// surface -> distinct first-hanja strings (all rows, including gloss-less)
    pub hanja_by_surface: HashMap<String, BTreeSet<String>>,
}

struct Group {
    ext_id: String,
    surface: String,
    hanja: String,
    alt: Vec<String>,
    glosses: Vec<(String, String)>, // (lowercase, original) in order
    klevel: Option<u8>,
}

pub fn parse<R: BufRead>(mut rd: R) -> Result<Kengdic> {
    let mut idx: HashMap<(String, String), usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut hanja_by_surface: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut raw = Vec::new();
    let mut first = true;
    loop {
        raw.clear();
        if rd.read_until(b'\n', &mut raw)? == 0 {
            break;
        }
        if first {
            first = false;
            continue; // header
        }
        let line = String::from_utf8_lossy(&raw);
        let line = line.trim_end_matches(['\n', '\r']);
        let row: Vec<&str> = line.split('\t').collect();
        if row.len() < 4 {
            continue;
        }
        let surface: String = clean(&row[1].chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>());
        if surface.is_empty() || !has_hangul(&surface) {
            continue;
        }
        let variants: Vec<String> = row[2].split(',').map(clean).filter(|v| !v.is_empty() && has_cjk(v)).collect();
        let hanja = variants.first().cloned().unwrap_or_default();
        if !hanja.is_empty() {
            hanja_by_surface.entry(surface.clone()).or_default().insert(hanja.clone());
        }
        let gloss = clean_gloss(row[3]);
        if gloss.is_empty() || gloss.chars().count() > 250 || is_junk(&gloss) {
            continue;
        }
        // quality filters: no Latin letters in a Korean headword, no gloss without
        // ASCII letters, and glosses that merely repeat the headword
        if surface.chars().any(|c| c.is_ascii_alphabetic())
            || !gloss.chars().any(|c| c.is_ascii_alphabetic())
            || gloss.to_lowercase() == surface.to_lowercase()
        {
            continue;
        }
        let gloss = normalize_case(&gloss);
        let level = row.get(4).map(|s| s.trim().to_uppercase()).unwrap_or_default();
        let key = (surface.clone(), hanja.clone());
        let i = *idx.entry(key).or_insert_with(|| {
            groups.push(Group {
                ext_id: row[0].to_string(),
                surface: surface.clone(),
                hanja: hanja.clone(),
                alt: variants.iter().skip(1).cloned().collect(),
                glosses: Vec::new(),
                klevel: None,
            });
            groups.len() - 1
        });
        let g = &mut groups[i];
        let low = gloss.to_lowercase();
        if !g.glosses.iter().any(|(l, _)| *l == low) {
            g.glosses.push((low, gloss));
        }
        if let Some(lv) = level_ord(&level) {
            if g.klevel.is_none_or(|k| lv < k) {
                g.klevel = Some(lv);
            }
        }
    }

    let mut entries = Vec::with_capacity(groups.len());
    for g in groups {
        let phrase = g.surface.contains(' ');
        let pos = if phrase { "phrase" } else { guess_pos(&g.surface, &g.glosses[0].1) };
        let mut data = Map::new();
        data.insert("senses".into(), g.glosses.iter().map(|(_, o)| json!({"gloss": o})).collect::<Vec<_>>().into());
        if !g.alt.is_empty() {
            data.insert("hanja_alt".into(), json!(g.alt));
        }
        if let Some(k) = g.klevel {
            data.insert("kengdic_level".into(), ["A", "B", "C", "D"][k as usize].into());
        }
        entries.push(Entry {
            source: "kengdic",
            lang: "en",
            ext_id: g.ext_id,
            headword: g.surface,
            hanja: if g.hanja.is_empty() { None } else { Some(g.hanja) },
            pos: pos.to_string(),
            kind: if phrase { "phrase" } else { "word" }.to_string(),
            data: Value::Object(data),
            ..Default::default()
        });
    }
    Ok(Kengdic { entries, hanja_by_surface })
}
