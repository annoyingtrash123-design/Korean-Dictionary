use super::*;
use crate::hangul::norm_headword;

const SCHEMA: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER,
  hanja TEXT, pos TEXT, pron TEXT, source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER,
  rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);
CREATE INDEX entries_hw ON entries(hw_norm);
CREATE INDEX entries_rank ON entries(rank);
CREATE TABLE forms (form TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX forms_form ON forms(form);
CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX hanja_words_ch ON hanja_words(ch);
"#;

const CORE_ONLY: &str = r#"
CREATE VIRTUAL TABLE entries_fts USING fts5(en, content='', tokenize='porter unicode61');
CREATE TABLE hanja_chars (ch TEXT PRIMARY KEY, readings TEXT, meaning_en TEXT, strokes INTEGER, radical TEXT, word_count INTEGER);
CREATE TABLE sentences (id INTEGER PRIMARY KEY, ko TEXT NOT NULL, en TEXT, source TEXT);
CREATE VIRTUAL TABLE sentences_fts USING fts5(ko, content='sentences', content_rowid='id', tokenize='trigram');
CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL, category TEXT NOT NULL, level INTEGER, summary_en TEXT, sort INTEGER);
"#;

struct E {
    id: i64,
    hw: &'static str,
    hanja: Option<&'static str>,
    pos: &'static str,
    source: &'static str,
    level: Option<i64>,
    rank: i64,
    kind: &'static str,
    gloss: &'static str,
    forms: &'static [&'static str],
}

const fn e(
    id: i64,
    hw: &'static str,
    hanja: Option<&'static str>,
    pos: &'static str,
    source: &'static str,
    level: Option<i64>,
    rank: i64,
    gloss: &'static str,
    forms: &'static [&'static str],
) -> E {
    E { id, hw, hanja, pos, source, level, rank, kind: "word", gloss, forms }
}

const CORE: &[E] = &[
    e(1, "학교", Some("學校"), "noun", "krdict", Some(1), 100, "school", &["학교가", "학교는"]),
    e(2, "학생", Some("學生"), "noun", "krdict", Some(1), 110, "student", &[]),
    e(3, "가다", None, "verb", "krdict", Some(1), 50, "to go", &["갔다", "가요"]),
    e(4, "먹다", None, "verb", "krdict", Some(1), 60, "to eat", &["먹어요"]),
    e(5, "춥다", None, "adjective", "krdict", Some(1), 120, "cold (weather)", &[]),
    e(6, "듣다", None, "verb", "krdict", Some(1), 130, "to listen; to hear", &[]),
    e(7, "들다", None, "verb", "krdict", Some(2), 400, "to hold; to enter", &[]),
    e(8, "돕다", None, "verb", "krdict", Some(2), 410, "to help", &[]),
    e(9, "모르다", None, "verb", "krdict", Some(1), 140, "not to know", &[]),
    e(10, "공부하다", None, "verb", "krdict", Some(1), 150, "to study", &[]),
    e(11, "공부", None, "noun", "krdict", Some(1), 70, "study; studies", &[]),
    e(12, "하다", None, "verb", "krdict", Some(1), 10, "to do", &[]),
    e(13, "빨갛다", None, "adjective", "krdict", Some(1), 300, "red", &[]),
    e(14, "살다", None, "verb", "krdict", Some(1), 160, "to live", &[]),
    e(15, "학교", Some("學校"), "noun", "wikt", None, 2000, "school (an institution)", &[]),
    e(16, "학교", Some("學校"), "noun", "kengdic", None, 3000, "school", &[]),
    e(17, "대학교", Some("大學校"), "noun", "krdict", Some(1), 105, "university", &[]),
    e(18, "-아서/어서", None, "ending", "krdict", Some(1), 20, "because; and then", &[]),
    e(19, "학년", Some("學年"), "noun", "krdict", Some(2), 500, "school year; grade", &[]),
    e(20, "사랑", None, "noun", "krdict", Some(1), 80, "love", &[]),
    e(21, "학", None, "noun", "krdict", Some(3), 900, "crane (bird)", &[]),
    e(22, "선생님", None, "noun", "krdict", Some(1), 90, "teacher", &[]),
];

