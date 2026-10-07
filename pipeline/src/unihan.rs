//! Unihan.zip (or a directory of its txt files) -> per-character info.

use anyhow::Result;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const FILES: [&str; 2] = ["Unihan_Readings.txt", "Unihan_IRGSources.txt"];
const KEEP: [&str; 5] = ["kHangul", "kDefinition", "kKorean", "kTotalStrokes", "kRSUnicode"];

#[derive(Debug, Clone, Default)]
pub struct HanjaInfo {
    pub readings: Vec<String>,
    pub meaning_en: Option<String>,
    pub strokes: Option<i64>,
    pub radical_num: Option<i64>,
    pub radical: Option<String>,
    pub has_hangul: bool,
}

pub fn parse_readers<R: BufRead>(readers: Vec<R>) -> HashMap<char, HanjaInfo> {
    let mut raw: HashMap<char, HashMap<String, String>> = HashMap::new();
    for rd in readers {
        for line in rd.split(b'\n').map_while(Result::ok) {
            let line = String::from_utf8_lossy(&line);
            if !line.starts_with("U+") {
                continue;
            }
            let mut p = line.trim_end_matches(['\r', '\n']).splitn(3, '\t');
            let (Some(cp), Some(field), Some(val)) = (p.next(), p.next(), p.next()) else { continue };
            if !KEEP.contains(&field) {
                continue;
            }
            let Some(ch) = u32::from_str_radix(&cp[2..], 16).ok().and_then(char::from_u32) else { continue };
            raw.entry(ch).or_default().insert(field.to_string(), val.to_string());
        }
    }
    let mut out = HashMap::new();
    for (ch, f) in raw {
        let mut readings: Vec<String> = Vec::new();
        if let Some(h) = f.get("kHangul") {
            for tok in h.split_whitespace() {
                let r = tok.split(':').next().unwrap_or("");
                if !r.is_empty() && !readings.iter().any(|x| x == r) {
                    readings.push(r.to_string());
                }
            }
        }
        if readings.is_empty() {
            if let Some(k) = f.get("kKorean") {
                readings = k.split_whitespace().map(|t| t.to_lowercase()).collect();
            }
        }
        let strokes = f.get("kTotalStrokes").and_then(|s| s.split_whitespace().next()).and_then(|s| s.parse::<i64>().ok());
        let (mut radical_num, mut radical) = (None, None);
        if let Some(rs) = f.get("kRSUnicode").and_then(|s| s.split_whitespace().next()) {
            let num = rs.split('.').next().unwrap_or("").trim_end_matches('\'');
            if let Ok(n) = num.parse::<i64>() {
                if (1..=214).contains(&n) {
                    radical_num = Some(n);
                    radical = char::from_u32(0x2F00 + n as u32 - 1).map(|c| c.to_string());
                }
            }
        }
        let meaning = f.get("kDefinition").cloned();
        if readings.is_empty() && meaning.is_none() && strokes.is_none() {
            continue;
        }
        out.insert(ch, HanjaInfo { readings, meaning_en: meaning, strokes, radical_num, radical, has_hangul: f.contains_key("kHangul") });
    }
    out
}

pub fn parse(path: &Path) -> Result<HashMap<char, HanjaInfo>> {
    let mut readers: Vec<BufReader<Box<dyn Read>>> = Vec::new();
    if path.is_dir() {
        for name in FILES {
            let f = path.join(name);
            if f.exists() {
                readers.push(BufReader::new(Box::new(std::fs::File::open(f)?)));
            }
        }
    } else {
        let mut zf = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        for i in 0..zf.len() {
            let mut f = zf.by_index(i)?;
            let base = f.name().rsplit('/').next().unwrap_or("").to_string();
            if FILES.contains(&base.as_str()) {
                // files are small enough (a few MB) to buffer
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                readers.push(BufReader::new(Box::new(std::io::Cursor::new(buf))));
            }
        }
    }
    Ok(parse_readers(readers))
}

