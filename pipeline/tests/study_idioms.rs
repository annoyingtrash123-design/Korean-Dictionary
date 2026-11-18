//! Guards for the "Idioms & proverbs" study lists: the committed TSVs parse, every list has its size
//! and unique contiguous ranks, and `add_items` resolves entries in the three packs.

use kdict_pipeline::study_idioms::*;
use rusqlite::{params, Connection};
use std::collections::HashSet;

#[test]
fn tsvs_parse_with_expected_list_sizes_and_unique_ranks() {
    let items = builtin();
    for (list, _, n) in LISTS {
        let mut ranks: Vec<i64> = items.iter().filter(|i| i.list == list).map(|i| i.rank).collect();
        assert_eq!(ranks.len(), n, "{list}");
        let set: HashSet<i64> = ranks.iter().copied().collect();
        assert_eq!(set.len(), n, "{list}: ranks are not unique");
        ranks.sort_unstable();
        assert_eq!(ranks, (1..=n as i64).collect::<Vec<_>>(), "{list}: ranks are not 1..=N");
    }
    assert_eq!(items.len(), LISTS.iter().map(|l| l.2).sum::<usize>(), "rows of an unknown list");
    for it in &items {
        assert!(!it.text.is_empty() && !it.headword.is_empty() && !it.ext_id.is_empty(), "{it:?}");
        assert!(!it.text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{f900}'..='\u{faff}').contains(&c)), "hanja noise in the title: {it:?}");
    }
}

#[test]
fn lists_are_ordered_by_score_and_nest() {
    let items = builtin();
    let of = |l: &str| items.iter().filter(|i| i.list == l).collect::<Vec<_>>();
    // the top lists are sorted by score; "All learner proverbs" has the zero-count proverbs last
    for l in ["sokdam100", "sokdam_all", "seongeo100", "seongeo300", "seongeo_learner"] {
        let v = of(l);
        assert!(v.windows(2).all(|w| w[0].score >= w[1].score), "{l} is not in score order");
    }
    assert_eq!(of("sokdam_all").iter().filter(|i| i.score == 0.0).count() > 100, true);
    // the essential 성어 are the first 100 of the common ones
    let a: Vec<_> = of("seongeo100").iter().map(|i| &i.text).collect();
    let b: Vec<_> = of("seongeo300").iter().take(100).map(|i| &i.text).collect();
    assert_eq!(a, b);
    // 성어 are four syllables (hanja glosses already removed); no false positives from the review survive
    for i in of("seongeo300") {
        assert_eq!(i.text.chars().count(), 4, "{i:?}");
    }
    let texts: HashSet<&str> = of("seongeo300").iter().map(|i| i.text.as_str()).collect();
    for dropped in ["국가대표", "돌연변이", "의사소통", "공작부인", "양성평등", "공동생활", "근친상간", "기회균등", "자기중심"] {
        assert!(!texts.contains(dropped), "{dropped} is not a 성어");
    }
    for kept in ["우유부단", "설상가상", "십중팔구", "만장일치", "자급자족", "명실상부"] {
        assert!(texts.contains(kept), "{kept} is missing");
    }
    // "All learner 성어": krdict entries only, four syllables, no dropped compounds, and it reaches beyond
    // the top 300 (krdict 성어 the first review cut did not cover)
    let learner = of("seongeo_learner");
    assert!(learner.iter().all(|i| i.source == "krdict" && i.text.chars().count() == 4), "seongeo_learner has a non-krdict row");
    let lt: HashSet<&str> = learner.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(lt.len(), learner.len(), "seongeo_learner has duplicates");
    assert!(!lt.contains("돌연변이") && !lt.contains("의사소통"));
    assert!(lt.contains("만장일치") && lt.contains("각골난망"));
    assert!(lt.iter().any(|t| !texts.contains(t)), "seongeo_learner adds nothing to seongeo300");
}

fn pack(path: &std::path::Path, rows: &[(i64, &str, &str, &str, &str, Option<&str>)]) {
    let c = Connection::open(path).unwrap();
    c.execute_batch(
        "CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, source TEXT NOT NULL, ext_id TEXT, gloss TEXT, data TEXT NOT NULL);",
    )
    .unwrap();
    for (id, hw, norm, source, ext, gloss) in rows {
        let data = if *source == "krdict" {
            r#"{"senses":[{"gloss":"ignored","ko_def":"한국어 뜻"}]}"#
        } else {
            r#"{"senses":[{"ko_def":"눈 위에 서리가 덮인다는 뜻으로, 난처한 일이나 불행한 일이 잇따라 일어남을 이르는 말."}]}"#
        };
        c.execute("INSERT INTO entries VALUES (?1,?2,?3,?4,?5,?6,?7)", params![id, hw, norm, source, ext, gloss, data]).unwrap();
    }
}

fn item(list: &str, rank: i64, text: &str, source: &str, ext: &str, hw: &str) -> Item {
    Item { list: list.into(), rank, text: text.into(), source: source.into(), ext_id: ext.into(), headword: hw.into(), score: 1.0 }
}

#[test]
fn rows_resolve_in_their_pack_with_the_right_gloss_and_ids() {
    let dir = tempfile::tempdir().unwrap();
    let (core, st) = (dir.path().join("core.sqlite"), dir.path().join("stdict.sqlite"));
    pack(&core, &[
        (7, "가난 구제는 나라님[임금님]도 못한다", "가난구제는나라님[임금님]도못한다", "krdict", "16391", Some("Even a king can't relieve poverty")),
        (8, "가난 구제는 나라님도 못한다", "가난구제는나라님도못한다", "krdict", "16391", Some("another variant")),
    ]);
    pack(&st, &[(900, "설상가상", "설상가상", "stdict", "448384", Some("눈 위에 서리가 덮인다는 뜻으로"))]);
    let items = [
        item("sokdam_all", 1, "가난 구제는 나라님도 못한다", "krdict", "16391", "가난 구제는 나라님[임금님]도 못한다"),
        item("seongeo100", 1, "설상가상", "stdict", "448384", "설상가상"),
        item("seongeo100", 2, "금상첨화", "opendict", "1", "금상첨화"),
        item("seongeo100", 3, "없는말", "stdict", "999", "없는말"),
    ];
    // no opendict pack: its row (and the unknown stdict one) are left out
    let stats = add_items(&core, Some(&st), None, &items).unwrap();
    assert_eq!(stats, Stats { items: 2, unresolved: 2 });
    let c = Connection::open(&core).unwrap();
    let row = |list: &str, rank: i64| -> (String, String, Option<i64>, String, String) {
        c.query_row("SELECT kind, title, entry_id, pack, gloss FROM study_item WHERE list = ?1 AND rank = ?2", params![list, rank], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
    };
    // krdict row: the core entry id (the exact headword wins over the other variant) and the English gloss
    assert_eq!(row("sokdam_all", 1), ("sokdam".into(), "가난 구제는 나라님도 못한다".into(), Some(7), "core".into(), "Even a king can't relieve poverty".into()));
    // stdict-only row: no core id (the pack's ids are not stable), the Korean definition, at most 80 characters
    let (kind, _, id, pack, gloss) = row("seongeo100", 1);
    assert_eq!((kind.as_str(), id, pack.as_str()), ("seongeo", None, "stdict"));
    assert!(gloss.starts_with("눈 위에 서리가 덮인다는 뜻으로")&& gloss.chars().count() <= 80, "{gloss}");
    // the lists page by (list, rank)
    let n: i64 = c.query_row("SELECT COUNT(*) FROM study_item", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 2);
}

#[test]
fn unknown_list_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let core = dir.path().join("core.sqlite");
    pack(&core, &[]);
    assert!(add_items(&core, None, None, &[item("nope", 1, "x", "krdict", "1", "x")]).is_err());
}