const STDICT: &[E] = &[
    e(1, "학교", Some("學校"), "noun", "stdict", None, 9000, "", &["학교의"]),
    e(2, "가다", None, "verb", "stdict", None, 9010, "", &["갔다"]),
    e(3, "학문", Some("學問"), "noun", "stdict", None, 9020, "", &[]),
];

fn insert(conn: &Conn, pack: &str, items: &[E]) {
    for it in items {
        let data = serde_json::json!({
            "senses": [{ "gloss": it.gloss, "def": format!("definition of {}", it.gloss), "examples": [{"ko": "예문", "en": "example"}] }],
            "related": [{"type": "derived", "word": "파생"}],
        })
        .to_string();
        conn.query(
            "INSERT INTO entries (id, headword, hw_norm, homonym, hanja, pos, pron, source, lang, level, rank, kind, gloss, data) \
             VALUES (?1,?2,?3,NULL,?4,?5,NULL,?6,?7,?8,?9,?10,?11,?12)",
            &[
                it.id.into(),
                it.hw.into(),
                norm_headword(it.hw).into(),
                it.hanja.into(),
                it.pos.into(),
                it.source.into(),
                (if pack == "stdict" { "ko" } else { "en" }).into(),
                it.level.map_or(Val::Null, Val::Int),
                it.rank.into(),
                it.kind.into(),
                (if it.gloss.is_empty() { Val::Null } else { it.gloss.into() }),
                data.into(),
            ],
        )
        .unwrap();
        for f in it.forms {
            conn.query("INSERT INTO forms VALUES (?1, ?2)", &[(*f).into(), it.id.into()]).unwrap();
        }
        for ch in it.hanja.unwrap_or("").chars() {
            conn.query("INSERT INTO hanja_words VALUES (?1, ?2)", &[ch.to_string().into(), it.id.into()]).unwrap();
        }
    }
}

fn build_core() -> PackDb {
    let c = Conn::open_memory().unwrap();
    c.exec(SCHEMA).unwrap();
    c.exec(CORE_ONLY).unwrap();
    c.exec("INSERT INTO meta VALUES ('pack','core'), ('version','2025.01.test')").unwrap();
    insert(&c, "core", CORE);
    // contentless FTS rows (rowid = entries.id)
    for it in CORE {
        if it.source == "krdict" || it.id == 15 {
            c.query(
                "INSERT INTO entries_fts(rowid, en) VALUES (?1, ?2)",
                &[it.id.into(), format!("{} definition of {}", it.gloss, it.gloss).into()],
            )
            .unwrap();
        }
    }
    for (ch, rd, mean, st, rad, wc) in [
        ("學", "학", "to study; learning", 16, "子", 6),
        ("校", "교", "school", 10, "木", 3),
        ("生", "생", "to live; to be born", 5, "生", 1),
    ] {
        c.query(
            "INSERT INTO hanja_chars VALUES (?1,?2,?3,?4,?5,?6)",
            &[ch.into(), rd.into(), mean.into(), (st as i64).into(), rad.into(), (wc as i64).into()],
        )
        .unwrap();
    }
    for (i, (ko, en)) in [
        ("저는 학교에 갑니다.", "I go to school."),
        ("학교는 아홉 시에 시작해요.", "School starts at nine."),
        ("나는 학생이에요.", "I am a student."),
        ("학교", "school"),
        ("사랑은 아름다운 것이다.", "Love is a beautiful thing."),
    ]
    .iter()
    .enumerate()
    {
        c.query("INSERT INTO sentences VALUES (?1,?2,?3,'tatoeba')", &[(i as i64 + 1).into(), (*ko).into(), (*en).into()])
            .unwrap();
    }
    c.exec("INSERT INTO sentences_fts(sentences_fts) VALUES('rebuild')").unwrap();
    c.exec(
        "INSERT INTO grammar VALUES (1, 18, '-아서/어서', 'Connective endings', 1, 'because', 2), \
         (2, NULL, '이/가', 'Particles', 1, 'subject', 1), (3, NULL, '-았/었-', 'Pre-final endings', 1, 'past', 3)",
    )
    .unwrap();
    PackDb::new("core", c).unwrap()
}

