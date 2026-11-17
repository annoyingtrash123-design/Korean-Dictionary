//! "Idioms & proverbs" study lists (Lists tab): the most common 속담 (proverbs) and 성어 (four-character
//! idioms), ranked by corpus frequency at build time (table `study_item`, see pipeline/src/study_idioms.rs).
//!
//! A row's entry may live in another pack than core (stdict / 우리말샘 hold most of them): it is found
//! again here by headword key + `ext_id` when that pack is installed, and is `None` otherwise (the app
//! then opens a search for the title). A pack without the table gives no lists (the app hides the group).

use super::{HomeEntryRef, PackDb};
use crate::sql::{Result, Val};
use serde::Serialize;

/// The lists in the order the app shows them.
const LISTS: [&str; 4] = ["sokdam100", "sokdam_all", "seongeo100", "seongeo300"];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyIdiomList {
    pub list: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyIdiom {
    pub rank: i64,
    /// "sokdam" (속담, proverb) or "seongeo" (성어, four-character idiom).
    pub kind: String,
    pub title: String,
    /// The entry's exact headword in its pack (what a saved item is keyed on); `title` is the clean text.
    pub headword: String,
    /// English gloss when krdict has one, else the Korean definition (shortened at build time).
    pub gloss: String,
    /// Pack the entry belongs to ("core", "stdict", "opendict").
    pub pack: String,
    /// The entry, when its pack is installed.
    pub entry: Option<HomeEntryRef>,
}

fn study_pack<'a>(packs: &[&'a PackDb]) -> Option<&'a PackDb> {
    packs.iter().copied().find(|p| p.caps.study_item && !p.caps.texts)
}

/// Lists that have rows, in display order.
pub fn idiom_lists(packs: &[&PackDb]) -> Result<Vec<StudyIdiomList>> {
    let Some(p) = study_pack(packs) else { return Ok(vec![]) };
    let rows = p.conn.query("SELECT list, COUNT(*) FROM study_item GROUP BY list", &[])?;
    let count = |l: &str| rows.iter().find(|r| r.string(0) == l).and_then(|r| r.int(1)).unwrap_or(0);
    Ok(LISTS.iter().map(|l| StudyIdiomList { list: (*l).to_string(), count: count(l) }).filter(|l| l.count > 0).collect())
}

