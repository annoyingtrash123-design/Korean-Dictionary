//! Precomputed Home-card eligibility (`curriculum.home_ok`, `study_item.home_ok`).
//!
//! The Home tab used to read every candidate entry's JSON at runtime (about 2 s cold for the ~5k
//! curriculum 5-6급 words) to find out which entries can be a card. The rules live in
//! `kdict_core::search` (`word_home_ok`, `idiom_home_ok`; grammar: a non-blank English summary) and
//! are applied here once, so the engine only reads the ids. Packs without the columns still work
//! (the engine falls back to the runtime rules), see docs/SCOPE.md.

use anyhow::Result;
use kdict_core::search::{idiom_home_ok, word_home_ok};
use rusqlite::{params, Connection};

/// Add and fill `curriculum.home_ok` (needs `entries`, `grammar`, `curriculum`). Returns (word rows, grammar rows) flagged.
pub fn mark_curriculum(conn: &Connection) -> Result<(usize, usize)> {
    conn.execute_batch("ALTER TABLE curriculum ADD COLUMN home_ok INTEGER NOT NULL DEFAULT 0")?;
    let rows: Vec<i64> = {
        let mut st = conn.prepare("SELECT c.rowid, e.level, e.gloss, e.data FROM curriculum c JOIN entries e ON e.id = c.entry_id WHERE c.kind = 'word'")?;
        let v = st
            .query_map([], |r| {
                let data: String = r.get(3)?;
                let ok = word_home_ok(r.get(1)?, r.get::<_, Option<String>>(2)?.as_deref(), &data);
                Ok((r.get::<_, i64>(0)?, ok))
            })?
            .filter_map(|r| match r {
                Ok((id, true)) => Some(Ok(id)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<std::result::Result<_, _>>()?;
        v
    };
    {
        let mut up = conn.prepare("UPDATE curriculum SET home_ok = 1 WHERE rowid = ?")?;
        for rowid in &rows {
            up.execute(params![rowid])?;
        }
    }
    let ng = conn.execute(
        "UPDATE curriculum SET home_ok = 1 WHERE kind = 'grammar' AND grammar_id IN (SELECT id FROM grammar WHERE summary_en IS NOT NULL AND trim(summary_en) != '')",
        [],
    )?;
    Ok((rows.len(), ng))
}

/// Add and fill `study_item.home_ok` (needs `entries` in the same file): a krdict (or reviewed hanppuri idiom) entry of the core pack with an
/// English gloss that passes `idiom_home_ok`. Returns the number flagged.
pub fn mark_study(conn: &Connection) -> Result<usize> {
    conn.execute_batch("ALTER TABLE study_item ADD COLUMN home_ok INTEGER NOT NULL DEFAULT 0")?;
    let rows: Vec<(String, i64)> = {
        let mut st = conn.prepare(
            "SELECT s.list, s.rank, e.headword, e.data FROM study_item s JOIN entries e ON e.id = s.entry_id \
             WHERE s.pack = 'core' AND e.source IN ('krdict', 'hanppuri') AND e.gloss IS NOT NULL AND trim(e.gloss) != ''",
        )?;
        let v = st
            .query_map([], |r| {
                let (hw, data): (String, String) = (r.get(2)?, r.get(3)?);
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, idiom_home_ok(&hw, &data)))
            })?
            .filter_map(|r| match r {
                Ok((l, k, true)) => Some(Ok((l, k))),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<std::result::Result<_, _>>()?;
        v
    };
    let mut up = conn.prepare("UPDATE study_item SET home_ok = 1 WHERE list = ? AND rank = ?")?;
    for (l, k) in &rows {
        up.execute(params![l, k])?;
    }
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_follow_the_engine_rules() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            r#"CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT, source TEXT, level INTEGER, gloss TEXT, data TEXT);
               CREATE TABLE grammar (id INTEGER PRIMARY KEY, summary_en TEXT);
               CREATE TABLE curriculum (kind TEXT, level INTEGER, entry_id INTEGER, grammar_id INTEGER, form TEXT, rank INTEGER);
               CREATE TABLE study_item (list TEXT, rank INTEGER, entry_id INTEGER, pack TEXT, PRIMARY KEY (list, rank));
               INSERT INTO entries VALUES
                 (1, '가', 'krdict', 5, 'a', '{"senses":[{"examples":[{"ko":"문장.","type":"sentence"}]}]}'),
                 (2, '나', 'krdict', NULL, 'a', '{"senses":[{"examples":[{"ko":"문장.","type":"sentence"}]}]}'),
                 (3, '다', 'krdict', 5, '', '{"senses":[{"examples":[{"ko":"문장.","type":"sentence"}]}]}'),
                 (4, '라', 'krdict', 5, 'a', '{"senses":[{"examples":[{"ko":"구절","type":"phrase"}]}]}'),
                 (5, '말이 ∼지', 'krdict', NULL, 'a', '{"senses":[{"examples":[{"ko":"문장.","type":"sentence"}]}]}');
               INSERT INTO grammar VALUES (1, 'x'), (2, '  '), (3, NULL);
               INSERT INTO curriculum VALUES ('word',5,1,NULL,'a',1),('word',5,2,NULL,'b',2),('word',5,3,NULL,'c',3),('word',5,4,NULL,'d',4),('word',5,NULL,NULL,'e',5),
                 ('grammar',5,NULL,1,'g',6),('grammar',5,NULL,2,'g',7),('grammar',5,NULL,3,'g',8);
               INSERT INTO study_item VALUES ('l',1,1,'core'),('l',2,5,'core'),('l',3,1,'stdict'),('l',4,3,'core');"#,
        )
        .unwrap();
        // word 1 is eligible (the sentence example has no `"examples":[{` gap issue); 2 has no level, 3 no gloss, 4 only a phrase
        let (w, g) = mark_curriculum(&c).unwrap();
        assert_eq!((w, g), (1, 1));
        let flagged: Vec<i64> = c.prepare("SELECT rank FROM curriculum WHERE home_ok = 1 ORDER BY rank").unwrap().query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert_eq!(flagged, vec![1, 6]);
        // study: row 1 only (2 is a frame, 3 is another pack, 4 has no gloss)
        assert_eq!(mark_study(&c).unwrap(), 1);
        let s: i64 = c.query_row("SELECT rank FROM study_item WHERE home_ok = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(s, 1);
    }
}