fn build_stdict() -> PackDb {
    let c = Conn::open_memory().unwrap();
    c.exec(SCHEMA).unwrap();
    c.exec("INSERT INTO meta VALUES ('pack','stdict')").unwrap();
    insert(&c, "stdict", STDICT);
    PackDb::new("stdict", c).unwrap()
}

fn heads(r: &SearchResult) -> Vec<String> {
    r.rows.iter().map(|r| format!("{}:{}", r.source, r.headword)).collect()
}

#[test]
fn caps_detected() {
    let core = build_core();
    assert!(core.caps.entries_fts && core.caps.hanja_chars && core.caps.sentences_fts && core.caps.grammar);
    let st = build_stdict();
    assert!(st.caps.forms && !st.caps.entries_fts && !st.caps.grammar && !st.caps.sentences);
    assert_eq!(core.meta("version").as_deref(), Some("2025.01.test"));
    assert!(PackDb::new("x", Conn::open_memory().unwrap()).is_err());
}

#[test]
fn hangul_exact_then_prefix_with_via() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "학교", None).unwrap();
    assert_eq!(r.mode, "hangul");
    // exact matches first: krdict (rank 100), wikt, kengdic, stdict by rank
    assert_eq!(
        heads(&r)[..4],
        ["krdict:학교", "wikt:학교", "kengdic:학교", "stdict:학교"].map(String::from)
    );
    assert!(r.rows[..4].iter().all(|x| x.via == Some("exact")));
    // prefix rows afterwards; none repeated
    let prefix: Vec<_> = r.rows.iter().filter(|x| x.via == Some("prefix")).collect();
    assert!(prefix.is_empty() || prefix.iter().all(|x| x.hw_norm.starts_with("학교")));
    let mut ids: Vec<_> = r.rows.iter().map(|x| (x.pack.clone(), x.id)).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n);
}

#[test]
fn hangul_prefix_ordered_by_rank_and_limited() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "학", None).unwrap();
    let hw: Vec<_> = r.rows.iter().map(|x| x.headword.as_str()).collect();
    // exact '학' (crane) first, then prefix by rank: 학교(100) 학생(110) 학년(500) ... 학문(stdict)
    assert_eq!(hw[0], "학");
    assert_eq!(r.rows[0].via, Some("exact"));
    let pre: Vec<_> = r.rows.iter().filter(|x| x.via == Some("prefix")).map(|x| x.headword.clone()).collect();
    assert_eq!(&pre[..3], ["학교", "학생", "학년"]);
    assert!(pre.contains(&"학문".to_string()));
    assert!(r.rows.windows(2).filter(|w| w[0].via == Some("prefix") && w[1].via == Some("prefix")).all(|w| w[0].rank <= w[1].rank));
    let r = search(&[&core, &st], "학", Some(2)).unwrap();
    assert_eq!(r.rows.len(), 2);
}

#[test]
fn hangul_prefix_does_not_cross_into_other_syllables() {
    let core = build_core();
    let r = search(&[&core], "사", None).unwrap();
    let hw: Vec<_> = r.rows.iter().map(|x| x.headword.clone()).collect();
    assert!(hw.contains(&"사랑".to_string()));
    assert!(!hw.contains(&"선생님".to_string()));
}

#[test]
fn hangul_forms_match() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "갔다", None).unwrap();
    let forms: Vec<_> = r.rows.iter().filter(|x| x.via == Some("form")).map(|x| format!("{}:{}", x.source, x.headword)).collect();
    assert_eq!(forms, ["krdict:가다", "stdict:가다"]);
    // the first row is the form match; deconjugation does not duplicate it
    assert_eq!(r.rows[0].via, Some("form"));
    assert_eq!(r.rows.iter().filter(|x| x.source == "krdict" && x.headword == "가다").count(), 1);
}

