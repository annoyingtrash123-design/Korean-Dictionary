//! Quality gate of the grammar mini-lessons (`data/grammar_lessons/<level>.json`): schema, level, that
//! every lesson belongs to a curriculum row, content shape, and that only approved lessons ship.

use kdict_pipeline::curriculum::{builtin_grammar, grammar_key};
use kdict_pipeline::grammar_lessons::*;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

#[test]
fn lesson_files_pass_the_quality_gate() {
    let lessons = builtin();
    let problems = check_all(&lessons, &builtin_grammar());
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
    for (level, json) in FILES {
        for l in parse(json, level).unwrap() {
            assert_eq!(l.level, level);
            assert!(!l.review.status.is_empty(), "{}", l.form);
        }
    }
}

/// The pilot batch: every 6급 pattern of the curriculum list has a lesson.
#[test]
fn level_6_pilot_covers_every_pattern() {
    let rows: Vec<_> = builtin_grammar().into_iter().filter(|g| g.level == 6).collect();
    assert_eq!(rows.len(), 56);
    let lessons = builtin();
    let have: HashSet<&str> = lessons.iter().filter(|l| l.level == 6).map(|l| l.form.as_str()).collect();
    for g in &rows {
        assert!(have.contains(lesson_key(&g.form_raw)), "no lesson for {}", g.form_raw);
    }
    assert_eq!(have.len(), 56);
}

/// Keys are unique per level: `-게3` / `-게4` are told apart by their homonym digit.
#[test]
fn lesson_keys_are_unique_per_level() {
    let mut seen = HashSet::new();
    for g in builtin_grammar() {
        assert!(seen.insert((g.level, lesson_key(&g.form_raw).to_string())), "{} twice at {}급", g.form_raw, g.level);
    }
}

fn sample() -> Lesson {
    parse(
        r#"[{"form":"-노라면","level":6,"label":"if you keep doing","meaning_en":"“If you keep doing X, in time Y happens”; an old-fashioned, literary ending.",
        "usage":"Attaches to a verb stem; the clause that follows states what you will find or what will happen.",
        "examples":[
          {"ko":"열심히 살다 보면 좋은 날도 오겠지, 열심히 사노라면.","en":"Good days will come if you keep living hard.","ctx":"conversation"},
          {"ko":"앞을 향해 계속 가노라면 붉은 기와집이 나타날 것이다.","en":"If you keep going ahead, a red-tiled house will appear.","ctx":"collocation"},
          {"ko":"세월이 흐르노라면 상처도 아문다.","en":"As time passes, wounds heal too.","ctx":"essay"},
          {"ko":"글을 쓰노라면 시간 가는 줄 모른다.","en":"When I write, I lose track of time.","ctx":"essay"}],
        "compare":[{"pattern":"-다 보면","difference":"The everyday equivalent.","pair":[{"ko":"사노라면 알게 된다.","en":"You will come to know."},{"ko":"살다 보면 알게 된다.","en":"You will come to know."}]}],
        "pitfalls":[{"note":"It cannot take a past tense.","wrong":"살았노라면 알게 된다.","right":"사노라면 알게 된다.","en":"You will come to know."}],
        "review":{"status":"draft"}}]"#,
        6,
    )
    .unwrap()
    .remove(0)
}

#[test]
fn validator_flags_what_it_should() {
    let rows = builtin_grammar();
    let row = rows.iter().find(|g| g.level == 6 && g.form_raw == "-노라면").unwrap();
    let ok = sample();
    assert_eq!(problems(&ok, row), Vec::<String>::new());

    let mut l = ok.clone();
    l.meaning_en = "Means 하다가 보면 in old style, quite a lot of words here.".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("unquoted Hangul")));
    l.meaning_en = "Means “하다가 보면” in old style, quite a lot of words here.".into();
    assert!(problems(&l, row).is_empty(), "quoted forms are fine");

    let mut l = ok.clone();
    l.examples.truncate(3);
    assert!(problems(&l, row).iter().any(|p| p.contains("expected 4-6")));
    let mut l = ok.clone();
    l.examples[1].en = "붉은 기와집".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("example 2 en")));
    let mut l = ok.clone();
    l.examples[0].ko = "열심히 살다 보면 좋은 날도 오겠지.".into();
    l.examples[1].ko = "계속 가다 보면 집이 나타날 것이다.".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("contain the pattern")));
    let mut l = ok.clone();
    for e in &mut l.examples {
        e.ctx = Some("essay".into());
    }
    let pr = problems(&l, row);
    assert!(pr.iter().any(|p| p.contains("collocation")) && pr.iter().any(|p| p.contains("two different contexts")));
    let mut l = ok.clone();
    l.review.status = "approved".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("reviewer")));
    l.review.status = "maybe".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("review.status")));
    let mut l = ok;
    l.compare[0].pattern = "노라면".into();
    assert!(problems(&l, row).iter().any(|p| p.contains("itself")));
}

