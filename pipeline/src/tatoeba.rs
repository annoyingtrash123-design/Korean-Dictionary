//! Tatoeba Korean-English sentence pairs.

use crate::common::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};

pub struct Pair {
    pub ko: String,
    pub en: String,
}

/// Read `id \t lang \t text` lines, optionally keeping only ids in `wanted`.
pub fn read_sentences<R: Read>(r: R, wanted: Option<&HashSet<String>>) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    let mut br = BufReader::new(r);
    let mut raw = Vec::new();
    loop {
        raw.clear();
        if br.read_until(b'\n', &mut raw)? == 0 {
            break;
        }
        let line = String::from_utf8_lossy(&raw);
        let mut p = line.trim_end_matches(['\r', '\n']).splitn(3, '\t');
        let (Some(id), Some(_lang), Some(text)) = (p.next(), p.next(), p.next()) else { continue };
        if wanted.is_none_or(|w| w.contains(id)) {
            out.insert(id.to_string(), text.to_string());
        }
    }
    Ok(out)
}

/// Stream (sentence_id, translation_id) pairs from a plain `links.csv` stream.
pub fn iter_links<R: Read>(r: R, mut f: impl FnMut(&str, &str)) -> Result<()> {
    let mut br = BufReader::new(r);
    let mut raw = Vec::new();
    loop {
        raw.clear();
        if br.read_until(b'\n', &mut raw)? == 0 {
            break;
        }
        let line = String::from_utf8_lossy(&raw);
        let mut p = line.trim_end_matches(['\r', '\n']).split('\t');
        if let (Some(a), Some(b)) = (p.next(), p.next()) {
            f(a, b);
        }
    }
    Ok(())
}

/// links.tar(.bz2) contains `links.csv`; the reader here is the already
/// bz2-decompressed tar stream.
pub fn iter_links_tar<R: Read>(r: R, f: impl FnMut(&str, &str)) -> Result<()> {
    let mut ar = tar::Archive::new(r);
    for entry in ar.entries()? {
        let entry = entry?;
        if entry.header().entry_type().is_file() && entry.path()?.to_string_lossy().ends_with(".csv") {
            return iter_links(entry, f);
        }
    }
    Ok(())
}

/// Combine kor sentences, eng sentences and a links stream callback.
/// `links` must call its argument for every (a, b) link.
pub fn pairs<K: Read, E: Read>(
    kor: K,
    eng: E,
    links: impl FnOnce(&mut dyn FnMut(&str, &str)) -> Result<()>,
) -> Result<Vec<Pair>> {
    let kor = read_sentences(kor, None)?;
    let mut cand: HashMap<String, Vec<String>> = HashMap::new();
    links(&mut |a, b| {
        if kor.contains_key(a) {
            cand.entry(a.to_string()).or_default().push(b.to_string());
        }
    })?;
    let need: HashSet<String> = cand.values().flatten().cloned().collect();
    let eng = read_sentences(eng, Some(&need))?;
    let mut ids: Vec<&String> = kor.keys().collect();
    ids.sort_by_key(|k| k.parse::<u64>().unwrap_or(0));
    let mut out = Vec::new();
    for kid in ids {
        let ko = clean(&kor[kid]);
        if ko.is_empty() || !has_hangul(&ko) {
            continue;
        }
        let en = cand.get(kid).and_then(|bs| bs.iter().find_map(|b| eng.get(b))).map(|e| clean(e));
        if let Some(en) = en.filter(|e| !e.is_empty()) {
            out.push(Pair { ko, en });
        }
    }
    Ok(out)
}
