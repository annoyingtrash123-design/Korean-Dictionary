//! Owner rule "if in doubt, no hanja" (CLAUDE.md), applied to the hanja the pipeline adds itself.
//!
//! A pipeline-added hanja that no Korean dictionary ties to the word's meaning (`low` certainty in
//! [`crate::hanja_fill`]) and every kengdic-fallback hanja was put to an independent review (`data/hanja_reviewed.tsv`):
//! the reviewer had the entry's meaning and every 표준국어대사전 entry with that spelling, and kept a hanja only when
//! 표준국어대사전 gives exactly that hanja on the matching meaning, fixed it when 표준국어대사전 gives a different
//! hanja for that meaning, and dropped everything else.
//!
//! In the build: a `low` fill is written only with a `keep` or `fix` verdict (an unreviewed one is dropped, so new
//! doubtful fills never ship unreviewed); a fallback hanja with a `drop` verdict is dropped and with `fix` replaced.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Keep,
    Fix(String),
    Drop,
}

#[derive(Default)]
pub struct HanjaReview {
    /// (headword without spaces, proposed hanja) -> verdict
    by_key: HashMap<(String, String), Verdict>,
}

fn key(headword: &str) -> String {
    headword.chars().filter(|c| !c.is_whitespace()).collect()
}

/// NFC: a compatibility ideograph (率 U+F9DB, as 표준국어대사전 data often has it) matches its unified form (率 U+7387).
fn nfc(h: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    h.nfc().collect()
}

impl HanjaReview {
    /// `headword  hanja  verdict  fixed_hanja  reason` (the header line and `#` lines are skipped).
    pub fn parse(tsv: &str) -> HanjaReview {
        let mut by_key = HashMap::new();
        for l in tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() < 4 || f[0] == "headword" {
                continue;
            }
            let v = match (f[2], f[3].trim()) {
                ("keep", _) => Verdict::Keep,
                ("fix", h) if !h.is_empty() => Verdict::Fix(h.to_string()),
                _ => Verdict::Drop,
            };
            by_key.insert((key(f[0]), nfc(f[1])), v);
        }
        HanjaReview { by_key }
    }

    pub fn builtin() -> HanjaReview {
        Self::parse(include_str!("../data/hanja_reviewed.tsv"))
    }

    pub fn get(&self, headword: &str, hanja: &str) -> Option<&Verdict> {
        self.by_key.get(&(key(headword), nfc(hanja)))
    }

    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verdicts() {
        let r = HanjaReview::parse("headword\thanja\tverdict\tfixed_hanja\treason\n극피동물\t棘皮動物\tkeep\t\tok\n환생\t幻生\tfix\t還生\tmeaning\n하악\t下顎\tdrop\t\tonomatopoeia\nx\ty\tfix\t\tno hanja given\n");
        assert_eq!(r.get("극피동물", "棘皮動物"), Some(&Verdict::Keep));
        assert_eq!(r.get("환생", "幻生"), Some(&Verdict::Fix("還生".into())));
        assert_eq!(r.get("하악", "下顎"), Some(&Verdict::Drop));
        assert_eq!(r.get("x", "y"), Some(&Verdict::Drop));
        assert_eq!(r.get("환생", "還生"), None);
        // compatibility ideographs match their unified form
        let c = HanjaReview::parse("범죄율\t犯罪\u{F9DB}\tkeep\t\tok\n");
        assert_eq!(c.get("범죄율", "犯罪\u{7387}"), Some(&Verdict::Keep));
    }

    #[test]
    fn builtin_parses() {
        let r = HanjaReview::builtin();
        // every verdict row of the shipped file is a real row (no silent format drift)
        let rows = include_str!("../data/hanja_reviewed.tsv").lines().filter(|l| !l.starts_with('#') && !l.starts_with("headword") && !l.trim().is_empty()).count();
        assert_eq!(r.len(), rows);
    }
}
