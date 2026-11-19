//! AI English for krdict example sentences (`data/example_en.tsv`): one reviewed translation per
//! 초급/중급 word, keyed by the krdict entry id, sense and example index. Merged into the entry
//! at build time (`en`, plus `en_ai: true` so the app labels it); a row whose Korean no longer
//! matches the source example is ignored.
//!
//! `data/example_en_idioms.tsv` is the staging file of the same overlay for the examples of krdict 속담 and 성어
//! (same columns plus a sixth, `review`): `draft` rows are not loaded, only rows whose review column starts
//! with `approved` ship. A dialogue's turns are one line in the file, joined by a literal `\n` (backslash, n)
//! that is read back as the newline the source example has (in the Korean and in the English).

use crate::common::Entry;
use serde_json::Value;
use std::collections::HashMap;

/// A turn break written `\n` (two characters) in the files is the newline of the source example.
pub fn unescape(s: &str) -> String {
    s.replace("\\n", "\n")
}

pub struct ExampleEn(HashMap<(String, usize, usize), (String, String)>);

impl ExampleEn {
    /// Rows of five columns (reviewed), or six with a review column that must start with `approved`.
    pub fn parse(tsv: &str) -> ExampleEn {
        let mut m = HashMap::new();
        for line in tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            let (id, sense, ex, ko, en) = match f[..] {
                [id, sense, ex, ko, en] => (id, sense, ex, ko, en),
                [id, sense, ex, ko, en, review] if review.starts_with("approved") => (id, sense, ex, ko, en),
                _ => continue,
            };
            if let (Ok(s), Ok(x)) = (sense.parse(), ex.parse()) {
                m.insert((id.to_string(), s, x), (unescape(ko), unescape(en)));
            }
        }
        ExampleEn(m)
    }

    /// The reviewed overlay plus the approved rows of the idiom staging file.
    pub fn builtin() -> ExampleEn {
        let mut t = Self::parse(include_str!("../data/example_en.tsv"));
        t.0.extend(Self::parse(include_str!("../data/example_en_idioms.tsv")).0);
        t
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Fill in the English of matching examples of a krdict entry; returns how many were set.
    pub fn apply(&self, e: &mut Entry) -> usize {
        if self.0.is_empty() || e.source != "krdict" {
            return 0;
        }
        let mut n = 0;
        let Some(senses) = e.data.get_mut("senses").and_then(Value::as_array_mut) else { return 0 };
        for (si, s) in senses.iter_mut().enumerate() {
            let Some(exs) = s.get_mut("examples").and_then(Value::as_array_mut) else { continue };
            for (xi, x) in exs.iter_mut().enumerate() {
                let Some((ko, en)) = self.0.get(&(e.ext_id.clone(), si, xi)) else { continue };
                if x.get("ko").and_then(Value::as_str) == Some(ko.as_str()) && x.get("en").is_none() {
                    x["en"] = Value::String(en.clone());
                    x["en_ai"] = Value::Bool(true);
                    n += 1;
                }
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn staging_rows_ship_only_when_approved_and_dialogues_keep_their_turns() {
        let t = ExampleEn::parse("1\t0\t0\t가: 응.\\n나: 아.\tA: Yes.\\nB: Oh.\tdraft\n1\t0\t1\t좋아요.\tGood.\tapproved:x\n1\t0\t2\t아니요.\tNo.\n");
        assert_eq!(t.len(), 2, "the draft row is not loaded");
        let mut e = Entry { source: "krdict", ext_id: "1".into(), data: json!({"senses": [{"examples": [{"ko": "가: 응.\n나: 아."}, {"ko": "좋아요."}, {"ko": "아니요."}]}]}), ..Entry::default() };
        assert_eq!(t.apply(&mut e), 2);
        assert!(e.data["senses"][0]["examples"][0].get("en").is_none());
        let d = ExampleEn::parse("1\t0\t0\t가: 응.\\n나: 아.\tA: Yes.\\nB: Oh.\tapproved\n");
        let mut e = Entry { source: "krdict", ext_id: "1".into(), data: json!({"senses": [{"examples": [{"ko": "가: 응.\n나: 아."}]}]}), ..Entry::default() };
        assert_eq!(d.apply(&mut e), 1);
        assert_eq!(e.data["senses"][0]["examples"][0]["en"], "A: Yes.\nB: Oh.");
    }

    #[test]
    fn merges_only_matching_examples() {
        let t = ExampleEn::parse("# header\n7\t0\t1\t학교에 가요.\tI go to school.\n7\t0\t0\t바뀐 문장.\tStale.\n");
        assert_eq!(t.len(), 2);
        let mut e = Entry { source: "krdict", ext_id: "7".into(), data: json!({"senses": [{"examples": [{"ko": "다른 문장."}, {"ko": "학교에 가요."}]}]}), ..Entry::default() };
        assert_eq!(t.apply(&mut e), 1);
        assert_eq!(e.data["senses"][0]["examples"][1]["en"], "I go to school.");
        assert_eq!(e.data["senses"][0]["examples"][1]["en_ai"], true);
        assert!(e.data["senses"][0]["examples"][0].get("en").is_none()); // Korean changed: ignored
        let mut other = Entry { source: "kengdic", ext_id: "7".into(), data: e.data.clone(), ..Entry::default() };
        assert_eq!(t.apply(&mut other), 0);
    }
}