#[test]
fn hangul_deconjugation_filtered_by_existence() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "갔어요", None).unwrap();
    assert_eq!(r.rows[0].headword, "가다");
    assert_eq!(r.rows[0].via, Some("deconj"));
    assert_eq!(r.rows[0].source, "krdict");
    let d = r.deconj.as_ref().unwrap();
    assert!(d.iter().all(|x| ["가다"].contains(&x.lemma.as_str()) || r.rows.iter().any(|y| y.hw_norm == x.lemma)));
    assert_eq!(d[0].lemma, "가다");
    assert_eq!(d[0].rule, "past -았/었- + polite -아요/어요");
    assert!(r.grammar_hints.as_ref().unwrap().contains(&"-았/었-".to_string()));
    // candidates that are not in the database never appear
    assert!(!d.iter().any(|x| x.lemma == "그다"));
}

#[test]
fn hangul_irregular_lookups() {
    let (core, st) = (build_core(), build_stdict());
    for (q, lemma) in [
        ("추워요", "춥다"),
        ("들어요", "듣다"),
        ("들어요", "들다"),
        ("도와요", "돕다"),
        ("몰라요", "모르다"),
        ("빨개요", "빨갛다"),
        ("사는", "살다"),
        ("공부했어요", "공부하다"),
        ("공부했어요", "공부"),
        ("학교에서", "학교"),
        ("먹었어요", "먹다"),
    ] {
        let r = search(&[&core, &st], q, None).unwrap();
        assert!(
            r.rows.iter().any(|x| x.headword == lemma && x.via == Some("deconj")),
            "{q} should find {lemma}: {:?}",
            heads(&r)
        );
    }
}

#[test]
fn hangul_deconj_order_follows_candidate_rank() {
    let core = build_core();
    let r = search(&[&core], "들어요", None).unwrap();
    let d: Vec<_> = r.rows.iter().filter(|x| x.via == Some("deconj")).map(|x| x.headword.clone()).collect();
    assert_eq!(d, ["들다", "듣다"]);
}

#[test]
fn hangul_query_normalisation() {
    let core = build_core();
    let r = search(&[&core], "  -아서/어서 ", None).unwrap();
    assert_eq!(r.rows[0].headword, "-아서/어서");
    let r = search(&[&core], "학 교", None).unwrap();
    assert_eq!(r.rows[0].headword, "학교");
}

#[test]
fn pack_selection_is_respected() {
    let (core, st) = (build_core(), build_stdict());
    let only_core = search(&[&core], "학교", None).unwrap();
    assert!(only_core.rows.iter().all(|r| r.pack == "core"));
    let only_st = search(&[&st], "학교", None).unwrap();
    assert!(only_st.rows.iter().all(|r| r.pack == "stdict" && r.source == "stdict"));
    let none = search(&[], "학교", None).unwrap();
    assert!(none.rows.is_empty());
}

#[test]
fn english_fts_prefix_and_ranking() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "school", None).unwrap();
    assert_eq!(r.mode, "english");
    assert!(r.rows.iter().all(|x| x.via == Some("fts") && x.pack == "core"));
    let hw: Vec<_> = r.rows.iter().map(|x| x.headword.clone()).collect();
    assert!(hw.contains(&"학교".to_string()));
    assert!(hw.contains(&"학년".to_string()));
    // prefix on the last token
    let r = search(&[&core], "scho", None).unwrap();
    assert!(r.rows.iter().any(|x| x.headword == "학교"));
    // multiple tokens: AND, last token prefix
    let r = search(&[&core], "to ea", None).unwrap();
    assert!(r.rows.iter().any(|x| x.headword == "먹다"), "{:?}", heads(&r));
    // porter stemming
    let r = search(&[&core], "studies", None).unwrap();
    assert!(r.rows.iter().any(|x| x.headword == "공부"), "{:?}", heads(&r));
    // limit
    assert_eq!(search(&[&core], "school", Some(1)).unwrap().rows.len(), 1);
}

#[test]
fn english_query_escaping() {
    let core = build_core();
    for q in ["school\"", "\"", "sch\"ool", "a*b", "to (eat)", "-", "NEAR(", "AND", "學school?", "school OR"] {
        let r = search(&[&core], q, None);
        assert!(r.is_ok(), "query {q:?} errored: {:?}", r.err());
    }
    assert!(search(&[&core], "!!!", None).unwrap().rows.is_empty());
    assert_eq!(fts_query("it's \"quoted\" text").unwrap(), "\"it\" \"s\" \"quoted\" \"text\"*");
    assert_eq!(fts_query("Hello").unwrap(), "\"hello\"*");
    assert_eq!(fts_query("???"), None);
}

