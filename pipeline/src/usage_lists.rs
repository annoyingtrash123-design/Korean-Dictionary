//! "Writing & usage" study lists (Lists > Study lists): 쓰기 expressions for TOPIK writing (`data/writing_expressions.tsv`) and
//! confusable words (`data/confusables.tsv`). Both files are AI-written (Claude); only rows whose `review` column says
//! `approved...` ship, into the core pack tables `writing_expr` and `confusable`, flagged `ai = 1` so the app labels them.
//!
//! Links to dictionary entries are not stored in the data: a 쓰기 row names a grammar row (`g:<pattern>`, matched with
//! [`crate::curriculum::grammar_key`] against the pack's `grammar` table) or a headword (`w:<headword>`); a confusable word is
//! looked up as a headword (krdict first; a word that starts with "-" as a grammar pattern). What cannot be resolved is left
//! without a link (the app then searches for the word) and is reported in [`Stats`].

use crate::curriculum::grammar_key;
use crate::grammar_lessons::{has_hangul, without_quotes};
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub const WRITING_HEADER: [&str; 11] = ["id", "group", "expression", "meaning_en", "register_note", "ex1_ko", "ex1_en", "ex2_ko", "ex2_en", "link", "review"];
pub const CONFUSABLE_HEADER: [&str; 8] = ["id", "kind", "words", "explanation_en", "ex_ko", "ex_en", "mistake_en", "review"];
pub const KINDS: [&str; 3] = ["spelling", "meaning", "grammar"];

#[derive(Debug, Clone, PartialEq)]
pub struct WritingRow {
    pub id: String,
    pub group: String,
    pub expression: String,
    pub meaning_en: String,
    pub register_note: String,
    pub ex: Vec<(String, String)>,
    pub link: String,
    pub review: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    /// A spelling people write but that is wrong (marked × in the file); it has no sentence.
    pub wrong: bool,
    pub ko: String,
    pub en: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfusableRow {
    pub id: String,
    pub kind: String,
    pub words: Vec<Word>,
    pub explanation_en: String,
    pub mistake_en: String,
    pub review: String,
}

fn body_lines(tsv: &str) -> impl Iterator<Item = &str> {
    tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty())
}

fn table<'a>(tsv: &'a str, header: &[&str], name: &str) -> Result<Vec<Vec<&'a str>>> {
    let mut lines = body_lines(tsv);
    let head: Vec<&str> = lines.next().with_context(|| format!("{name} has no header"))?.split('\t').collect();
    if head != header {
        bail!("{name} header: {head:?}");
    }
    lines
        .enumerate()
        .map(|(i, l)| {
            let f: Vec<&str> = l.split('\t').map(str::trim).collect();
            if f.len() != header.len() {
                bail!("{name} row {}: {} columns, expected {}", i + 1, f.len(), header.len());
            }
            Ok(f)
        })
        .collect()
}

pub fn parse_writing(tsv: &str) -> Result<Vec<WritingRow>> {
    Ok(table(tsv, &WRITING_HEADER, "writing_expressions.tsv")?
        .into_iter()
        .map(|f| WritingRow {
            id: f[0].into(),
            group: f[1].into(),
            expression: f[2].into(),
            meaning_en: f[3].into(),
            register_note: f[4].into(),
            ex: vec![(f[5].into(), f[6].into()), (f[7].into(), f[8].into())],
            link: f[9].into(),
            review: f[10].into(),
        })
        .collect())
}

pub fn parse_confusables(tsv: &str) -> Result<Vec<ConfusableRow>> {
    table(tsv, &CONFUSABLE_HEADER, "confusables.tsv")?
        .into_iter()
        .map(|f| {
            let words: Vec<&str> = f[2].split(" | ").collect();
            let (ko, en): (Vec<&str>, Vec<&str>) = (f[4].split(" | ").collect(), f[5].split(" | ").collect());
            if ko.len() != words.len() || en.len() != words.len() {
                bail!("{}: {} words but {} sentences / {} translations", f[0], words.len(), ko.len(), en.len());
            }
            Ok(ConfusableRow {
                id: f[0].into(),
                kind: f[1].into(),
                words: words
                    .iter()
                    .zip(ko.iter().zip(&en))
                    .map(|(w, (k, e))| Word { text: w.trim_start_matches('×').to_string(), wrong: w.starts_with('×'), ko: (*k).into(), en: (*e).into() })
                    .collect(),
                explanation_en: f[3].into(),
                mistake_en: f[6].into(),
                review: f[7].into(),
            })
        })
        .collect()
}

