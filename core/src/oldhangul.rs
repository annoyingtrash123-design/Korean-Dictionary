//! Old-spelling normalisation for Reader lookups of Middle / early-modern Korean text.
//!
//! [`variants`] turns an eojeol written in old orthography into a short list of modern-spelling
//! candidates that the engine then looks up (the original is always tried first by the caller).
//! The table is deliberately small; every rule is a guess that only costs one extra index probe.
//!
//! Jamo level (conjoining Jamo block U+1100..U+11FF, as Wikisource and most corpora write old
//! Hangul; such runs do not compose to precomposed syllables under NFC):
//!
//! | old | candidates (in order) | example |
//! |---|---|---|
//! | `ᆞ` U+119E (arae-a), `ᆢ` U+11A2 | ㅏ, ㅡ | ᄒᆞᆫ → 한, 흔 |
//! | `ᆟ` / `ᆠ` / `ᆡ` (arae-a + ㅓ / ㅜ / ㅣ) | ㅓ / ㅜ / ㅐ, ㅔ | |
//! | initial `ᅀ` (ㅿ) | ㅅ, ㅇ | 아ᅀᆞ → 아사, 아아 |
//! | initial `ᄫ` `ᄬ` (ㅸ) | ㅂ, ㅇ | 사ᄫᅵ → 사비, 사이 |
//! | initial `ᅌ` (ㆁ) | ㅇ | |
//! | initial `ᅙ` (ㆆ) | ㅎ, ㅇ | |
//! | initial sios clusters `ᄭ ᄯ ᄲ ᄧ` and pieup clusters `ᄠ ᄡ ᄢ ᄣ ᄤ ᄥ ᄦ ᄧ` | ㄲ ㄸ ㅃ ㅉ ... | 어ᄯᅥ → 어떠 |
//! | final `ᇫ` (ㅿ) / `ᇰ` (ㆁ) / `ᇹ` (ㆆ) | ㅅ,∅ / ㅇ / ㅎ,∅ | |
//! | tone marks U+302E / U+302F (방점) | dropped | |
//!
//! Syllable level, applied on top of every jamo-level candidate, each as an extra variant:
//!
//! * palatalisation: ㄷ/ㅌ + ㅣ/ㅑ/ㅕ/ㅛ/ㅠ → ㅈ/ㅊ with the glide dropped (됴타 → 조타, 텨 → 처);
//!   ㅈ/ㅊ/ㅉ + ㅑ/ㅕ/ㅛ/ㅠ → glide dropped (져 → 저); ㅅ + ㅕ → ㅓ (셔울 → 서울);
//! * early-20th-century past tense: a ㅅ-final syllable before an ending syllable → ㅆ (갓다 → 갔다),
//!   and 하얏 → 하였 (하얏다 → 하였다).

use crate::hangul::{compose, decompose};
use unicode_normalization::UnicodeNormalization;

/// Most candidates returned.
pub const MAX_VARIANTS: usize = 48;
const MAX_JAMO_COMBOS: usize = 24;

/// Syllables that may follow a ㅅ-final past-tense stem (갓다, 하얏던, 잇고 ...).
const ENDING_START: [char; 16] = ['다', '던', '든', '고', '는', '습', '으', '스', '며', '나', '니', '지', '겠', '어', '아', '을'];

pub fn is_tone_mark(c: char) -> bool {
    matches!(c, '\u{302E}' | '\u{302F}')
}

/// Remove 방점 tone marks and compose with NFC.
pub fn clean(word: &str) -> String {
    word.chars().filter(|c| !is_tone_mark(*c)).collect::<String>().nfc().collect()
}

/// True for old-Hangul jamo that need the jamo-level table (conjoining Jamo block).
fn is_conjoining(c: char) -> bool {
    ('\u{1100}'..='\u{11FF}').contains(&c)
}

