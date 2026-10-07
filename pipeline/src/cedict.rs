//! CC-CEDICT (Chinese-English, CC BY-SA 4.0) parser: `cedict_1_0_ts_utf-8_mdbg.zip` (member
//! `cedict_ts.u8`) or the plain `.u8` text.
//!
//! Line format: `Traditional Simplified [pin1 yin1] /gloss/gloss/` ('#' lines are comments).

use crate::common::*;
use anyhow::{anyhow, Result};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Most glosses kept as senses of one entry.
const MAX_SENSES: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub trad: String,
    pub simp: String,
    /// Numbered pinyin as written in the file ("xue2 xiao4", "nu:3").
    pub pinyin: String,
    pub glosses: Vec<String>,
}

pub fn parse_line(line: &str) -> Option<Line> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (trad, rest) = line.split_once(' ')?;
    let (simp, rest) = rest.trim_start().split_once(' ')?;
    let rest = rest.trim_start().strip_prefix('[')?;
    let (pinyin, rest) = rest.split_once(']')?;
    let rest = rest.trim().strip_prefix('/')?;
    let glosses: Vec<String> = rest.trim_end_matches('/').split('/').map(|g| g.trim().to_string()).filter(|g| !g.is_empty()).collect();
    if glosses.is_empty() {
        return None;
    }
    Some(Line { trad: trad.to_string(), simp: simp.to_string(), pinyin: pinyin.trim().to_string(), glosses })
}

// --- pinyin ----------------------------------------------------------------------

const TONED: [(char, [char; 4]); 12] = [
    ('a', ['ā', 'á', 'ǎ', 'à']),
    ('e', ['ē', 'é', 'ě', 'è']),
    ('i', ['ī', 'í', 'ǐ', 'ì']),
    ('o', ['ō', 'ó', 'ǒ', 'ò']),
    ('u', ['ū', 'ú', 'ǔ', 'ù']),
    ('ü', ['ǖ', 'ǘ', 'ǚ', 'ǜ']),
    ('A', ['Ā', 'Á', 'Ǎ', 'À']),
    ('E', ['Ē', 'É', 'Ě', 'È']),
    ('I', ['Ī', 'Í', 'Ǐ', 'Ì']),
    ('O', ['Ō', 'Ó', 'Ǒ', 'Ò']),
    ('U', ['Ū', 'Ú', 'Ǔ', 'Ù']),
    ('Ü', ['Ǖ', 'Ǘ', 'Ǚ', 'Ǜ']),
];

fn mark(c: char, tone: usize) -> char {
    TONED.iter().find(|(b, _)| *b == c).map(|(_, t)| t[tone - 1]).unwrap_or(c)
}

/// One numbered syllable ("xue2", "lu:e4", "Zhong1", "r5") -> tone-marked ("xué", "lüè", "Zhōng", "r").
/// Tokens without a final tone digit 1-5 are returned unchanged. Mark placement: `a` or `e` if
/// present, else the `o` of `ou`, else the last vowel (so `iu`/`ui` mark the second letter).
pub fn mark_syllable(tok: &str) -> String {
    let Some(last) = tok.chars().last() else { return String::new() };
    let Some(tone) = last.to_digit(10).filter(|t| (1..=5).contains(t)) else { return tok.replace("u:", "ü").replace("U:", "Ü") };
    let base: Vec<char> = {
        let t = tok[..tok.len() - 1].replace("u:", "ü").replace("U:", "Ü");
        t.chars().collect()
    };
    if tone == 5 {
        return base.into_iter().collect();
    }
    let is_v = |c: char| "aeiouüAEIOUÜ".contains(c);
    let lower = |c: char| c.to_ascii_lowercase();
    let pos = base
        .iter()
        .position(|&c| lower(c) == 'a')
        .or_else(|| base.iter().position(|&c| lower(c) == 'e'))
        .or_else(|| base.windows(2).position(|w| lower(w[0]) == 'o' && lower(w[1]) == 'u'))
        .or_else(|| base.iter().rposition(|&c| is_v(c)));
    let mut out = base;
    if let Some(i) = pos {
        out[i] = mark(out[i], tone as usize);
    }
    out.into_iter().collect()
}

/// "xue2 xiao4" -> "xué xiào".
pub fn pinyin_marked(numbered: &str) -> String {
    numbered.split_whitespace().map(mark_syllable).collect::<Vec<_>>().join(" ")
}

// --- Sino-Korean reading ------------------------------------------------------------