pub fn builtin_writing() -> Vec<WritingRow> {
    parse_writing(include_str!("../data/writing_expressions.tsv")).expect("writing_expressions.tsv parses")
}

pub fn builtin_confusables() -> Vec<ConfusableRow> {
    parse_confusables(include_str!("../data/confusables.tsv")).expect("confusables.tsv parses")
}

/// Ships only when the independent reviewer approved it.
pub fn ships(review: &str) -> bool {
    review.starts_with("approved")
}

/// Review column value: draft, approved..., rejected: why.
fn review_ok(r: &str) -> bool {
    r == "draft" || r.starts_with("approved") || r.starts_with("rejected: ")
}

/// English text may hold Korean only inside quotes.
fn stray_hangul(s: &str) -> bool {
    has_hangul(&without_quotes(s))
}

/// Spoken-register endings that must not appear in an essay sentence (한다체).
const SPOKEN: [&str; 6] = ["거든요", "잖아요", "해요", "에요", "예요", "어요."];

/// What is wrong with a 쓰기 row (empty: nothing).
pub fn writing_problems(r: &WritingRow) -> Vec<String> {
    let mut v = Vec::new();
    let mut bad = |c: bool, m: &str| {
        if c {
            v.push(format!("{}: {m}", r.id));
        }
    };
    let notice = r.group.contains("Notices");
    bad(!r.id.starts_with('W') || r.id.len() != 4 || !r.id[1..].chars().all(|c| c.is_ascii_digit()), "id must look like W001");
    bad(r.group.trim().is_empty(), "group is empty");
    bad(!has_hangul(&r.expression) || r.expression.contains('\t'), "expression must be Korean");
    bad(r.meaning_en.chars().count() < 6 || stray_hangul(&r.meaning_en), "meaning_en empty or has unquoted Korean");
    bad(!(r.register_note.starts_with("Written") || r.register_note.starts_with("Formal")), "register_note must start with Written or Formal");
    bad(stray_hangul(&r.register_note), "register_note has unquoted Korean");
    bad(r.register_note.chars().count() < 40, "register_note too short to say which spoken forms to avoid");
    bad(!(r.link.is_empty() || r.link.starts_with("g:") || r.link.starts_with("w:")), "link must be empty, g:<pattern> or w:<headword>");
    bad(!review_ok(&r.review), "review must be draft, approved..., or rejected: why");
    for (i, (ko, en)) in r.ex.iter().enumerate() {
        let n = i + 1;
        bad(!has_hangul(ko) || !ko.ends_with(['.', '!', '?']), &format!("ex{n}_ko must be a Korean sentence ending in a full stop"));
        bad(has_hangul(en) || en.chars().count() < 12, &format!("ex{n}_en must be English without Korean"));
        bad(SPOKEN.iter().any(|s| ko.contains(s)), &format!("ex{n}_ko uses a spoken ending"));
        bad(!notice && (ko.contains("습니다") || ko.contains("세요") || ko.contains("십시오")), &format!("ex{n}_ko is polite style outside the notices group (essays use 한다체)"));
        bad(ko.contains('\t') || en.contains('\t'), "tab inside a field");
    }
    v
}

