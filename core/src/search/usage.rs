//! "Writing & usage" study lists (Lists > Study lists): 쓰기 expressions for TOPIK writing (table `writing_expr`) and
//! confusable-word groups (table `confusable`), built by the pipeline from reviewed AI-written data
//! (see `pipeline/src/usage_lists.rs`). Both are small (about 160 and 100 rows), so each is returned whole, in order.
//!
//! A pack without the tables (older packs) gives nothing and the app hides the lists.

use super::{HomeEntryRef, PackDb};
use crate::sql::Result;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct UsageCounts {
    pub writing: i64,
    pub confusables: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Sentence {
    pub ko: String,
    pub en: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WritingExpr {
    pub id: String,
    pub group: String,
    pub expr: String,
    pub meaning: String,
    /// Register note: written / formal, and the spoken equivalents to avoid in an essay.
    pub register: String,
    pub examples: Vec<Sentence>,
    /// The dictionary or grammar entry of the expression, when there is one.
    pub entry: Option<HomeEntryRef>,
    /// AI-written (always set for shipped rows; the app labels it).
    pub ai: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfusableWord {
    pub word: String,
    /// A spelling people write but that is wrong (no sentence, no entry).
    pub wrong: bool,
    pub ko: String,
    pub en: String,
    pub entry: Option<HomeEntryRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Confusable {
    pub id: String,
    /// "spelling", "meaning" or "grammar".
    pub kind: String,
    pub words: Vec<ConfusableWord>,
    pub explanation: String,
    pub mistake: String,
    pub ai: bool,
}

fn pack<'a>(packs: &[&'a PackDb], table: fn(&PackDb) -> bool) -> Option<&'a PackDb> {
    packs.iter().copied().find(|p| table(p) && !p.caps.texts)
}

/// Rows per list (0: the list is not in the pack).
pub fn usage_counts(packs: &[&PackDb]) -> Result<UsageCounts> {
    let count = |p: Option<&PackDb>, table: &str| -> Result<i64> {
        Ok(match p {
            Some(p) => p.conn.query(&format!("SELECT COUNT(*) FROM {table}"), &[])?.first().and_then(|r| r.int(0)).unwrap_or(0),
            None => 0,
        })
    };
    Ok(UsageCounts {
        writing: count(pack(packs, |p| p.caps.writing_expr), "writing_expr")?,
        confusables: count(pack(packs, |p| p.caps.confusable), "confusable")?,
    })
}

/// The entries behind a set of ids: id -> (source, homonym, pos).
fn entries(p: &PackDb, ids: &[i64]) -> Result<std::collections::HashMap<i64, HomeEntryRef>> {
    let mut out = std::collections::HashMap::new();
    for chunk in ids.chunks(200) {
        let list = chunk.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
        for r in p.conn.query(&format!("SELECT id, source, homonym, pos FROM entries WHERE id IN ({list})"), &[])? {
            if let Some(id) = r.int(0) {
                out.insert(id, HomeEntryRef { source: r.string(1), id, homonym: r.int(2), pos: r.text(3) });
            }
        }
    }
    Ok(out)
}

/// Every 쓰기 expression, in the order of the list.
pub fn writing_expressions(packs: &[&PackDb]) -> Result<Vec<WritingExpr>> {
    let Some(p) = pack(packs, |p| p.caps.writing_expr) else { return Ok(vec![]) };
    let rows = p.conn.query("SELECT id, grp, expr, meaning, register, ex, entry_id, ai FROM writing_expr ORDER BY rank", &[])?;
    let ids: Vec<i64> = rows.iter().filter_map(|r| r.int(6)).collect();
    let by_id = entries(p, &ids)?;
    Ok(rows
        .iter()
        .map(|r| {
            let ex: Value = r.text(5).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or(Value::Null);
            let examples = ex
                .as_array()
                .map(|a| a.iter().map(|e| Sentence { ko: e["ko"].as_str().unwrap_or("").to_string(), en: e["en"].as_str().unwrap_or("").to_string() }).collect())
                .unwrap_or_default();
            WritingExpr {
                id: r.string(0),
                group: r.string(1),
                expr: r.string(2),
                meaning: r.string(3),
                register: r.string(4),
                examples,
                entry: r.int(6).and_then(|id| by_id.get(&id).cloned()),
                ai: r.int(7).unwrap_or(1) != 0,
            }
        })
        .collect())
}

/// Every confusable group, in the order of the list.
pub fn confusables(packs: &[&PackDb]) -> Result<Vec<Confusable>> {
    let Some(p) = pack(packs, |p| p.caps.confusable) else { return Ok(vec![]) };
    let rows = p.conn.query("SELECT id, kind, words, explanation, mistake, ai FROM confusable ORDER BY rank", &[])?;
    let parsed: Vec<Value> = rows.iter().map(|r| r.text(2).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or(Value::Null)).collect();
    let ids: Vec<i64> = parsed.iter().flat_map(|v| v.as_array().into_iter().flatten().filter_map(|w| w["entry"].as_i64())).collect();
    let by_id = entries(p, &ids)?;
    Ok(rows
        .iter()
        .zip(&parsed)
        .map(|(r, v)| Confusable {
            id: r.string(0),
            kind: r.string(1),
            words: v
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|w| ConfusableWord {
                            word: w["w"].as_str().unwrap_or("").to_string(),
                            wrong: w["wrong"].as_bool().unwrap_or(false),
                            ko: w["ko"].as_str().unwrap_or("").to_string(),
                            en: w["en"].as_str().unwrap_or("").to_string(),
                            entry: w["entry"].as_i64().and_then(|id| by_id.get(&id).cloned()),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            explanation: r.string(3),
            mistake: r.string(4),
            ai: r.int(5).unwrap_or(1) != 0,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;

    fn pack(with_tables: bool) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER, pos TEXT, source TEXT NOT NULL, lang TEXT NOT NULL, rank INTEGER NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL);
             INSERT INTO entries VALUES (1, '다르다', '다르다', 1, 'adjective', 'krdict', 'ko', 1, 'word', '{}'), (2, '-기 때문', '-기 때문', NULL, NULL, 'krdict', 'ko', 2, 'grammar', '{}');",
        )
        .unwrap();
        if with_tables {
            c.exec(
                "CREATE TABLE writing_expr (rank INTEGER PRIMARY KEY, id TEXT NOT NULL, grp TEXT NOT NULL, expr TEXT NOT NULL, meaning TEXT NOT NULL, register TEXT NOT NULL, ex TEXT NOT NULL, entry_id INTEGER, ai INTEGER NOT NULL DEFAULT 1);
                 CREATE TABLE confusable (rank INTEGER PRIMARY KEY, id TEXT NOT NULL, kind TEXT NOT NULL, words TEXT NOT NULL, explanation TEXT NOT NULL, mistake TEXT NOT NULL, ai INTEGER NOT NULL DEFAULT 1);
                 INSERT INTO writing_expr VALUES (1, 'W001', 'Reasons', '-기 때문이다', 'the reason is that', 'Written, formal.', '[{\"ko\":\"가.\",\"en\":\"A.\"},{\"ko\":\"나.\",\"en\":\"B.\"}]', 2, 1),
                                                 (2, 'W002', 'Reasons', '결론적으로', 'in conclusion', 'Written.', '[]', NULL, 1);
                 INSERT INTO confusable VALUES (1, 'C001', 'meaning', '[{\"w\":\"다르다\",\"wrong\":false,\"ko\":\"다르다.\",\"en\":\"Differ.\",\"entry\":1},{\"w\":\"다릅다\",\"wrong\":true,\"ko\":\"-\",\"en\":\"-\",\"entry\":null}]', 'x', 'y', 1);",
            )
            .unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn rows_come_back_whole_in_order_with_their_entries() {
        let p = pack(true);
        assert_eq!(usage_counts(&[&p]).unwrap(), UsageCounts { writing: 2, confusables: 1 });
        let w = writing_expressions(&[&p]).unwrap();
        assert_eq!(w.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["W001", "W002"]);
        assert_eq!((w[0].examples.len(), w[0].examples[1].en.as_str(), w[0].ai), (2, "B.", true));
        assert_eq!(w[0].entry.as_ref().map(|e| (e.source.as_str(), e.id)), Some(("krdict", 2)));
        assert!(w[1].entry.is_none() && w[1].examples.is_empty());
        let c = confusables(&[&p]).unwrap();
        assert_eq!(c[0].words.len(), 2);
        assert_eq!((c[0].words[0].entry.as_ref().map(|e| e.id), c[0].words[0].wrong), (Some(1), false));
        assert!(c[0].words[1].wrong && c[0].words[1].entry.is_none());
    }

    #[test]
    fn an_older_pack_gives_nothing() {
        let p = pack(false);
        assert_eq!(usage_counts(&[&p]).unwrap(), UsageCounts::default());
        assert!(writing_expressions(&[&p]).unwrap().is_empty());
        assert!(confusables(&[&p]).unwrap().is_empty());
    }
}