#[test]
fn unknown_fields_and_wrong_levels_are_rejected() {
    assert!(parse(r#"[{"form":"x","level":6,"surprise":1}]"#, 6).is_err());
    let mut l = sample();
    l.level = 5;
    let json = serde_json::to_string(&vec![l]).unwrap();
    assert!(parse(&json, 6).is_err());
    assert!(parse(&json, 5).is_ok());
}

#[test]
fn check_all_matches_forms_to_rows() {
    let rows = builtin_grammar();
    let mut a = sample();
    a.form = "-없는형태".into();
    let pr = check_all(&[a], &rows);
    assert!(pr.iter().any(|p| p.contains("no such curriculum row")), "{pr:?}");
    // the same form at another level is not the row
    let mut b = sample();
    b.level = 5;
    assert!(check_all(&[b], &rows).iter().any(|p| p.contains("no such curriculum row")));
    let mut twice = vec![sample(), sample()];
    twice[1].examples.clear();
    assert!(check_all(&twice, &rows).iter().any(|p| p.contains("two lessons")));
}

#[test]
fn quotes_hide_korean_from_the_english_check() {
    assert_eq!(without_quotes("use “-는다” or ‘-다’ and \"-고\" here"), "use  or  and  here");
    assert!(!has_hangul(&without_quotes("plain English, “한글”.")));
    assert!(has_hangul(&without_quotes("plain 한글 English")));
}

/// Unquoted Hangul is a problem in English fields, except the TOPIK section names and grammar jamo.
#[test]
fn english_fields_allow_exam_sections_and_jamo_terms() {
    let rows = builtin_grammar();
    let row = rows.iter().find(|g| g.level == 6 && g.form_raw == "-노라면").unwrap();
    let mut l = sample();
    l.topik_tip = Some("Common in 읽기 and 쓰기 (51–54), and with ㄹ-stems like “사노라면”.".into());
    assert!(problems(&l, row).is_empty(), "{:?}", problems(&l, row));
    l.topik_tip = Some("Common with 노라면 here.".into());
    assert!(problems(&l, row).iter().any(|p| p.contains("topik_tip has unquoted Hangul")));
}

#[test]
fn body_drops_review_and_links_compare_patterns() {
    let mut l = sample();
    l.review = Review { status: "approved".into(), reviewer: Some("r".into()), date: Some("2026-11-17".into()) };
    let mut links = HashMap::new();
    links.insert(grammar_key("-다 보면"), ("krdict".to_string(), 77));
    let b = body(&l, &links);
    assert!(b.get("review").is_none() && b.get("form").is_none() && b.get("level").is_none());
    assert_eq!(b["compare"][0]["entry"]["id"], 77);
    assert_eq!(b["compare"][0]["entry"]["source"], "krdict");
    assert!(b["meaning_en"].as_str().unwrap().contains("old-fashioned"));
    // a compare pattern without a dictionary row has no link
    let b = body(&l, &HashMap::new());
    assert!(b["compare"][0].get("entry").is_none());
}

#[test]
fn lesson_key_drops_the_lists_trailing_comma() {
    assert_eq!(lesson_key("-을망정,"), "-을망정");
    assert_eq!(lesson_key("-거들랑1"), "-거들랑1");
}

/// Only approved lessons reach the pack table; the log line counts both.
#[test]
fn only_approved_lessons_ship() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(
        "CREATE TABLE entries (id INTEGER PRIMARY KEY, source TEXT);
         CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT);
         CREATE TABLE curriculum (kind TEXT, level INTEGER, entry_id INTEGER, grammar_id INTEGER, form TEXT, rank INTEGER);",
    )
    .unwrap();
    let rows = builtin_grammar();
    for (i, g) in rows.iter().enumerate() {
        c.execute("INSERT INTO curriculum VALUES ('grammar', ?, ?, ?, ?, ?)", rusqlite::params![g.level, i as i64 + 1, i as i64 + 1, g.form_raw, g.rank]).unwrap();
        c.execute("INSERT INTO entries VALUES (?, 'krdict')", [i as i64 + 1]).unwrap();
        c.execute("INSERT INTO grammar VALUES (?, ?, ?)", rusqlite::params![i as i64 + 1, i as i64 + 1, g.form]).unwrap();
    }
    let mut lessons = builtin();
    assert!(lessons.len() >= 4);
    // start from all drafts, whatever the shipped files have already approved
    for l in lessons.iter_mut() {
        l.review = Review { status: "draft".into(), reviewer: None, date: None };
    }
    for l in lessons.iter_mut().take(3) {
        l.review = Review { status: "approved".into(), reviewer: Some("test".into()), date: Some("2026-11-17".into()) };
    }
    let (approved, drafted) = build_table_from(&c, &lessons, &rows).unwrap();
    assert_eq!((approved, drafted), (3, lessons.len()));
    let n: i64 = c.query_row("SELECT COUNT(*) FROM grammar_lesson", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 3);
    // keyed by the curriculum row's own form and level, linked to its entry; the body has no review block
    let (form, level, entry, body): (String, i64, i64, String) =
        c.query_row("SELECT form, level, entry_id, body FROM grammar_lesson ORDER BY rowid LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap();
    let first = &lessons[0];
    assert_eq!(level, first.level);
    assert_eq!(lesson_key(&form), first.form);
    let idx = rows.iter().position(|g| g.level == first.level && lesson_key(&g.form_raw) == first.form).unwrap();
    assert_eq!(entry, idx as i64 + 1);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v.get("review").is_none() && v["examples"].as_array().unwrap().len() >= 4);
    // a lesson for a form the list does not have fails the build instead of shipping silently
    let mut bad = lessons.clone();
    bad[0].form = "-없는형태".into();
    assert!(build_table_from(&Connection::open_in_memory().unwrap(), &bad, &rows).is_err());
}

/// Real lessons stay drafts until the independent review pass (content-review skill) says otherwise.
#[test]
fn nothing_is_approved_without_a_reviewer() {
    for l in builtin() {
        if l.approved() {
            assert!(l.review.reviewer.is_some() && l.review.date.is_some(), "{}", l.form);
        }
    }
}