/// What is wrong with a confusable group (empty: nothing).
pub fn confusable_problems(r: &ConfusableRow) -> Vec<String> {
    let mut v = Vec::new();
    let mut bad = |c: bool, m: &str| {
        if c {
            v.push(format!("{}: {m}", r.id));
        }
    };
    bad(!r.id.starts_with('C') || r.id.len() != 4 || !r.id[1..].chars().all(|c| c.is_ascii_digit()), "id must look like C001");
    bad(!KINDS.contains(&r.kind.as_str()), "kind must be spelling, meaning or grammar");
    bad(!(2..=4).contains(&r.words.len()), "a group has two to four words");
    bad(r.words.iter().all(|w| w.wrong), "a group needs a correct word");
    bad(r.words.iter().map(|w| &w.text).collect::<HashSet<_>>().len() != r.words.len(), "a word twice in a group");
    bad(r.explanation_en.chars().count() < 60 || stray_hangul(&r.explanation_en), "explanation_en short or has unquoted Korean");
    bad(r.mistake_en.chars().count() < 20 || stray_hangul(&r.mistake_en), "mistake_en short or has unquoted Korean");
    bad(!review_ok(&r.review), "review must be draft, approved..., or rejected: why");
    for w in &r.words {
        let t = &w.text;
        bad(t.is_empty() || !has_hangul(t) || t.contains(['\t', '|']), &format!("word {t:?}"));
        if w.wrong {
            bad(w.ko != "-" || w.en != "-", &format!("wrong spelling {t} has no sentence: write - in ex_ko and ex_en"));
        } else {
            bad(!has_hangul(&w.ko) || !w.ko.ends_with(['.', '!', '?']), &format!("sentence of {t} must be Korean ending in punctuation"));
            bad(has_hangul(&w.en) || w.en.chars().count() < 8, &format!("translation of {t} must be English without Korean"));
        }
    }
    v
}

pub fn check_all(w: &[WritingRow], c: &[ConfusableRow]) -> Vec<String> {
    let mut v: Vec<String> = w.iter().flat_map(writing_problems).chain(c.iter().flat_map(confusable_problems)).collect();
    let mut seen = HashSet::new();
    for id in w.iter().map(|r| &r.id).chain(c.iter().map(|r| &r.id)) {
        if !seen.insert(id) {
            v.push(format!("{id}: duplicate id"));
        }
    }
    v
}

// ---------------------------------------------------------------- links

/// Resolves links against the (finished) core pack: grammar rows by [`grammar_key`], headwords by exact text.
pub struct Linker<'a> {
    conn: &'a Connection,
    grammar: HashMap<String, i64>,
}