#[test]
fn english_empty_and_stdict_only() {
    let (core, st) = (build_core(), build_stdict());
    assert!(search(&[&core], "   ", None).unwrap().rows.is_empty());
    // stdict has no FTS: english search simply yields nothing for it
    assert!(search(&[&st], "school", None).unwrap().rows.is_empty());
}

#[test]
fn hanja_single_char() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "學", None).unwrap();
    assert_eq!(r.mode, "hanja");
    let cards = r.hanja.as_ref().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].ch, "學");
    assert_eq!(cards[0].readings.as_deref(), Some("학"));
    assert_eq!(cards[0].strokes, Some(16));
    let hw: Vec<_> = r.rows.iter().map(|x| x.headword.clone()).collect();
    for w in ["학교", "학생", "학년", "학문", "대학교"] {
        assert!(hw.contains(&w.to_string()), "{w} missing from {hw:?}");
    }
    assert!(r.rows.iter().all(|x| x.via == Some("hanja")));
    assert!(r.rows.windows(2).all(|w| w[0].rank <= w[1].rank));
}

#[test]
fn hanja_multi_char() {
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st], "學校", None).unwrap();
    assert_eq!(r.hanja.as_ref().unwrap().len(), 2);
    assert_eq!(r.rows[0].headword, "학교");
    // exact hanja first, then words that merely contain it (大學校)
    let hw: Vec<_> = r.rows.iter().map(|x| x.headword.clone()).collect();
    assert!(hw.contains(&"대학교".to_string()));
    assert!(hw.iter().position(|h| h == "학교").unwrap() < hw.iter().position(|h| h == "대학교").unwrap());
    // char with no card
    let r = search(&[&core], "學問", None).unwrap();
    assert_eq!(r.hanja.as_ref().unwrap().len(), 1);
}

#[test]
fn entries_and_entry_lookup() {
    let (core, st) = (build_core(), build_stdict());
    let v = entries_by_headword(&[&core, &st], "학교").unwrap();
    assert_eq!(v.len(), 4);
    assert_eq!(v[0].source, "krdict");
    assert_eq!(v[0].data["senses"][0]["gloss"], "school");
    assert_eq!(v[0].data["related"][0]["word"], "파생");
    assert_eq!(v[0].hanja.as_deref(), Some("學校"));
    assert!(entries_by_headword(&[&core], "없는말").unwrap().is_empty());
    assert!(entries_by_headword(&[&core], "").unwrap().is_empty());
    assert_eq!(entries_by_headword(&[&core], "-아서/어서").unwrap().len(), 1);

    let e = entry(&[&core, &st], "stdict", 1).unwrap().unwrap();
    assert_eq!(e.headword, "학교");
    assert_eq!(e.lang, "ko");
    assert!(entry(&[&core, &st], "krdict", 9999).unwrap().is_none());
    assert!(entry(&[&core], "stdict", 1).unwrap().is_none());
}

#[test]
fn hanja_char_and_words_paging() {
    let (core, st) = (build_core(), build_stdict());
    let h = hanja_char(&[&core, &st], "校").unwrap().unwrap();
    assert_eq!(h.meaning_en.as_deref(), Some("school"));
    assert_eq!(h.radical.as_deref(), Some("木"));
    assert!(hanja_char(&[&core], "鬱").unwrap().is_none());
    assert!(hanja_char(&[&st], "校").unwrap().is_none());

    let all = words_with_hanja(&[&core, &st], "學", 50, 0).unwrap();
    assert!(all.len() >= 6);
    let page1 = words_with_hanja(&[&core, &st], "學", 3, 0).unwrap();
    let page2 = words_with_hanja(&[&core, &st], "學", 3, 3).unwrap();
    assert_eq!(page1.len(), 3);
    let mut joined: Vec<_> = page1.iter().chain(page2.iter()).map(|r| (r.pack.clone(), r.id)).collect();
    let expect: Vec<_> = all.iter().take(joined.len()).map(|r| (r.pack.clone(), r.id)).collect();
    assert_eq!(joined.len(), expect.len());
    joined.sort();
    let mut e2 = expect.clone();
    e2.sort();
    assert_eq!(joined, e2);
    assert!(words_with_hanja(&[&core], "學", 3, 1000).unwrap().is_empty());
}

