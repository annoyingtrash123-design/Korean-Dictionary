//! Shared helpers: text normalisation, POS maps, entry model.

use serde_json::Value;

/// One dictionary entry, source-agnostic, ready to be inserted into SQLite.
#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub source: &'static str,
    pub lang: &'static str,
    pub ext_id: String,
    pub headword: String,
    pub homonym: Option<i64>,
    pub hanja: Option<String>,
    pub pos: String,
    pub ko_pos: String,
    pub pron: Option<String>,
    pub level: Option<i64>,
    pub kind: String,
    pub forms: Vec<String>,
    pub data: Value,
    /// Korean definitions of all senses (used for grammar categorisation).
    pub ko_defs: Vec<String>,
}

// --- CJK / hangul --------------------------------------------------------

pub fn is_cjk(c: char) -> bool {
    let o = c as u32;
    (0x3400..=0x4DBF).contains(&o)
        || (0x4E00..=0x9FFF).contains(&o)
        || (0xF900..=0xFAFF).contains(&o)
        || (0x20000..=0x2FA1F).contains(&o)
}

pub fn has_cjk(s: &str) -> bool {
    s.chars().any(is_cjk)
}

/// Distinct CJK ideographs in order of first appearance.
pub fn cjk_chars(s: &str) -> Vec<char> {
    let mut out = Vec::new();
    for c in s.chars() {
        if is_cjk(c) && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

pub fn is_hangul(c: char) -> bool {
    let o = c as u32;
    (0xAC00..=0xD7A3).contains(&o) || (0x1100..=0x11FF).contains(&o) || (0x3130..=0x318F).contains(&o)
}

pub fn has_hangul(s: &str) -> bool {
    s.chars().any(is_hangul)
}

// --- text ----------------------------------------------------------------

/// Minimal HTML entity decoder (krdict double-escapes annotations).
pub fn html_unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(j) = rest.find(';').filter(|&j| j <= 10) {
            let name = &rest[1..j];
            let rep: Option<String> = match name {
                "amp" => Some("&".into()),
                "lt" => Some("<".into()),
                "gt" => Some(">".into()),
                "quot" => Some("\"".into()),
                "apos" => Some("'".into()),
                "nbsp" => Some("\u{a0}".into()),
                "middot" => Some("·".into()),
                "hellip" => Some("…".into()),
                n if n.starts_with("#x") || n.starts_with("#X") => {
                    u32::from_str_radix(&n[2..], 16).ok().and_then(char::from_u32).map(|c| c.to_string())
                }
                n if n.starts_with('#') => n[1..].parse::<u32>().ok().and_then(char::from_u32).map(|c| c.to_string()),
                _ => None,
            };
            if let Some(r) = rep {
                out.push_str(&r);
                rest = &rest[j + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

/// Unescape stray entities, collapse whitespace, trim.
pub fn clean(s: &str) -> String {
    let s = if s.contains('&') { html_unescape(s) } else { s.to_string() };
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Lookup key: headword with '-', '^', ' ', '·' removed.
pub fn hw_norm(h: &str) -> String {
    h.chars().filter(|c| !matches!(c, '-' | '^' | ' ' | '\u{a0}' | '·' | '・' | 'ㆍ' | '‧')).collect()
}

pub fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let cut: String = s.chars().take(n - 1).collect();
    format!("{}…", cut.trim_end())
}

/// Join glosses with "; " while they fit within `limit` characters.
pub fn make_gloss<S: AsRef<str>>(glosses: &[S], limit: usize) -> String {
    let mut out = String::new();
    for g in glosses {
        let g = g.as_ref().trim();
        if g.is_empty() {
            continue;
        }
        let cand = if out.is_empty() { g.to_string() } else { format!("{out}; {g}") };
        if cand.chars().count() > limit {
            if out.is_empty() {
                return truncate(g, limit);
            }
            break;
        }
        out = cand;
    }
    out
}

// --- POS / kind maps -------------------------------------------------------

pub fn kr_pos(ko: &str) -> Option<&'static str> {
    Some(match ko {
        "명사" => "noun",
        "동사" => "verb",
        "형용사" => "adjective",
        "부사" => "adverb",
        "조사" => "particle",
        "어미" => "ending",
        "접사" => "affix",
        "의존 명사" | "의존명사" => "bound noun",
        "보조 동사" | "보조동사" => "auxiliary verb",
        "보조 형용사" | "보조형용사" => "auxiliary adjective",
        "대명사" => "pronoun",
        "수사" => "numeral",
        "관형사" => "determiner",
        "감탄사" => "interjection",
        "구" => "phrase",
        "품사 없음" | "품사없음" => "other",
        _ => return None,
    })
}

pub fn kr_pos_or_other(ko: &str) -> &'static str {
    kr_pos(ko).unwrap_or("other")
}

/// lexicalUnit that overrides POS for non-word units.
pub fn kr_unit_pos(unit: &str) -> Option<&'static str> {
    Some(match unit {
        "구" => "phrase",
        "관용구" => "idiom",
        "속담" => "proverb",
        "문법‧표현" | "문법·표현" => "expression",
        _ => return None,
    })
}

pub fn kr_unit_kind(unit: &str) -> &'static str {
    match unit {
        "구" => "phrase",
        "관용구" => "idiom",
        "속담" => "proverb",
        "문법‧표현" | "문법·표현" => "grammar",
        _ => "word",
    }
}

pub fn kr_level(s: &str) -> Option<i64> {
    match s {
        "초급" => Some(1),
        "중급" => Some(2),
        "고급" => Some(3),
        _ => None,
    }
}

pub fn kr_rel(ty: &str) -> String {
    match ty {
        "반대말" => "antonym",
        "유의어" | "비슷한말" | "동의어" => "synonym",
        "높임말" => "honorific",
        "낮춤말" => "humble",
        "참고어" | "참고 어휘" => "see also",
        "큰말" => "larger form",
        "작은말" => "smaller form",
        "센말" => "stronger form",
        "여린말" => "softer form",
        "준말" => "abbreviation",
        "본말" => "full form",
        "파생어" => "derived",
        "부표제어" => "sub-entry",
        "☞(가 보라)" => "reference",
        other => other,
    }
    .to_string()
}

pub fn wikt_pos(p: &str) -> &'static str {
    match p {
        "noun" | "name" => "noun",
        "verb" => "verb",
        "adj" => "adjective",
        "adv" => "adverb",
        "pron" => "pronoun",
        "num" => "numeral",
        "det" => "determiner",
        "intj" => "interjection",
        "particle" | "postp" | "prep" => "particle",
        "suffix" | "prefix" | "infix" | "affix" | "circumfix" => "affix",
        "proverb" => "proverb",
        "phrase" | "prep_phrase" => "phrase",
        "idiom" => "idiom",
        "classifier" => "bound noun",
        _ => "other",
    }
}

pub fn kind_for_pos(pos: &str) -> &'static str {
    match pos {
        "phrase" => "phrase",
        "idiom" => "idiom",
        "proverb" => "proverb",
        _ => "word",
    }
}

pub fn is_verbal(pos: &str) -> bool {
    matches!(pos, "verb" | "adjective" | "auxiliary verb" | "auxiliary adjective")
}
