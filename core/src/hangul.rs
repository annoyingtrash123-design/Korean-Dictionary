//! Hangul syllable utilities: compose / decompose precomposed syllables
//! (U+AC00..U+D7A3) and helpers for batchim (final consonant) handling.

const BASE: u32 = 0xAC00;
const LAST: u32 = 0xD7A3;

/// Initial consonants (choseong), index 0..18.
pub const CHOSEONG: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ',
    'ㅍ', 'ㅎ',
];
/// Vowels (jungseong), index 0..20.
pub const JUNGSEONG: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ',
    'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];
/// Final consonants (jongseong), index 0..27; index 0 = no batchim.
pub const JONGSEONG: [char; 28] = [
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ',
    'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// Compound batchim -> its two parts, e.g. 'ㄺ' -> ('ㄹ', 'ㄱ').
pub fn split_compound_final(j: char) -> Option<(char, char)> {
    Some(match j {
        'ㄳ' => ('ㄱ', 'ㅅ'),
        'ㄵ' => ('ㄴ', 'ㅈ'),
        'ㄶ' => ('ㄴ', 'ㅎ'),
        'ㄺ' => ('ㄹ', 'ㄱ'),
        'ㄻ' => ('ㄹ', 'ㅁ'),
        'ㄼ' => ('ㄹ', 'ㅂ'),
        'ㄽ' => ('ㄹ', 'ㅅ'),
        'ㄾ' => ('ㄹ', 'ㅌ'),
        'ㄿ' => ('ㄹ', 'ㅍ'),
        'ㅀ' => ('ㄹ', 'ㅎ'),
        'ㅄ' => ('ㅂ', 'ㅅ'),
        _ => return None,
    })
}

/// Jamo indices of a syllable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Jamo {
    pub initial: u8,
    pub medial: u8,
    pub fin: u8,
}

/// True for a precomposed Hangul syllable (가..힣).
pub fn is_syllable(c: char) -> bool {
    (BASE..=LAST).contains(&(c as u32))
}

/// True for a compatibility jamo letter (ㄱ..ㅣ).
pub fn is_jamo(c: char) -> bool {
    (0x3131..=0x3163).contains(&(c as u32))
}

/// Syllable or jamo.
pub fn is_hangul_char(c: char) -> bool {
    is_syllable(c) || is_jamo(c)
}

/// True if the string is non-empty and made only of Hangul syllables/jamo.
pub fn is_hangul(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_hangul_char)
}

/// True if the string contains at least one Hangul syllable or jamo.
pub fn has_hangul(s: &str) -> bool {
    s.chars().any(is_hangul_char)
}

/// CJK ideograph (hanja / kanji / hanzi), including extension A and compatibility ideographs.
pub fn is_han(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0x20000..=0x2FA1F).contains(&u)
}

pub fn has_han(s: &str) -> bool {
    s.chars().any(is_han)
}

/// Split a syllable into jamo indices.
pub fn decompose(c: char) -> Option<Jamo> {
    if !is_syllable(c) {
        return None;
    }
    let n = c as u32 - BASE;
    Some(Jamo { initial: (n / 588) as u8, medial: ((n % 588) / 28) as u8, fin: (n % 28) as u8 })
}

/// Build a syllable from jamo indices.
pub fn compose(initial: u8, medial: u8, fin: u8) -> Option<char> {
    if initial > 18 || medial > 20 || fin > 27 {
        return None;
    }
    char::from_u32(BASE + (initial as u32 * 21 + medial as u32) * 28 + fin as u32)
}

pub fn initial_index(j: char) -> Option<u8> {
    CHOSEONG.iter().position(|&x| x == j).map(|i| i as u8)
}
pub fn medial_index(j: char) -> Option<u8> {
    JUNGSEONG.iter().position(|&x| x == j).map(|i| i as u8)
}
/// Index of a final consonant letter (never 0; use `without_final` to clear).
pub fn final_index(j: char) -> Option<u8> {
    if j == '\0' {
        return None;
    }
    JONGSEONG.iter().position(|&x| x == j).map(|i| i as u8)
}

/// Does the syllable have a batchim?
pub fn has_batchim(c: char) -> bool {
    decompose(c).map_or(false, |d| d.fin != 0)
}

/// Final consonant letter of a syllable (None when there is none / not a syllable).
pub fn final_of(c: char) -> Option<char> {
    decompose(c).and_then(|d| if d.fin == 0 { None } else { Some(JONGSEONG[d.fin as usize]) })
}