#[test]
fn sentences_trigram_and_like_fallback() {
    let (core, st) = (build_core(), build_stdict());
    // >= 3 chars: FTS5 trigram
    let s = sentences(&[&core, &st], "학교에", 10).unwrap();
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].ko, "저는 학교에 갑니다.");
    assert_eq!(s[0].en.as_deref(), Some("I go to school."));
    assert_eq!(s[0].source, "tatoeba");
    // 2 chars: LIKE fallback, shortest first
    let s = sentences(&[&core], "학교", 10).unwrap();
    assert_eq!(s.len(), 3);
    assert_eq!(s[0].ko, "학교");
    assert_eq!(sentences(&[&core], "학교", 2).unwrap().len(), 2);
    // LIKE wildcards are literal
    assert!(sentences(&[&core], "%", 5).unwrap().is_empty());
    assert!(sentences(&[&core], "_", 5).unwrap().is_empty());
    assert!(sentences(&[&core], "", 5).unwrap().is_empty());
    // quote in trigram query is safe
    assert!(sentences(&[&core], "학\"교에", 5).is_ok());
    // stdict has no sentences
    assert!(sentences(&[&st], "학교에", 5).unwrap().is_empty());
}

#[test]
fn grammar_list_sorted() {
    let core = build_core();
    let g = grammar_list(&[&core]).unwrap();
    let pats: Vec<_> = g.iter().map(|x| x.pattern.as_str()).collect();
    assert_eq!(pats, ["이/가", "-아서/어서", "-았/었-"]);
    assert_eq!(g[1].entry_id, Some(18));
    assert_eq!(g[0].entry_id, None);
    assert_eq!(g[1].category, "Connective endings");
    assert!(grammar_list(&[&build_stdict()]).unwrap().is_empty());
}

#[test]
fn word_of_day_is_deterministic_and_eligible() {
    let core = build_core();
    let a = word_of_day(&[&core], "2025-03-14").unwrap().unwrap();
    let b = word_of_day(&[&core], "2025-03-14").unwrap().unwrap();
    assert_eq!(a, b);
    assert_eq!(a.source, "krdict");
    assert!(matches!(a.level, Some(1) | Some(2)));
    assert_eq!(a.kind, "word");
    let mut seen = HashSet::new();
    for d in 1..=28 {
        let w = word_of_day(&[&core], &format!("2025-02-{d:02}")).unwrap().unwrap();
        assert!(matches!(w.level, Some(1) | Some(2)));
        seen.insert(w.id);
    }
    assert!(seen.len() >= 5, "poor spread: {seen:?}");
    assert!(word_of_day(&[&build_stdict()], "2025-01-01").unwrap().is_none());
    // garbage dates still give something stable
    assert_eq!(word_of_day(&[&core], "garbage").unwrap(), word_of_day(&[&core], "garbage").unwrap());
}

#[test]
fn date_parsing() {
    assert_eq!(days_from_date("1970-01-01"), Some(0));
    assert_eq!(days_from_date("1970-01-02"), Some(1));
    assert_eq!(days_from_date("2000-03-01"), Some(11017));
    assert_eq!(days_from_date("2025-03-14T10:00:00Z"), Some(20161));
    assert_eq!(days_from_date("nope"), None);
    assert_eq!(days_from_date("2025-13-01"), None);
}

#[test]
fn script_routing() {
    let core = build_core();
    assert_eq!(search(&[&core], "가", None).unwrap().mode, "hangul");
    assert_eq!(search(&[&core], "ㄱ", None).unwrap().mode, "hangul");
    assert_eq!(search(&[&core], "學", None).unwrap().mode, "hanja");
    assert_eq!(search(&[&core], "love", None).unwrap().mode, "english");
    assert_eq!(search(&[&core], "學생", None).unwrap().mode, "hangul");
}

// ---- optional tests against real data (skipped when absent) ---------------------------------

