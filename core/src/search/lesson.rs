//! Grammar mini-lessons (pack table `grammar_lesson`, built by the pipeline from the reviewed
//! `data/grammar_lessons/*.json`; only approved lessons are in the pack). A lesson belongs to a curriculum
//! row and shows on the page of the dictionary entry that row links to.
//!
//! A pack without the table (older packs) gives no lessons.

use super::PackDb;
use crate::sql::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GrammarLesson {
    /// Curriculum level 1-6.
    pub level: i64,
    /// The official list's form (`-거들랑1`: the digit tells homonym rows apart).
    pub form: String,
    /// The lesson as JSON text (meaning_en, usage, examples, compare, pitfalls, topik_tip); the app parses it.
    pub body: String,
}

/// The lessons of entry (`source`, `id`), lowest level first (an entry can be several list rows:
/// `-거들랑1` / `-거들랑2`, or a pattern the list has at two levels).
pub fn grammar_lessons(packs: &[&PackDb], source: &str, id: i64) -> Result<Vec<GrammarLesson>> {
    let mut out = Vec::new();
    for p in packs.iter().filter(|p| p.caps.grammar_lesson) {
        let rows = p.conn.query(
            "SELECT l.level, l.form, l.body FROM grammar_lesson l JOIN entries e ON e.id = l.entry_id \
             WHERE l.entry_id = ?1 AND e.source = ?2 ORDER BY l.level, l.rowid",
            &[id.into(), source.into()],
        )?;
        out.extend(rows.iter().map(|r| GrammarLesson { level: r.int(0).unwrap_or(0), form: r.string(1), body: r.string(2) }));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::study::study_grammar;
    use crate::sql::Conn;

    fn pack(with_lessons: bool) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER, hanja TEXT, pos TEXT, pron TEXT, \
             source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER, rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);
             CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL, category TEXT NOT NULL, level INTEGER, summary_en TEXT, sort INTEGER);
             CREATE TABLE curriculum (kind TEXT NOT NULL, level INTEGER NOT NULL, entry_id INTEGER, grammar_id INTEGER, form TEXT NOT NULL, rank INTEGER NOT NULL);
             INSERT INTO entries VALUES (1, '-거들랑', '-거들랑', NULL, NULL, 'ending', NULL, 'krdict', 'en', 3, 1, 'grammar', 'if', '{}'),
                                        (2, '-노라면', '-노라면', NULL, NULL, 'ending', NULL, 'hanppuri', 'en', 3, 2, 'grammar', 'if you keep', '{}'),
                                        (3, '-건대', '-건대', NULL, NULL, 'ending', NULL, 'krdict', 'en', 3, 3, 'grammar', 'confess', '{}');
             INSERT INTO grammar VALUES (1, 1, '-거들랑', 'Connective endings', 3, 'if', 1), (2, 2, '-노라면', 'Connective endings', 3, 'if you keep', 2), (3, 3, '-건대', 'Connective endings', 3, 'confess', 3);
             INSERT INTO curriculum VALUES ('grammar', 6, 1, 1, '-거들랑1', 1), ('grammar', 6, 1, 1, '-거들랑2', 2), ('grammar', 6, 2, 2, '-노라면', 3), ('grammar', 6, 3, 3, '-건대', 4);",
        )
        .unwrap();
        if with_lessons {
            c.exec(
                "CREATE TABLE grammar_lesson (level INTEGER NOT NULL, form TEXT NOT NULL, entry_id INTEGER, grammar_id INTEGER, body TEXT NOT NULL);
                 INSERT INTO grammar_lesson VALUES (6, '-거들랑2', 1, 1, '{\"meaning_en\":\"reason\"}'), (6, '-거들랑1', 1, 1, '{\"meaning_en\":\"if\"}'), (5, '-노라면', 2, 2, '{\"meaning_en\":\"keep\"}');",
            )
            .unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn lessons_of_an_entry_come_by_level_then_insertion() {
        let p = pack(true);
        let l = grammar_lessons(&[&p], "krdict", 1).unwrap();
        assert_eq!(l.iter().map(|x| x.form.as_str()).collect::<Vec<_>>(), vec!["-거들랑2", "-거들랑1"]);
        assert_eq!(l[0].level, 6);
        assert!(l[1].body.contains("\"if\""));
        assert_eq!(grammar_lessons(&[&p], "hanppuri", 2).unwrap().len(), 1);
    }

    #[test]
    fn an_entry_without_a_lesson_or_the_wrong_source_gives_nothing() {
        let p = pack(true);
        assert!(grammar_lessons(&[&p], "krdict", 3).unwrap().is_empty());
        assert!(grammar_lessons(&[&p], "hanppuri", 1).unwrap().is_empty());
        assert!(grammar_lessons(&[&p], "krdict", 99).unwrap().is_empty());
    }

    #[test]
    fn a_pack_without_the_table_has_no_lessons() {
        let p = pack(false);
        assert!(!p.caps.grammar_lesson);
        assert!(grammar_lessons(&[&p], "krdict", 1).unwrap().is_empty());
        assert!(study_grammar(&[&p], 6).unwrap().iter().all(|g| !g.has_lesson));
    }

    #[test]
    fn study_rows_say_which_forms_have_a_lesson() {
        let p = pack(true);
        let g = study_grammar(&[&p], 6).unwrap();
        // the lesson's form and level must both match the curriculum row: -노라면's lesson is level 5
        assert_eq!(g.iter().map(|x| (x.form.as_str(), x.has_lesson)).collect::<Vec<_>>(), vec![("-거들랑1", true), ("-거들랑2", true), ("-노라면", false), ("-건대", false)]);
    }
}