/// Apply the 두음법칙 (initial-sound rule) to the first syllable of a Sino-Korean word and the
/// 렬/률 -> 열/율 rule after a vowel or ㄴ batchim (`規律` 규율, `比率` 비율).
pub fn dueum(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let dec = |c: char| (c as u32).checked_sub(0xAC00).filter(|n| *n < 11172).map(|n| (n / 588, (n % 588) / 28, n % 28));
    let comp = |i: u32, m: u32, f: u32| char::from_u32(0xAC00 + (i * 21 + m) * 28 + f).unwrap();
    // initial indices: ㄴ 2, ㄹ 5, ㅇ 11. vowel indices: ㅏ0 ㅐ1 ㅑ2 ㅒ3 ㅓ4 ㅔ5 ㅕ6 ㅖ7 ㅗ8 ㅘ9 ㅙ10 ㅚ11 ㅛ12 ㅜ13 ㅝ14 ㅞ15 ㅟ16 ㅠ17 ㅡ18 ㅢ19 ㅣ20
    if let Some((i, m, f)) = chars.first().copied().and_then(dec) {
        let y_like = [2, 3, 6, 7, 12, 17, 20].contains(&m); // ㅑ ㅒ ㅕ ㅖ ㅛ ㅠ ㅣ
        let ni = if i == 5 {
            if y_like {
                Some(11)
            } else if [0, 1, 8, 11, 13, 18].contains(&m) {
                Some(2) // ㅏ ㅐ ㅗ ㅚ ㅜ ㅡ
            } else {
                None
            }
        } else if i == 2 && y_like {
            Some(11)
        } else {
            None
        };
        if let Some(ni) = ni {
            chars[0] = comp(ni, m, f);
        }
    }
    for k in 1..chars.len() {
        let (Some((_, _, pf)), Some((i, m, f))) = (dec(chars[k - 1]), dec(chars[k])) else { continue };
        // 렬 / 률 (ㅕ+ㄹ, ㅠ+ㄹ) after a vowel or ㄴ batchim -> 열 / 율
        if i == 5 && (m == 6 || m == 17) && f == 8 && (pf == 0 || pf == 4) {
            chars[k] = comp(11, m, f);
        }
    }
    chars.into_iter().collect()
}

/// Sino-Korean reading of an all-hanja headword from per-character readings (first reading of
/// each character); `None` when any character is not a hanja or has no known reading.
pub fn sino_korean(hanja: &str, readings: &HashMap<char, String>) -> Option<String> {
    let mut out = String::new();
    let mut n = 0;
    for c in hanja.chars() {
        if c == '·' || c == '・' {
            continue;
        }
        if !is_cjk(c) {
            return None;
        }
        out.push_str(readings.get(&c)?);
        n += 1;
    }
    if n == 0 {
        return None;
    }
    Some(dueum(&out))
}

// --- entries ---------------------------------------------------------------------------

pub fn to_entry(l: &Line, sino: &HashMap<char, String>) -> Entry {
    let mut senses: Vec<Value> = Vec::new();
    let mut cl: Vec<String> = Vec::new();
    for g in &l.glosses {
        if let Some(c) = g.strip_prefix("CL:") {
            cl.extend(c.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()));
        } else if senses.len() < MAX_SENSES {
            senses.push(json!({ "gloss": g }));
        }
    }
    if senses.is_empty() {
        senses.push(json!({ "gloss": l.glosses.join("; ") }));
    }
    let mut data = Map::new();
    data.insert("senses".into(), senses.into());
    if l.simp != l.trad {
        data.insert("simplified".into(), l.simp.clone().into());
    }
    data.insert("pinyin".into(), pinyin_marked(&l.pinyin).into());
    data.insert("pinyin_num".into(), l.pinyin.clone().into());
    if !cl.is_empty() {
        data.insert("cl".into(), cl.into());
    }
    let mut forms = Vec::new();
    if l.simp != l.trad {
        forms.push(l.simp.clone());
    }
    Entry {
        source: "cedict",
        lang: "en",
        ext_id: format!("{} {} [{}]", l.trad, l.simp, l.pinyin),
        headword: l.trad.clone(),
        hanja: has_cjk(&l.trad).then(|| l.trad.clone()),
        pos: "other".into(),
        pron: sino_korean(&l.trad, sino),
        kind: "word".into(),
        forms,
        data: Value::Object(data),
        ..Default::default()
    }
}

/// Parse CEDICT text from a reader, calling `cb` per entry in file order.
pub fn parse_reader<R: BufRead>(rd: R, sino: &HashMap<char, String>, mut cb: impl FnMut(Entry) -> Result<()>) -> Result<()> {
    for line in rd.split(b'\n') {
        let line = line?;
        let line = String::from_utf8_lossy(&line);
        if let Some(l) = parse_line(&line) {
            cb(to_entry(&l, sino))?;
        }
    }
    Ok(())
}

/// Parse `cedict_1_0_ts_utf-8_mdbg.zip` (any member ending `.u8` / `.txt`) or a plain text file.
pub fn parse_file(path: &Path, sino: &HashMap<char, String>, cb: impl FnMut(Entry) -> Result<()>) -> Result<()> {
    let is_zip = path.extension().is_some_and(|e| e == "zip");
    if !is_zip {
        return parse_reader(BufReader::new(std::fs::File::open(path)?), sino, cb);
    }
    let mut zf = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    for i in 0..zf.len() {
        let f = zf.by_index(i)?;
        let name = f.name().to_string();
        if name.ends_with(".u8") || name.ends_with(".txt") {
            let rd: Box<dyn Read> = Box::new(f);
            return parse_reader(BufReader::new(rd), sino, cb);
        }
    }
    Err(anyhow!("no .u8 member in {}", path.display()))
}

/// Sort key: shorter headwords first, then file order.
pub fn rank(trad: &str, seq: usize) -> i64 {
    1_000_000 + (trad.chars().count().min(8) as i64) * 100_000 + seq.min(99_999) as i64
}
