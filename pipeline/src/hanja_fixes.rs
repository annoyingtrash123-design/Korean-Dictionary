//! Reviewed corrections to the hanja of krdict core entries (`data/hanja_fixes.tsv`) and the reviewed keep list
//! (`data/hanja_keep.tsv`) that the suspect scan ([`crate::hanja_fill::suspect_core_hanja`]) skips.
//! A fix applies (in the built core pack, after the kengdic fallback, before the stdict fill) only while the
//! entry's hanja still equals `old_hanja`; the entry gets `"hanja_fixed": true` and its `hanja_words` rows follow.

use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    pub ext_id: String,
    pub headword: String,
    pub old: String,
    /// `None`: remove (the word is native in this sense).
    pub new: Option<String>,
}

pub struct HanjaFixes {
    pub fixes: Vec<Fix>,
    /// (ext_id, hanja) reviewed and kept.
    pub keep: HashSet<(String, String)>,
}

fn rows(tsv: &str) -> impl Iterator<Item = Vec<&str>> {
    tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|l| l.split('\t').collect())
}

impl HanjaFixes {
    pub fn parse(fixes: &str, keep: &str) -> HanjaFixes {
        let fixes = rows(fixes)
            .filter_map(|f| match f[..] {
                [id, hw, old, new, _reason] => Some(Fix { ext_id: id.into(), headword: hw.into(), old: old.into(), new: (!new.is_empty()).then(|| new.to_string()) }),
                _ => None,
            })
            .collect();
        let keep = rows(keep).filter_map(|f| match f[..] {
            [id, _hw, h, _reason] => Some((id.to_string(), h.to_string())),
            _ => None,
        });
        HanjaFixes { fixes, keep: keep.collect() }
    }

    pub fn builtin() -> HanjaFixes {
        Self::parse(include_str!("../data/hanja_fixes.tsv"), include_str!("../data/hanja_keep.tsv"))
    }

    pub fn len(&self) -> usize {
        self.fixes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fixes.is_empty()
    }

    pub fn is_kept(&self, ext_id: &str, hanja: &str) -> bool {
        self.keep.contains(&(ext_id.to_string(), hanja.to_string()))
    }

    /// Apply to the core pack's krdict word entries; returns how many entries changed.
    pub fn apply(&self, conn: &Connection) -> Result<usize> {
        let mut n = 0;
        for f in &self.fixes {
            let ids: Vec<i64> = {
                let mut st = conn.prepare_cached("SELECT id FROM entries WHERE source = 'krdict' AND kind = 'word' AND ext_id = ?1 AND headword = ?2 AND hanja = ?3")?;
                let r = st.query_map(params![f.ext_id, f.headword, f.old], |r| r.get(0))?;
                r.collect::<std::result::Result<_, _>>()?
            };
            for id in ids {
                crate::hanja_fill::replace_hanja(conn, id, f.new.as_deref(), "hanja_fixed")?;
                n += 1;
            }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT, source TEXT, kind TEXT, ext_id TEXT, hanja TEXT, data TEXT);
             CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, rank INTEGER NOT NULL DEFAULT 0);
             INSERT INTO entries VALUES (1, '구경', 'krdict', 'word', '7', '究竟', '{\"senses\":[]}');
             INSERT INTO entries VALUES (2, '구경도 못하다', 'krdict', 'idiom', '7', '究竟', '{}');
             INSERT INTO entries VALUES (3, '종지', 'krdict', 'word', '8', '宗旨', '{}');
             INSERT INTO hanja_words(ch, entry_id) VALUES ('究',1),('竟',1),('究',2),('宗',3),('旨',3);",
        )
        .unwrap();
        c
    }
    fn hw(c: &Connection, id: i64) -> Vec<String> {
        let mut st = c.prepare("SELECT ch FROM hanja_words WHERE entry_id = ? ORDER BY rowid").unwrap();
        st.query_map([id], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    }

    #[test]
    fn removes_and_changes_only_on_match() {
        let t = HanjaFixes::parse("# h\n7\t구경\t究竟\t\tnative\n8\t종지\t宗旨\t鍾子\ttest\n9\t멱\t蓂\t\tgone upstream\n", "70748\t동생\t同生\tetymology\n");
        assert_eq!(t.len(), 3);
        assert!(t.is_kept("70748", "同生") && !t.is_kept("70748", "同牲"));
        let c = db();
        assert_eq!(t.apply(&c).unwrap(), 2);
        let (h, d): (Option<String>, String) = c.query_row("SELECT hanja, data FROM entries WHERE id = 1", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(h, None);
        assert!(d.contains("\"hanja_fixed\":true"), "{d}");
        assert!(hw(&c, 1).is_empty());
        // the idiom with the same ext_id is a different entry
        assert_eq!(hw(&c, 2), ["究"]);
        assert_eq!(c.query_row("SELECT hanja FROM entries WHERE id = 3", [], |r| r.get::<_, String>(0)).unwrap(), "鍾子");
        assert_eq!(hw(&c, 3), ["鍾", "子"]);
        // idempotent: old_hanja no longer matches
        assert_eq!(t.apply(&c).unwrap(), 0);
    }
}
