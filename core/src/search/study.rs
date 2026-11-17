//! Built-in study lists (Lists tab): the official curriculum's words and grammar by level (table
//! `curriculum`) and krdict's topic categories (table `topic`). Every list is read in pages, in a
//! stable order, so the app can show 2,500 rows without loading them all.
//!
//! A pack without the tables gives empty results (the app then hides that list).

use super::home::first_items;
use super::{HomeEntryRef, PackDb};
use crate::sql::{Result, Row, Val};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyCount {
    pub level: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyTopic {
    /// "인간 > 감각": top-level category, then sub-category.
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StudyOverview {
    /// Words per curriculum level (1-6) that are matched to a dictionary entry.
    pub vocab: Vec<StudyCount>,
    /// Grammar patterns per curriculum level (1-6).
    pub grammar: Vec<StudyCount>,
    /// Words per krdict category, sorted by category.
    pub topics: Vec<StudyTopic>,
    /// Idioms & proverbs lists (`study_item`), in list order; see `study_idioms.rs`.
    pub idioms: Vec<super::study_idioms::StudyIdiomList>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyWord {
    pub source: String,
    pub id: i64,
    pub headword: String,
    pub homonym: Option<i64>,
    pub hanja: Option<String>,
    pub pos: Option<String>,
    /// krdict level (1 beginner, 2 intermediate, 3 advanced).
    pub level: Option<i64>,
    pub gloss: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StudyGrammar {
    /// The pattern as the official list writes it.
    pub form: String,
    /// The linked grammar row's pattern (what a saved item is keyed on).
    pub pattern: String,
    pub summary_en: Option<String>,
    pub entry: Option<HomeEntryRef>,
}

const WORD_COLS: &str = "e.source, e.id, e.headword, e.homonym, e.hanja, e.pos, e.level, e.gloss";

fn study_word(r: &Row) -> StudyWord {
    StudyWord {
        source: r.string(0),
        id: r.int(1).unwrap_or(0),
        headword: r.string(2),
        homonym: r.int(3),
        hanja: r.text(4),
        pos: r.text(5),
        level: r.int(6),
        gloss: first_items(&r.string(7)),
    }
}

/// The first pack that has `curriculum` (krdict lives in the core pack).
fn curriculum_pack<'a>(packs: &[&'a PackDb]) -> Option<&'a PackDb> {
    packs.iter().copied().find(|p| p.caps.curriculum && !p.caps.texts)
}
fn topic_pack<'a>(packs: &[&'a PackDb]) -> Option<&'a PackDb> {
    packs.iter().copied().find(|p| p.caps.topic && !p.caps.texts)
}

/// Counts for the Lists tab: every list is shown only when it has rows.
pub fn study_overview(packs: &[&PackDb]) -> Result<StudyOverview> {
    let mut o = StudyOverview::default();
    if let Some(p) = curriculum_pack(packs) {
        let counts = |sql: &str| -> Result<Vec<StudyCount>> {
            Ok(p.conn.query(sql, &[])?.iter().map(|r| StudyCount { level: r.int(0).unwrap_or(0), count: r.int(1).unwrap_or(0) }).collect())
        };
        o.vocab = counts(
            "SELECT level, COUNT(DISTINCT entry_id) FROM curriculum WHERE kind = 'word' AND entry_id IS NOT NULL GROUP BY level ORDER BY level",
        )?;
        if p.caps.grammar {
            o.grammar = counts("SELECT level, COUNT(*) FROM curriculum WHERE kind = 'grammar' GROUP BY level ORDER BY level")?;
        }
    }
    if let Some(p) = topic_pack(packs) {
        o.topics = p
            .conn
            .query("SELECT category, COUNT(*) FROM topic GROUP BY category ORDER BY category", &[])?
            .iter()
            .map(|r| StudyTopic { category: r.string(0), count: r.int(1).unwrap_or(0) })
            .collect();
    }
    o.idioms = super::study_idioms::idiom_lists(packs)?;
    Ok(o)
}

/// Words of curriculum `level` in the official list's order (`rank`), one row per entry.
pub fn study_words(packs: &[&PackDb], level: i64, limit: usize, offset: usize) -> Result<Vec<StudyWord>> {
    let Some(p) = curriculum_pack(packs) else { return Ok(vec![]) };
    let sql = format!(
        "SELECT {WORD_COLS} FROM (SELECT entry_id, MIN(rank) AS r FROM curriculum \
         WHERE kind = 'word' AND level = ?1 AND entry_id IS NOT NULL GROUP BY entry_id ORDER BY r, entry_id LIMIT ?2 OFFSET ?3) c \
         JOIN entries e ON e.id = c.entry_id ORDER BY c.r, e.id"
    );
    let rows = p.conn.query(&sql, &[level.into(), limit.into(), offset.into()])?;
    Ok(rows.iter().map(study_word).collect())
}