impl<'a> Linker<'a> {
    pub fn new(conn: &'a Connection) -> Result<Linker<'a>> {
        let mut grammar: HashMap<String, i64> = HashMap::new();
        let has: bool = conn.query_row("SELECT 1 FROM sqlite_master WHERE name = 'grammar'", [], |_| Ok(true)).optional()?.unwrap_or(false);
        if has {
            let mut st = conn.prepare("SELECT pattern, entry_id FROM grammar WHERE entry_id IS NOT NULL ORDER BY id")?;
            let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
            for r in rows {
                let (p, id) = r?;
                grammar.entry(grammar_key(&p)).or_insert(id);
            }
        }
        Ok(Linker { conn, grammar })
    }

    /// The entry of a grammar pattern: its key, then the key without a closing 에 (the official list writes `-기 때문에`, krdict `-기 때문`).
    pub fn grammar(&self, pattern: &str) -> Option<i64> {
        let k = grammar_key(pattern);
        self.grammar.get(&k).or_else(|| k.strip_suffix('에').and_then(|s| self.grammar.get(s))).copied()
    }

    /// The entry with this exact headword: krdict first, then the best-ranked one (never a grammar row of the same text).
    pub fn word(&self, headword: &str) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT id FROM entries WHERE headword = ?1 AND kind != 'grammar' ORDER BY (source = 'krdict') DESC, rank LIMIT 1",
                [headword],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    /// `g:<pattern>` or `w:<headword>` of a 쓰기 row.
    pub fn link(&self, spec: &str) -> Option<i64> {
        match spec.split_once(':') {
            Some(("g", p)) => self.grammar(p),
            Some(("w", w)) => self.word(w),
            _ => None,
        }
    }

    /// A confusable word: a headword, or a pattern when it starts with "-".
    pub fn confusable_word(&self, w: &str) -> Option<i64> {
        if w.starts_with('-') {
            self.grammar(w)
        } else {
            self.word(w)
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    pub writing: usize,
    pub writing_linked: usize,
    /// Links asked for in shipped 쓰기 rows that found no entry.
    pub writing_unresolved: Vec<String>,
    pub confusables: usize,
    pub words: usize,
    pub words_linked: usize,
    pub words_unresolved: Vec<String>,
}

/// Create `writing_expr` and `confusable` in the (finished) core pack from the approved rows of the built-in files.
pub fn add_tables(core: &Path) -> Result<Stats> {
    add_tables_with(core, &builtin_writing(), &builtin_confusables())
}

pub fn add_tables_with(core: &Path, writing: &[WritingRow], confusables: &[ConfusableRow]) -> Result<Stats> {
    let problems = check_all(writing, confusables);
    if !problems.is_empty() {
        bail!("usage lists data: {} problems, first: {}", problems.len(), problems[0]);
    }
    let conn = Connection::open(core)?;
    conn.execute_batch(
        "CREATE TABLE writing_expr (
           rank INTEGER PRIMARY KEY, id TEXT NOT NULL, grp TEXT NOT NULL, expr TEXT NOT NULL, meaning TEXT NOT NULL,
           register TEXT NOT NULL, ex TEXT NOT NULL, entry_id INTEGER, ai INTEGER NOT NULL DEFAULT 1
         );
         CREATE TABLE confusable (
           rank INTEGER PRIMARY KEY, id TEXT NOT NULL, kind TEXT NOT NULL, words TEXT NOT NULL,
           explanation TEXT NOT NULL, mistake TEXT NOT NULL, ai INTEGER NOT NULL DEFAULT 1
         );",
    )?;
    let linker = Linker::new(&conn)?;
    let mut stats = Stats::default();
    conn.execute_batch("BEGIN")?;
    for (i, r) in writing.iter().filter(|r| ships(&r.review)).enumerate() {
        let entry = if r.link.is_empty() { None } else { linker.link(&r.link) };
        if !r.link.is_empty() && entry.is_none() {
            stats.writing_unresolved.push(format!("{} {}", r.id, r.link));
        }
        let ex = json!(r.ex.iter().map(|(ko, en)| json!({"ko": ko, "en": en})).collect::<Vec<_>>()).to_string();
        conn.prepare_cached("INSERT INTO writing_expr VALUES (?1,?2,?3,?4,?5,?6,?7,?8,1)")?
            .execute(params![i as i64 + 1, r.id, r.group, r.expression, r.meaning_en, r.register_note, ex, entry])?;
        stats.writing += 1;
        stats.writing_linked += entry.is_some() as usize;
    }
    for (i, r) in confusables.iter().filter(|r| ships(&r.review)).enumerate() {
        let words: Vec<_> = r
            .words
            .iter()
            .map(|w| {
                let entry = if w.wrong { None } else { linker.confusable_word(&w.text) };
                if !w.wrong {
                    stats.words += 1;
                    stats.words_linked += entry.is_some() as usize;
                    if entry.is_none() {
                        stats.words_unresolved.push(format!("{} {}", r.id, w.text));
                    }
                }
                json!({"w": w.text, "wrong": w.wrong, "ko": w.ko, "en": w.en, "entry": entry})
            })
            .collect();
        conn.prepare_cached("INSERT INTO confusable VALUES (?1,?2,?3,?4,?5,?6,1)")?
            .execute(params![i as i64 + 1, r.id, r.kind, json!(words).to_string(), r.explanation_en, r.mistake_en])?;
        stats.confusables += 1;
    }
    conn.execute_batch("COMMIT")?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(id: &str, review: &str, link: &str) -> WritingRow {
        WritingRow {
            id: id.into(),
            group: "Conclusion".into(),
            expression: "결론적으로".into(),
            meaning_en: "in conclusion".into(),
            register_note: "Written, formal. The spoken 그래서 결국 is not used in an essay.".replace("그래서 결국", "“그래서 결국”"),
            ex: vec![("결론적으로 나는 찬성한다.".into(), "In conclusion, I agree.".into()), ("결론적으로 문제는 없다.".into(), "In conclusion, there is no problem.".into())],
            link: link.into(),
            review: review.into(),
        }
    }

    fn c(id: &str, review: &str, words: &[(&str, bool)]) -> ConfusableRow {
        ConfusableRow {
            id: id.into(),
            kind: "meaning".into(),
            words: words
                .iter()
                .map(|(t, wrong)| Word { text: (*t).into(), wrong: *wrong, ko: if *wrong { "-".into() } else { format!("{t}를 쓴다.") }, en: if *wrong { "-".into() } else { "He uses the word.".into() } })
                .collect(),
            explanation_en: "“다르다” means different and “틀리다” means wrong; they are mixed up in speech all the time.".into(),
            mistake_en: "Saying “생각이 틀리다” for a different opinion.".into(),
            review: review.into(),
        }
    }

    fn pack(dir: &Path) -> std::path::PathBuf {
        let db = dir.join("core.sqlite");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT, kind TEXT, source TEXT, rank INTEGER);
             CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL);
             INSERT INTO entries VALUES (1, '다르다', 'word', 'krdict', 5), (2, '다르다', 'word', 'kengdic', 1), (3, '틀리다', 'word', 'krdict', 7),
                                        (4, '-기 때문', 'grammar', 'krdict', 9), (5, '결론적', 'word', 'krdict', 3);
             INSERT INTO grammar VALUES (1, 4, '-기 때문'), (2, NULL, '-고자');",
        )
        .unwrap();
        db
    }

    #[test]
    fn only_approved_rows_ship_and_links_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let db = pack(dir.path());
        let writing = [w("W001", "approved", "g:-기 때문에"), w("W002", "draft", "w:다르다"), w("W003", "approved: ok", "w:없는말"), w("W004", "rejected: dup", ""), w("W005", "approved", "w:결론적")];
        let conf = [c("C001", "approved", &[("다르다", false), ("틀리다", false), ("×다른다", true)]), c("C002", "draft", &[("다르다", false), ("틀리다", false)]), c("C003", "approved", &[("다르다", false), ("모르는말", false)])];
        let st = add_tables_with(&db, &writing, &conf).unwrap();
        assert_eq!((st.writing, st.writing_linked), (3, 2));
        assert_eq!(st.writing_unresolved, vec!["W003 w:없는말"]);
        assert_eq!((st.confusables, st.words, st.words_linked), (2, 4, 3));
        assert_eq!(st.words_unresolved, vec!["C003 모르는말"]);
        let conn = Connection::open(&db).unwrap();
        // a grammar pattern resolves through its key (the closing 에 is dropped), a headword prefers krdict
        let e: i64 = conn.query_row("SELECT entry_id FROM writing_expr WHERE id = 'W001'", [], |r| r.get(0)).unwrap();
        assert_eq!(e, 4);
        let words: String = conn.query_row("SELECT words FROM confusable WHERE id = 'C001'", [], |r| r.get(0)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&words).unwrap();
        assert_eq!(v[0]["entry"], 1);
        assert_eq!(v[2]["wrong"], true);
        assert!(v[2]["entry"].is_null());
        let ai: i64 = conn.query_row("SELECT MIN(ai) FROM writing_expr", [], |r| r.get(0)).unwrap();
        assert_eq!(ai, 1);
        let ranks: Vec<i64> = conn.prepare("SELECT rank FROM writing_expr ORDER BY rank").unwrap().query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert_eq!(ranks, vec![1, 2, 3]);
    }

    #[test]
    fn a_draft_only_file_creates_empty_tables() {
        let dir = tempfile::tempdir().unwrap();
        let db = pack(dir.path());
        let st = add_tables_with(&db, &[w("W001", "draft", "")], &[c("C001", "draft", &[("다르다", false), ("틀리다", false)])]).unwrap();
        assert_eq!((st.writing, st.confusables), (0, 0));
        let n: i64 = Connection::open(&db).unwrap().query_row("SELECT COUNT(*) FROM confusable", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn bad_rows_fail_the_build_before_anything_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let db = pack(dir.path());
        let mut bad = w("W001", "approved", "");
        bad.ex[0].1 = "그래서 결국 찬성한다".into();
        assert!(add_tables_with(&db, &[bad], &[]).is_err());
        let n: i64 = Connection::open(&db).unwrap().query_row("SELECT COUNT(*) FROM sqlite_master WHERE name = 'writing_expr'", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn problems_are_found() {
        assert!(writing_problems(&w("W001", "draft", "g:-고자")).is_empty());
        let mut r = w("W002", "draft", "x:y");
        r.ex[0].0 = "나는 찬성해요.".into();
        r.meaning_en = "in 결론".into();
        r.register_note = "Spoken only.".into();
        assert_eq!(writing_problems(&r).len(), 5, "{:?}", writing_problems(&r));
        assert!(writing_problems(&w("W3", "draft", "")).iter().any(|p| p.contains("id must")));
        assert!(confusable_problems(&c("C001", "draft", &[("다르다", false), ("틀리다", false)])).is_empty());
        assert!(!confusable_problems(&c("C002", "draft", &[("다르다", false)])).is_empty());
        assert!(!confusable_problems(&c("C003", "draft", &[("×다른다", true), ("×틀린다", true)])).is_empty());
        let mut g = c("C004", "draft", &[("다르다", false), ("틀리다", false)]);
        g.mistake_en = "틀리다 is wrong, 다르다 is different.".into();
        assert!(!confusable_problems(&g).is_empty());
    }

    #[test]
    fn parsers_read_the_documented_columns() {
        let ok = parse_confusables(&format!("# c\n{}\nC001\tmeaning\t다르다 | ×다른다\tx\t문장이다. | -\tIt is. | -\ty\tdraft\n", CONFUSABLE_HEADER.join("\t"))).unwrap();
        assert_eq!(ok[0].words[1], Word { text: "다른다".into(), wrong: true, ko: "-".into(), en: "-".into() });
        assert!(parse_confusables(&format!("{}\nC001\tmeaning\ta | b\tx\t하나.\tOne.\ty\tdraft\n", CONFUSABLE_HEADER.join("\t"))).is_err(), "one sentence for two words");
        assert!(parse_writing("id\tgroup\n").is_err());
        assert!(parse_writing(&format!("{}\nW001\tg\n", WRITING_HEADER.join("\t"))).is_err());
    }

    // --- the real data files ---

    #[test]
    fn writing_expressions_file_is_well_formed() {
        let rows = builtin_writing();
        assert!((120..=170).contains(&rows.len()), "{} rows", rows.len());
        let p: Vec<String> = rows.iter().flat_map(writing_problems).collect();
        assert!(p.is_empty(), "{} problems, first: {:?}", p.len(), &p[..p.len().min(5)]);
        // each group is one consecutive block and the two sentences differ
        let mut seen = HashSet::new();
        let mut last = "";
        for r in &rows {
            if r.group != last {
                assert!(seen.insert(r.group.clone()), "group {:?} is split", r.group);
                last = &r.group;
            }
            assert_ne!(r.ex[0].0, r.ex[1].0, "{}: two identical sentences", r.id);
        }
        assert!(seen.len() >= 10, "{} groups", seen.len());
        // the graph and essay material the owner asked for is there
        for must in ["증가하다", "차지하다", "에 달하다", "반면", "그럼에도 불구하고", "뿐만 아니라", "에 따르면", "결론적으로", "요컨대", "예를 들어", "따라서", "그 결과"] {
            assert!(rows.iter().any(|r| r.expression.contains(must)), "no row for {must}");
        }
    }

    #[test]
    fn confusables_file_is_well_formed() {
        let rows = builtin_confusables();
        assert!((80..=120).contains(&rows.len()), "{} groups", rows.len());
        let p: Vec<String> = rows.iter().flat_map(confusable_problems).collect();
        assert!(p.is_empty(), "{} problems, first: {:?}", p.len(), &p[..p.len().min(5)]);
        let all: Vec<&str> = rows.iter().flat_map(|r| r.words.iter().map(|w| w.text.as_str())).collect();
        for must in ["바라다", "원하다", "다르다", "틀리다", "맞추다", "맞히다", "가르치다", "가리키다", "늘이다", "늘리다", "반드시", "반듯이", "결재", "결제", "지양", "지향", "일체", "일절", "체제", "체계", "계기", "기회", "경험", "체험", "비판", "비난", "효과", "효율", "보전", "보존", "실재", "실제", "부치다", "붙이다", "이따가", "있다가", "그러므로"] {
            assert!(all.contains(&must), "{must} is in no group");
        }
        // the set of correct words is not repeated across groups except where a word really belongs to two contrasts
        let mut count: HashMap<&str, usize> = HashMap::new();
        for r in &rows {
            for w in r.words.iter().filter(|w| !w.wrong) {
                *count.entry(w.text.as_str()).or_default() += 1;
            }
        }
        let repeated: Vec<_> = count.iter().filter(|(_, n)| **n > 2).collect();
        assert!(repeated.is_empty(), "{repeated:?}");
    }

    #[test]
    fn every_approved_row_has_a_review_note_in_the_sheet() {
        // the reviewer's sheet lists every AI field: it names every id of the data files
        let sheet = include_str!("../qa/writing_confusables_review.md");
        for id in builtin_writing().iter().map(|r| &r.id).chain(builtin_confusables().iter().map(|r| &r.id)) {
            assert!(sheet.contains(id.as_str()), "{id} is missing from qa/writing_confusables_review.md");
        }
    }
}