/// Initial consonant letter.
pub fn initial_of(c: char) -> Option<char> {
    decompose(c).map(|d| CHOSEONG[d.initial as usize])
}

/// Vowel letter.
pub fn medial_of(c: char) -> Option<char> {
    decompose(c).map(|d| JUNGSEONG[d.medial as usize])
}

/// Replace/add the final consonant.
pub fn with_final(c: char, j: char) -> Option<char> {
    let d = decompose(c)?;
    compose(d.initial, d.medial, final_index(j)?)
}

/// Remove the final consonant.
pub fn without_final(c: char) -> Option<char> {
    let d = decompose(c)?;
    compose(d.initial, d.medial, 0)
}

/// Replace the vowel.
pub fn with_medial(c: char, j: char) -> Option<char> {
    let d = decompose(c)?;
    compose(d.initial, medial_index(j)?, d.fin)
}

/// Replace the initial consonant.
pub fn with_initial(c: char, j: char) -> Option<char> {
    let d = decompose(c)?;
    compose(initial_index(j)?, d.medial, d.fin)
}

/// Build a syllable from jamo letters (final may be '\0').
pub fn syllable(initial: char, medial: char, fin: char) -> Option<char> {
    let f = if fin == '\0' { 0 } else { final_index(fin)? };
    compose(initial_index(initial)?, medial_index(medial)?, f)
}

/// Decompose a whole string into a flat jamo string ('한' -> 'ㅎㅏㄴ').
pub fn to_jamo(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match decompose(c) {
            Some(d) => {
                out.push(CHOSEONG[d.initial as usize]);
                out.push(JUNGSEONG[d.medial as usize]);
                if d.fin != 0 {
                    out.push(JONGSEONG[d.fin as usize]);
                }
            }
            None => out.push(c),
        }
    }
    out
}

/// Lookup key normalisation used by the packs: drop '-', '^', ' ', '·'.
pub fn norm_headword(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '-' | '^' | ' ' | '·' | '\u{3000}' | '\t')).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_compose_roundtrip() {
        for c in ['가', '힣', '한', '값', '닭', '쌍', '뷁'] {
            let d = decompose(c).unwrap();
            assert_eq!(compose(d.initial, d.medial, d.fin), Some(c));
        }
        assert_eq!(decompose('a'), None);
        assert_eq!(decompose('ㄱ'), None);
    }

    #[test]
    fn jamo_letters() {
        let d = decompose('한').unwrap();
        assert_eq!(CHOSEONG[d.initial as usize], 'ㅎ');
        assert_eq!(JUNGSEONG[d.medial as usize], 'ㅏ');
        assert_eq!(JONGSEONG[d.fin as usize], 'ㄴ');
        assert_eq!(to_jamo("한글"), "ㅎㅏㄴㄱㅡㄹ");
        assert_eq!(syllable('ㅎ', 'ㅏ', 'ㄴ'), Some('한'));
        assert_eq!(syllable('ㅎ', 'ㅏ', '\0'), Some('하'));
    }

    #[test]
    fn batchim_helpers() {
        assert!(has_batchim('먹'));
        assert!(!has_batchim('가'));
        assert_eq!(final_of('갔'), Some('ㅆ'));
        assert_eq!(final_of('가'), None);
        assert_eq!(with_final('가', 'ㅂ'), Some('갑'));
        assert_eq!(with_final('가', 'ㄹ'), Some('갈'));
        assert_eq!(without_final('갔'), Some('가'));
        assert_eq!(with_medial('가', 'ㅓ'), Some('거'));
        assert_eq!(with_initial('가', 'ㅅ'), Some('사'));
        assert_eq!(split_compound_final('ㄺ'), Some(('ㄹ', 'ㄱ')));
        assert_eq!(split_compound_final('ㄱ'), None);
    }

    #[test]
    fn script_detection() {
        assert!(is_hangul("학교"));
        assert!(is_hangul("ㄱㄴ"));
        assert!(!is_hangul("school"));
        assert!(!is_hangul(""));
        assert!(has_hangul("abc가"));
        assert!(is_han('學'));
        assert!(!is_han('학'));
        assert!(has_han("a學"));
    }

    #[test]
    fn normalisation() {
        assert_eq!(norm_headword("-아서/어서"), "아서/어서");
        assert_eq!(norm_headword("가^다"), "가다");
        assert_eq!(norm_headword("먹 다"), "먹다");
    }
}
