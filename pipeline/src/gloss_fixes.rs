//! Reviewed corrections to krdict English sense glosses (`data/gloss_fixes.tsv`), keyed by the
//! krdict entry id and 0-based sense index. Applied at build time (`gloss` replaced, `gloss_ai: true`
//! so the app labels it). A row applies only while the sense's `ko_def` and current `gloss` still
//! equal the row's, so an upstream change silently disables the fix.

use crate::common::Entry;
use serde_json::Value;
use std::collections::HashMap;

/// (ko_def, old_gloss, new_gloss)
type Fix = (String, String, String);

pub struct GlossFixes(HashMap<(String, usize), Fix>);

impl GlossFixes {
    pub fn parse(tsv: &str) -> GlossFixes {
        let mut m = HashMap::new();
        for line in tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            if let [id, sense, ko, old, new] = f[..] {
                if let Ok(s) = sense.parse() {
                    m.insert((id.to_string(), s), (ko.to_string(), old.to_string(), new.to_string()));
                }
            }
        }
        GlossFixes(m)
    }

    pub fn builtin() -> GlossFixes {
        Self::parse(include_str!("../data/gloss_fixes.tsv"))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Correct matching sense glosses of a krdict entry; returns how many were set.
    pub fn apply(&self, e: &mut Entry) -> usize {
        if self.0.is_empty() || e.source != "krdict" {
            return 0;
        }
        let mut n = 0;
        let Some(senses) = e.data.get_mut("senses").and_then(Value::as_array_mut) else { return 0 };
        for (si, s) in senses.iter_mut().enumerate() {
            let Some((ko, old, new)) = self.0.get(&(e.ext_id.clone(), si)) else { continue };
            if s.get("ko_def").and_then(Value::as_str) == Some(ko.as_str()) && s.get("gloss").and_then(Value::as_str) == Some(old.as_str()) {
                s["gloss"] = Value::String(new.clone());
                s["gloss_ai"] = Value::Bool(true);
                n += 1;
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry() -> Entry {
        Entry { source: "krdict", ext_id: "7".into(), data: json!({"senses": [{"ko_def": "뜻.", "gloss": "word for word"}]}), ..Entry::default() }
    }

    #[test]
    fn applies_on_match() {
        let t = GlossFixes::parse("# h\n7\t0\t뜻.\tword for word\tto be annoyed\n");
        assert_eq!(t.len(), 1);
        let mut e = entry();
        assert_eq!(t.apply(&mut e), 1);
        assert_eq!(e.data["senses"][0]["gloss"], "to be annoyed");
        assert_eq!(e.data["senses"][0]["gloss_ai"], true);
        let mut other = Entry { source: "kengdic", ..entry() };
        assert_eq!(t.apply(&mut other), 0);
    }

    #[test]
    fn skips_when_ko_def_or_old_gloss_differ() {
        for row in ["7\t0\t다른 뜻.\tword for word\tX", "7\t0\t뜻.\tchanged upstream\tX", "7\t1\t뜻.\tword for word\tX", "8\t0\t뜻.\tword for word\tX"] {
            let mut e = entry();
            assert_eq!(GlossFixes::parse(row).apply(&mut e), 0, "{row}");
            assert_eq!(e.data["senses"][0]["gloss"], "word for word");
            assert!(e.data["senses"][0].get("gloss_ai").is_none());
        }
    }
}