fn lead_opts(seq: &[char]) -> Option<Vec<u8>> {
    if seq.len() == 1 {
        let c = seq[0] as u32;
        if (0x1100..=0x1112).contains(&c) {
            return Some(vec![(c - 0x1100) as u8]);
        }
        return Some(match c {
            0x115F => vec![11],
            0x1140 => vec![9, 11],             // ㅿ
            0x112B | 0x112C => vec![7, 11],    // ㅸ
            0x114C => vec![11],                // ㆁ
            0x1159 => vec![18, 11],            // ㆆ
            0x112D => vec![1],                 // ᄭ sios-kiyeok
            0x112F | 0x1120 | 0x1123 => vec![4], // ᄯ ᄠ ᄣ
            0x1132 | 0x1124 => vec![8],        // ᄲ ᄤ
            0x1121 | 0x1125 => vec![10, 9],    // ᄡ ᄥ -> ㅆ, ㅅ
            0x1122 => vec![1],                 // ᄢ
            0x1127 | 0x1126 | 0x1135 => vec![13, 12], // ᄧ ᄦ ᄵ -> ㅉ, ㅈ
            _ => return None,
        });
    }
    // two-letter clusters written as separate jamo: ㅅ+ㄱ / ㅅ+ㄷ / ㅅ+ㅂ / ㅅ+ㅈ
    if seq.len() == 2 && seq[0] == '\u{1109}' {
        return Some(vec![match seq[1] {
            '\u{1100}' => 1,
            '\u{1103}' => 4,
            '\u{1107}' => 8,
            '\u{110C}' => 13,
            _ => return None,
        }]);
    }
    None
}

fn vowel_opts(seq: &[char]) -> Option<Vec<u8>> {
    if seq.len() != 1 {
        return None;
    }
    let c = seq[0] as u32;
    if (0x1161..=0x1175).contains(&c) {
        return Some(vec![(c - 0x1161) as u8]);
    }
    Some(match c {
        0x119E | 0x11A2 => vec![0, 18], // ㆍ -> ㅏ, ㅡ
        0x119F => vec![4],
        0x11A0 => vec![13],
        0x11A1 => vec![1, 5],
        _ => return None,
    })
}

/// Final consonant options as modern final indices (0 = none).
fn tail_opts(seq: &[char]) -> Option<Vec<u8>> {
    if seq.is_empty() {
        return Some(vec![0]);
    }
    if seq.len() != 1 {
        return None;
    }
    let c = seq[0] as u32;
    if (0x11A8..=0x11C2).contains(&c) {
        return Some(vec![(c - 0x11A7) as u8]);
    }
    Some(match c {
        0x11EB => vec![19, 0], // ㅿ
        0x11F0 => vec![21],    // ㆁ
        0x11F9 => vec![27, 0], // ㆆ
        0x11E6 => vec![17],    // ㅸ
        _ => return None,
    })
}

/// Expand one run of conjoining jamo into the modern syllable strings it may stand for.
fn expand_run(run: &[char]) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    let mut i = 0;
    while i < run.len() {
        let s = i;
        while i < run.len() && (('\u{1100}'..='\u{115F}').contains(&run[i])) {
            i += 1;
        }
        let lead = &run[s..i];
        let s = i;
        while i < run.len() && (('\u{1160}'..='\u{11A7}').contains(&run[i])) {
            i += 1;
        }
        let vowel: Vec<char> = run[s..i].iter().copied().filter(|c| *c != '\u{1160}').collect();
        let s = i;
        while i < run.len() && (('\u{11A8}'..='\u{11FF}').contains(&run[i])) {
            i += 1;
        }
        let tail = &run[s..i];
        let lead = if lead.is_empty() { Some(vec![11]) } else { lead_opts(lead) };
        let (Some(l), Some(v), Some(t)) = (lead, vowel_opts(&vowel), tail_opts(tail)) else { return vec![] };
        let mut syl: Vec<char> = Vec::new();
        for &a in &l {
            for &b in &v {
                for &c in &t {
                    if let Some(ch) = compose(a, b, c) {
                        syl.push(ch);
                    }
                }
            }
        }
        let mut next = Vec::new();
        for prefix in &out {
            for ch in &syl {
                next.push(format!("{prefix}{ch}"));
            }
        }
        next.truncate(MAX_JAMO_COMBOS);
        out = next;
    }
    out
}

/// Jamo-level candidates of a whole word (the NFC word itself is included when it has no old jamo).
fn jamo_level(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut out: Vec<String> = vec![String::new()];
    let mut i = 0;
    while i < chars.len() {
        let parts: Vec<String> = if is_conjoining(chars[i]) {
            let s = i;
            while i < chars.len() && is_conjoining(chars[i]) {
                i += 1;
            }
            let p = expand_run(&chars[s..i]);
            if p.is_empty() {
                return vec![];
            }
            p
        } else {
            let c = chars[i];
            i += 1;
            vec![c.to_string()]
        };
        let mut next = Vec::new();
        for pre in &out {
            for p in &parts {
                next.push(format!("{pre}{p}"));
            }
        }
        next.truncate(MAX_JAMO_COMBOS);
        out = next;
    }
    out
}

