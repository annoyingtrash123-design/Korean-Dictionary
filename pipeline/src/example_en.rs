//! AI English for krdict example sentences (`data/example_en.tsv`): one reviewed translation per
//! 초급/중급 word, keyed by the krdict entry id, sense and example index. Merged into the entry
//! at build time (`en`, plus `en_ai: true` so the app labels it); a row whose Korean no longer
//! matches the source example is ignored.

use crate::common::Entry;
use serde_json::Value;
use std::collections::HashMap;

pub struct ExampleEn(HashMap<(String, usize, usize), (String, String)>);

impl ExampleEn {
    pub fn parse(tsv: &str) -> ExampleEn {
        let mut m = HashMap::new();
        for line in tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            if let [id, sense, ex, ko, en] = f[..] {
                if let (Ok(s), Ok(x)) = (sense.parse(), ex.parse()) {
                    m.insert((id.to_string(), s, x), (ko.to_string(), en.to_string()));
                }
            }
        }
        ExampleEn(m)
    }

    pub fn builtin() -> ExampleEn {
        Self::parse(include_str!("../data/example_en.tsv"))
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