/// A page of list `list` in rank order.
pub fn study_idioms(packs: &[&PackDb], list: &str, limit: usize, offset: usize) -> Result<Vec<StudyIdiom>> {
    let Some(p) = study_pack(packs) else { return Ok(vec![]) };
    let rows = p.conn.query(
        "SELECT s.rank, s.kind, s.title, s.gloss, s.pack, s.ext_id, s.hw_norm, e.source, e.id, e.homonym, e.pos, s.headword \
         FROM study_item s LEFT JOIN entries e ON e.id = s.entry_id AND s.pack = 'core' \
         WHERE s.list = ?1 ORDER BY s.rank LIMIT ?2 OFFSET ?3",
        &[list.into(), limit.into(), offset.into()],
    )?;
    let mut out = Vec::with_capacity(rows.len());
    for r in &rows {
        let pack = r.string(4);
        let mut entry = r.int(8).map(|id| HomeEntryRef { source: r.string(7), id, homonym: r.int(9), pos: r.text(10) });
        if entry.is_none() && pack != "core" {
            // the entry is in an optional pack: found by headword key + the dictionary's own id (None when not installed)
            if let Some(other) = packs.iter().copied().find(|q| q.id == pack && !q.caps.texts) {
                let found = other
                    .conn
                    .query(
                        "SELECT id, homonym, pos FROM entries WHERE hw_norm = ?1 AND ext_id = ?2 AND source = ?3 ORDER BY rank LIMIT 1",
                        &[r.string(6).as_str().into(), r.string(5).as_str().into(), Val::from(pack.as_str())],
                    )
                    .unwrap_or_default();
                entry = found.first().and_then(|f| f.int(0).map(|id| HomeEntryRef { source: pack.clone(), id, homonym: f.int(1), pos: f.text(2) }));
            }
        }
        out.push(StudyIdiom { rank: r.int(0).unwrap_or(0), kind: r.string(1), title: r.string(2), headword: r.string(11), gloss: r.string(3), pack, entry });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;

    const ENTRIES: &str = "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER, hanja TEXT, pos TEXT, pron TEXT, \
         source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER, rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL, ext_id TEXT);";

    fn pack(id: &str, rows: &[(i64, &str, &str, &str)], items: &[&str]) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(ENTRIES).unwrap();
        for (eid, hw, source, ext) in rows {
            c.query(
                "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'noun', NULL, ?3, 'ko', NULL, ?1, 'word', 'g', '{}', ?4)",
                &[(*eid).into(), (*hw).into(), (*source).into(), (*ext).into()],
            )
            .unwrap();
        }
        if !items.is_empty() {
            c.exec("CREATE TABLE study_item (list TEXT NOT NULL, rank INTEGER NOT NULL, kind TEXT NOT NULL, title TEXT NOT NULL, entry_id INTEGER, pack TEXT NOT NULL, ext_id TEXT NOT NULL, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, gloss TEXT NOT NULL, PRIMARY KEY (list, rank));").unwrap();
            for sql in items {
                c.exec(sql).unwrap();
            }
        }
        PackDb::new(id, c).unwrap()
    }

    fn core() -> PackDb {
        pack(
            "core",
            &[(1, "누워서 떡 먹기", "krdict", "10")],
            &[
                "INSERT INTO study_item VALUES ('sokdam100', 1, 'sokdam', '누워서 떡 먹기', 1, 'core', '10', '누워서 떡 먹기', '누워서떡먹기', 'a piece of cake')",
                "INSERT INTO study_item VALUES ('sokdam100', 2, 'sokdam', '밑져야 본전', NULL, 'core', '99', '밑져야 본전', '밑져야본전', 'nothing to lose')",
                "INSERT INTO study_item VALUES ('seongeo100', 1, 'seongeo', '설상가상', NULL, 'stdict', '7', '설상가상', '설상가상', '눈 위에 서리가 덮인다는 뜻으로')",
                "INSERT INTO study_item VALUES ('seongeo100', 2, 'seongeo', '금상첨화', NULL, 'opendict', '8', '금상첨화', '금상첨화', 'x')",
                "INSERT INTO study_item VALUES ('seongeo300', 1, 'seongeo', '설상가상', NULL, 'stdict', '7', '설상가상', '설상가상', 'x')",
            ],
        )
    }

    #[test]
    fn lists_are_counted_in_display_order_and_hidden_when_empty() {
        let c = core();
        let l = idiom_lists(&[&c]).unwrap();
        let got: Vec<(&str, i64)> = l.iter().map(|x| (x.list.as_str(), x.count)).collect();
        assert_eq!(got, vec![("sokdam100", 2), ("seongeo100", 2), ("seongeo300", 1)]);
        // a pack without the table: no group at all
        let old = pack("core", &[], &[]);
        assert!(idiom_lists(&[&old]).unwrap().is_empty());
        assert!(study_idioms(&[&old], "sokdam100", 10, 0).unwrap().is_empty());
    }

    #[test]
    fn rows_page_in_rank_order_and_link_their_entry() {
        let c = core();
        let rows = study_idioms(&[&c], "sokdam100", 10, 0).unwrap();
        assert_eq!(rows.iter().map(|r| r.rank).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!((rows[0].kind.as_str(), rows[0].gloss.as_str()), ("sokdam", "a piece of cake"));
        assert_eq!(rows[0].entry, Some(HomeEntryRef { source: "krdict".into(), id: 1, homonym: None, pos: Some("noun".into()) }));
        // a core row whose entry is gone: still listed, opens a search
        assert_eq!(rows[1].entry, None);
        assert_eq!(study_idioms(&[&c], "sokdam100", 1, 1).unwrap()[0].title, "밑져야 본전");
        assert!(study_idioms(&[&c], "nope", 10, 0).unwrap().is_empty());
    }

    #[test]
    fn entries_in_optional_packs_are_found_only_when_installed() {
        let c = core();
        let st = pack("stdict", &[(500, "설상가상", "stdict", "7"), (501, "설상가상", "stdict", "other")], &[]);
        let od = pack("opendict", &[(600, "금상첨화", "opendict", "8")], &[]);
        // neither installed: both rows have no entry (the app opens a search)
        let rows = study_idioms(&[&c], "seongeo100", 10, 0).unwrap();
        assert!(rows.iter().all(|r| r.entry.is_none()));
        assert_eq!((rows[0].pack.as_str(), rows[1].pack.as_str()), ("stdict", "opendict"));
        // stdict installed: its entry is found by headword key + ext_id (not the other homonym)
        let rows = study_idioms(&[&c, &st], "seongeo100", 10, 0).unwrap();
        assert_eq!(rows[0].entry.as_ref().map(|e| (e.source.as_str(), e.id)), Some(("stdict", 500)));
        assert!(rows[1].entry.is_none());
        let rows = study_idioms(&[&c, &st, &od], "seongeo100", 10, 0).unwrap();
        assert_eq!(rows[1].entry.as_ref().map(|e| (e.source.as_str(), e.id)), Some(("opendict", 600)));
    }
}