/// Palatalisation / glide dropping (see the module table).
fn palatalise(word: &str) -> String {
    word.chars()
        .map(|c| {
            let Some(d) = decompose(c) else { return c };
            let (mut i, mut m) = (d.initial, d.medial);
            let yot = |m: u8| match m {
                2 => Some(0u8),  // ㅑ -> ㅏ
                6 => Some(4),    // ㅕ -> ㅓ
                12 => Some(8),   // ㅛ -> ㅗ
                17 => Some(13),  // ㅠ -> ㅜ
                _ => None,
            };
            match i {
                3 | 16 if matches!(m, 20 | 2 | 6 | 12 | 17) => {
                    i = if i == 3 { 12 } else { 14 };
                    m = yot(m).unwrap_or(m);
                }
                12..=14 => m = yot(m).unwrap_or(m),
                9 if m == 6 => m = 4,
                _ => return c,
            }
            compose(i, m, d.fin).unwrap_or(c)
        })
        .collect()
}

/// ㅅ-final syllable before an ending -> ㅆ; 하얏 -> 하였.
fn past_tense(word: &str) -> String {
    let cs: Vec<char> = word.chars().collect();
    let mut out = cs.clone();
    for k in 0..cs.len() {
        let Some(d) = decompose(cs[k]) else { continue };
        let next_ok = cs.get(k + 1).is_some_and(|n| ENDING_START.contains(n));
        if d.fin == 19 && next_ok && matches!(d.medial, 0 | 4 | 6 | 2 | 1 | 5) {
            let after_ha = k > 0 && cs[k - 1] == '하';
            let m = if after_ha && d.initial == 11 && d.medial == 2 { 6 } else { d.medial }; // 얏 -> 였
            if let Some(c) = compose(d.initial, m, 20) {
                out[k] = c;
            }
        }
    }
    out.into_iter().collect()
}

/// Modern-spelling candidates for `word` (excluding the cleaned word itself), best guess first,
/// at most [`MAX_VARIANTS`].
pub fn variants(word: &str) -> Vec<String> {
    let w = clean(word);
    let mut out: Vec<String> = Vec::new();
    let push = |s: String, out: &mut Vec<String>| {
        if s != w && !s.is_empty() && !out.contains(&s) && out.len() < MAX_VARIANTS {
            out.push(s);
        }
    };
    let bases = jamo_level(&w);
    for b in &bases {
        push(b.clone(), &mut out);
    }
    for b in &bases {
        push(palatalise(b), &mut out);
        push(past_tense(b), &mut out);
        push(past_tense(&palatalise(b)), &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(w: &str, want: &str) -> bool {
        variants(w).iter().any(|v| v == want)
    }

    #[test]
    fn arae_a_gives_a_and_eu() {
        let v = variants("ᄒᆞᆫ"); // ᄒ + ᆞ + ᆫ
        assert_eq!(&v[..2], ["한", "흔"]);
        assert!(has("ᄒᆞ다", "하다"));
        assert!(has("ᄂᆞᆯ", "날") && has("ᄂᆞᆯ", "늘"));
    }

    #[test]
    fn old_initials_and_finals() {
        assert!(has("어ᄯᅥ", "어떠"));
        assert!(has("사ᄫᅵ", "사비") && has("사ᄫᅵ", "사이"));
        assert!(has("아ᅀᆞ", "아사"));
        assert!(has("ᄠᅳᆺ", "뜻")); // pieup-tikeut initial
        assert!(has("ᄀᆞᆺ", "갓") || has("ᄀᆞᆺ", "곳"));
        assert!(has("ᄆᆞᅀᆞᆷ", "마음")); // ᅀ -> ㅇ, ᆞ -> ㅡ combos
    }

    #[test]
    fn tone_marks_are_dropped() {
        assert_eq!(clean("나랏\u{302E}말\u{302F}"), "나랏말");
        assert!(has("ᄒᆞ\u{302E}다", "하다"));
    }

    #[test]
    fn syllable_level_rules() {
        assert!(has("하얏다", "하였다"));
        assert!(has("갓다", "갔다"));
        assert!(has("됴타", "조타"));
        assert!(has("셔울", "서울"));
        assert!(has("텨", "처"));
        assert!(has("뎌", "저"));
        assert!(has("져", "저"));
        assert!(!has("먹었다", "먹었다")); // the word itself is never a variant
        assert!(variants("학교").is_empty());
    }

    #[test]
    fn bounded_and_safe() {
        let wild: String = "ᄒᆞ".repeat(30);
        assert!(variants(&wild).len() <= MAX_VARIANTS);
        assert!(variants("\u{1100}").len() <= MAX_VARIANTS); // lone consonant: no panic
        assert!(variants("").is_empty());
    }
}