/// Grammar patterns of curriculum `level` in the official order (a level has at most about seventy).
pub fn study_grammar(packs: &[&PackDb], level: i64) -> Result<Vec<StudyGrammar>> {
    let Some(p) = curriculum_pack(packs).filter(|p| p.caps.grammar) else { return Ok(vec![]) };
    let rows = p.conn.query(
        "SELECT c.form, g.pattern, g.summary_en, e.source, e.id, e.homonym, e.pos \
         FROM curriculum c LEFT JOIN grammar g ON g.id = c.grammar_id \
         LEFT JOIN entries e ON e.id = COALESCE(c.entry_id, g.entry_id) \
         WHERE c.kind = 'grammar' AND c.level = ?1 ORDER BY c.rank, c.rowid",
        &[level.into()],
    )?;
    Ok(rows
        .iter()
        .map(|r| {
            let form = r.string(0);
            StudyGrammar {
                pattern: r.text(1).unwrap_or_else(|| form.clone()),
                form,
                summary_en: r.text(2).filter(|s| !s.trim().is_empty()),
                entry: r.int(4).map(|id| HomeEntryRef { source: r.string(3), id, homonym: r.int(5), pos: r.text(6) }),
            }
        })
        .collect())
}

/// Words of a topic: `category` is a full category ("인간 > 감각") or a top-level one ("인간", all its
/// sub-categories). `by_level` orders 초급, 중급, 고급 first, each in 가나다 order; otherwise 가나다 only.
pub fn study_topic_words(packs: &[&PackDb], category: &str, by_level: bool, limit: usize, offset: usize) -> Result<Vec<StudyWord>> {
    let Some(p) = topic_pack(packs) else { return Ok(vec![]) };
    let order = if by_level { "e.level IS NULL, e.level, e.hw_norm, e.id" } else { "e.hw_norm, e.id" };
    let (clause, mut params): (&str, Vec<Val>) = if category.contains(" > ") {
        ("t.category = ?1", vec![category.into()])
    } else {
        // every "X > ..." sorts between "X > " and "X ?" ('?' follows '>')
        ("t.category >= ?1 AND t.category < ?2", vec![format!("{category} > ").into(), format!("{category} ?").into()])
    };
    let n = params.len();
    params.push(limit.into());
    params.push(offset.into());
    let sql = format!(
        "SELECT {WORD_COLS} FROM topic t JOIN entries e ON e.id = t.entry_id WHERE {clause} ORDER BY {order} LIMIT ?{} OFFSET ?{}",
        n + 1,
        n + 2
    );
    Ok(p.conn.query(&sql, &params)?.iter().map(study_word).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;

    const ENTRIES: &str = "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER, hanja TEXT, pos TEXT, pron TEXT, \
         source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER, rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);";

    fn pack(with_topic: bool) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(ENTRIES).unwrap();
        c.exec(
            "CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL, category TEXT NOT NULL, level INTEGER, summary_en TEXT, sort INTEGER);
             CREATE TABLE curriculum (kind TEXT NOT NULL, level INTEGER NOT NULL, entry_id INTEGER, grammar_id INTEGER, form TEXT NOT NULL, rank INTEGER NOT NULL);",
        )
        .unwrap();
        // ids are not in alphabetical order, so any ordering bug shows
        for (id, hw, level, gloss) in [
            (1i64, "하늘", Some(1i64), "sky; heaven; air; weather"),
            (2, "나무", Some(1), "tree"),
            (3, "다리", Some(2), "leg; bridge"),
            (4, "가방", None, "bag"),
            (5, "마음", Some(3), "heart"),
            (6, "바다", Some(1), "sea"),
        ] {
            c.query(
                "INSERT INTO entries VALUES (?1, ?2, ?2, NULL, NULL, 'noun', NULL, 'krdict', 'en', ?3, ?1, 'word', ?4, '{}')",
                &[id.into(), hw.into(), level.into(), gloss.into()],
            )
            .unwrap();
        }
        c.exec("INSERT INTO entries VALUES (7, '-고', '-고', NULL, NULL, 'ending', NULL, 'krdict', 'en', 1, 7, 'grammar', 'and', '{}')").unwrap();
        // level 1: official order 6, 2, 1 (1 listed twice) and one unmatched row; level 2: entry 3
        c.exec(
            "INSERT INTO curriculum VALUES ('word', 1, 6, NULL, '바다', 10), ('word', 1, 2, NULL, '나무', 20), ('word', 1, 1, NULL, '하늘', 30), \
             ('word', 1, 1, NULL, '하늘2', 31), ('word', 1, NULL, NULL, '없음', 40), ('word', 2, 3, NULL, '다리', 5)",
        )
        .unwrap();
        c.exec(
            "INSERT INTO grammar VALUES (1, 7, '-고', 'Connective endings', 1, 'and; then', 1), (2, NULL, '-아서', 'Connective endings', 1, '  ', 2);
             INSERT INTO curriculum VALUES ('grammar', 1, 7, 1, '-고', 2), ('grammar', 1, NULL, 2, '-아서/어서', 1), ('grammar', 1, NULL, NULL, '-기', 3), ('grammar', 3, 7, 1, '-고', 1)",
        )
        .unwrap();
        if with_topic {
            c.exec(
                "CREATE TABLE topic (entry_id INTEGER NOT NULL, category TEXT NOT NULL);
                 INSERT INTO topic VALUES (1, '자연 > 날씨'), (2, '동식물 > 식물'), (3, '인간 > 신체'), (4, '인간 > 물건'), (5, '인간 > 감정'), (6, '자연 > 지형'), (3, '인간 감각 > x')",
            )
            .unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn overview_counts_matched_words_and_all_grammar_rows() {
        let o = study_overview(&[&pack(true)]).unwrap();
        // level 1: three distinct entries (the repeated 하늘 and the unmatched row do not count)
        assert_eq!(o.vocab, vec![StudyCount { level: 1, count: 3 }, StudyCount { level: 2, count: 1 }]);
        assert_eq!(o.grammar, vec![StudyCount { level: 1, count: 3 }, StudyCount { level: 3, count: 1 }]);
        assert_eq!(o.topics.len(), 7);
        assert_eq!(o.topics[0], StudyTopic { category: "동식물 > 식물".into(), count: 1 });
    }

    #[test]
    fn missing_tables_give_empty_lists() {
        let p = pack(false);
        assert!(study_overview(&[&p]).unwrap().topics.is_empty());
        assert!(study_topic_words(&[&p], "인간", true, 10, 0).unwrap().is_empty());
        let c = Conn::open_memory().unwrap();
        c.exec(ENTRIES).unwrap();
        let old = PackDb::new("core", c).unwrap();
        assert_eq!(study_overview(&[&old]).unwrap(), StudyOverview::default());
        assert!(study_words(&[&old], 1, 10, 0).unwrap().is_empty());
        assert!(study_grammar(&[&old], 1).unwrap().is_empty());
    }

    #[test]
    fn words_follow_the_official_order_and_page() {
        let p = pack(true);
        let ids = |l: i64, n: usize, o: usize| study_words(&[&p], l, n, o).unwrap().iter().map(|w| w.id).collect::<Vec<_>>();
        assert_eq!(ids(1, 10, 0), vec![6, 2, 1]);
        assert_eq!(ids(1, 2, 0), vec![6, 2]);
        assert_eq!(ids(1, 2, 2), vec![1]);
        assert!(ids(1, 2, 3).is_empty());
        assert_eq!(ids(2, 10, 0), vec![3]);
        assert!(ids(6, 10, 0).is_empty());
        // gloss capped at three items
        assert_eq!(study_words(&[&p], 1, 10, 0).unwrap()[2].gloss, "sky; heaven; air");
    }

    #[test]
    fn grammar_keeps_the_official_form_and_links() {
        let g = study_grammar(&[&pack(true)], 1).unwrap();
        assert_eq!(g.iter().map(|x| x.form.as_str()).collect::<Vec<_>>(), vec!["-아서/어서", "-고", "-기"]);
        // blank summaries are dropped; an unlinked form is its own pattern; a link gives the entry
        assert_eq!((g[0].pattern.as_str(), g[0].summary_en.as_deref(), g[0].entry.is_none()), ("-아서", None, true));
        assert_eq!(g[1].summary_en.as_deref(), Some("and; then"));
        assert_eq!(g[1].entry.as_ref().map(|e| e.id), Some(7));
        assert_eq!((g[2].pattern.as_str(), g[2].summary_en.is_none()), ("-기", true));
    }

    #[test]
    fn topics_sort_by_level_then_alphabet_and_match_top_level_prefixes() {
        let p = pack(true);
        let ids = |c: &str, by_level: bool| study_topic_words(&[&p], c, by_level, 10, 0).unwrap().iter().map(|w| w.id).collect::<Vec<_>>();
        // 인간: 3 (중급 다리), 4 (no level 가방), 5 (고급 마음); "인간 감각 > x" is another top-level category
        assert_eq!(ids("인간", true), vec![3, 5, 4]);
        assert_eq!(ids("인간", false), vec![4, 3, 5]);
        assert_eq!(ids("인간 > 신체", true), vec![3]);
        // two sub-categories: level 1 words, 가나다 within the level
        assert_eq!(ids("자연", true), vec![6, 1]);
        assert_eq!(ids("자연", false), vec![6, 1]);
        assert!(ids("인", true).is_empty());
        // paging
        assert_eq!(study_topic_words(&[&p], "인간", true, 1, 1).unwrap()[0].id, 5);
    }
}