const VARIANT_FILES: [&str; 2] = ["Unihan_Variants.txt", "Unihan_IRGSources.txt"];
/// Variant links that name the *same* character written differently (Korean 敎/Chinese 教,
/// 眞/真, compatibility ideographs). Simplified/traditional links are deliberately left out.
const VARIANT_FIELDS: [&str; 3] = ["kZVariant", "kSemanticVariant", "kCompatibilityVariant"];

/// Map every character to the representative (smallest code point) of its variant class.
/// Characters without variants are absent (they are their own representative).
pub fn variant_classes<R: BufRead>(readers: Vec<R>) -> HashMap<char, char> {
    fn find(p: &mut HashMap<char, char>, c: char) -> char {
        let mut r = c;
        while let Some(&n) = p.get(&r) {
            if n == r {
                break;
            }
            r = n;
        }
        p.insert(c, r);
        r
    }
    let mut parent: HashMap<char, char> = HashMap::new();
    for rd in readers {
        for line in rd.split(b'\n').map_while(Result::ok) {
            let line = String::from_utf8_lossy(&line);
            if !line.starts_with("U+") {
                continue;
            }
            let mut p = line.trim_end_matches(['\r', '\n']).splitn(3, '\t');
            let (Some(cp), Some(field), Some(val)) = (p.next(), p.next(), p.next()) else { continue };
            if !VARIANT_FIELDS.contains(&field) {
                continue;
            }
            let cp_char = |s: &str| s.strip_prefix("U+").and_then(|h| u32::from_str_radix(h.split('<').next().unwrap_or(""), 16).ok()).and_then(char::from_u32);
            let Some(a) = cp_char(cp) else { continue };
            for tok in val.split_whitespace() {
                let Some(b) = cp_char(tok) else { continue };
                let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
                if ra != rb {
                    let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
                    parent.insert(hi, lo);
                    parent.insert(lo, lo);
                }
            }
        }
    }
    let keys: Vec<char> = parent.keys().copied().collect();
    let mut out = HashMap::new();
    for c in keys {
        let r = find(&mut parent, c);
        if r != c {
            out.insert(c, r);
        }
    }
    out
}

/// `variant_classes` of a Unihan.zip (or directory).
pub fn parse_variants(path: &Path) -> Result<HashMap<char, char>> {
    let mut readers: Vec<BufReader<Box<dyn Read>>> = Vec::new();
    if path.is_dir() {
        for name in VARIANT_FILES {
            let f = path.join(name);
            if f.exists() {
                readers.push(BufReader::new(Box::new(std::fs::File::open(f)?)));
            }
        }
    } else {
        let mut zf = zip::ZipArchive::new(std::fs::File::open(path)?)?;
        for i in 0..zf.len() {
            let mut f = zf.by_index(i)?;
            let base = f.name().rsplit('/').next().unwrap_or("").to_string();
            if VARIANT_FILES.contains(&base.as_str()) {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                readers.push(BufReader::new(Box::new(std::io::Cursor::new(buf))));
            }
        }
    }
    Ok(variant_classes(readers))
}

#[cfg(test)]
mod variant_tests {
    use super::*;

    #[test]
    fn variant_classes_join_korean_and_chinese_forms() {
        let txt = "U+654E\tkZVariant\tU+6559<kHanYu\nU+771E\tkSemanticVariant\tU+771F<kMatthews\nU+F900\tkCompatibilityVariant\tU+8C48\nU+4E00\tkSimplifiedVariant\tU+4E00\n";
        let m = variant_classes(vec![std::io::Cursor::new(txt.as_bytes())]);
        assert_eq!(m.get(&'敎').copied().unwrap_or('敎'), m.get(&'教').copied().unwrap_or('教'));
        assert_eq!(m.get(&'眞').copied().unwrap_or('眞'), m.get(&'真').copied().unwrap_or('真'));
        assert_eq!(m.get(&'\u{F900}'), Some(&'\u{8C48}'));
        assert!(!m.contains_key(&'一'));
    }
}