fn open_real(path: &std::path::Path, id: &str) -> Option<PackDb> {
    if !path.exists() {
        return None;
    }
    Some(PackDb::new(id, Conn::open_path(path.to_str()?, true).ok()?).ok()?)
}

#[test]
fn pipeline_output_if_present() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../pipeline/out/core.sqlite");
    let Some(core) = open_real(&p, "core") else {
        eprintln!("skipping: {} not found", p.display());
        return;
    };
    let r = search(&[&core], "학교", None).unwrap();
    assert!(!r.rows.is_empty(), "pipeline core.sqlite has no 학교");
    assert!(search(&[&core], "school", None).unwrap().rows.len() > 0);
    let r = search(&[&core], "갔어요", None).unwrap();
    assert!(r.rows.iter().any(|x| x.headword == "가다"));
    assert!(!grammar_list(&[&core]).unwrap().is_empty());
    assert!(word_of_day(&[&core], "2025-06-01").unwrap().is_some());
}

/// The app's committed fixture packs (gzip chunks + manifest) built by app/scripts/make-fixture.py.
#[test]
fn app_fixture_if_present() {
    use std::io::Read;
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/public/data");
    let Ok(manifest) = std::fs::read_to_string(dir.join("manifest.json")) else {
        eprintln!("skipping: no app fixture");
        return;
    };
    let m: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    let tmp = std::env::temp_dir().join(format!("kdict-core-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let mut dbs = Vec::new();
    for pack in m["packs"].as_array().unwrap() {
        let id = pack["id"].as_str().unwrap();
        let mut gz = Vec::new();
        for c in pack["chunks"].as_array().unwrap() {
            let name = c.as_str().or_else(|| c["file"].as_str()).unwrap();
            gz.extend(std::fs::read(dir.join(name)).unwrap());
        }
        let mut raw = Vec::new();
        flate2::read::GzDecoder::new(&gz[..]).read_to_end(&mut raw).unwrap();
        assert_eq!(raw.len() as u64, pack["bytes"].as_u64().unwrap());
        let path = tmp.join(format!("{id}.sqlite"));
        std::fs::write(&path, raw).unwrap();
        dbs.push(open_real(&path, id).unwrap());
    }
    let refs: Vec<&PackDb> = dbs.iter().collect();
    let r = search(&refs, "학교", None).unwrap();
    assert_eq!(r.rows[0].headword, "학교");
    assert!(search(&refs, "갔어요", None).unwrap().rows.iter().any(|x| x.headword == "가다"));
    assert!(search(&refs, "추워요", None).unwrap().rows.iter().any(|x| x.headword == "춥다"));
    assert!(search(&refs, "school", None).unwrap().rows.iter().any(|x| x.headword == "학교"));
    assert!(search(&refs, "學", None).unwrap().hanja.is_some());
    // (the committed fixture may have been generated without sentences)
    let n = dbs[0].conn.query("SELECT count(*) FROM sentences", &[]).unwrap()[0].int(0).unwrap();
    if n > 0 {
        assert!(!sentences(&refs, "학교에", 5).unwrap().is_empty());
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn any_number_of_packs_are_merged_by_rank() {
    const OPEN: &[E] = &[e(1, "학교", Some("學校"), "noun", "opendict", None, 9500, "", &["학교의"]), e(2, "학당", Some("學堂"), "noun", "opendict", None, 9600, "", &[])];
    let c = Conn::open_memory().unwrap();
    c.exec(SCHEMA).unwrap();
    insert(&c, "opendict", OPEN);
    let open = PackDb::new("opendict", c).unwrap();
    let (core, st) = (build_core(), build_stdict());
    let r = search(&[&core, &st, &open], "학교", None).unwrap();
    let ex: Vec<_> = r.rows.iter().filter(|x| x.via == Some("exact")).map(|x| x.source.clone()).collect();
    assert_eq!(ex, ["krdict", "wikt", "kengdic", "stdict", "opendict"]);
    let h = search(&[&core, &st, &open], "學", None).unwrap();
    assert!(h.rows.iter().any(|x| x.source == "opendict" && x.headword == "학당"));
    assert!(search(&[&core, &st], "학당", None).unwrap().rows.iter().all(|x| x.source != "opendict"));
}
