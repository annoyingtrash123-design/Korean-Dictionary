//! Quality gate of the AI-drafted English for 속담/성어 that krdict lacks (`data/idiom_extra.tsv`): structure,
//! provenance flags, that every item is one the study lists / review files ask for, and (when the packs are
//! built locally) that the Korean text is copied verbatim from stdict / 우리말샘.

use kdict_pipeline::idiom_extra::*;
use kdict_pipeline::study_idioms;
use std::collections::{HashMap, HashSet};

const CAND_SEONGEO: &str = include_str!("../qa/frequency/candidates_seongeo.tsv");
const REVIEW_SEONGEO: &str = include_str!("../qa/frequency/seongeo_review.tsv");

fn table(tsv: &str) -> Vec<HashMap<String, String>> {
    let mut lines = tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    lines.map(|l| head.iter().zip(l.split('\t')).map(|(h, v)| (h.to_string(), v.to_string())).collect()).collect()
}

/// 성어 the review kept that only stdict / opendict have: (text, hanja).
fn wanted_seongeo() -> Vec<(String, String)> {
    let cand: HashMap<String, HashMap<String, String>> = table(CAND_SEONGEO).into_iter().map(|c| (c["id"].clone(), c)).collect();
    table(REVIEW_SEONGEO)
        .into_iter()
        .filter(|r| r["status"] == "keep")
        .map(|r| &cand[&r["id"]])
        .filter(|c| c["krdict_id"].is_empty())
        .map(|c| (c["text"].clone(), c["hanja"].clone()))
        .collect()
}

/// The 속담 of `sokdam100` that have no core (krdict) entry.
fn wanted_sokdam() -> Vec<String> {
    study_idioms::builtin().into_iter().filter(|i| i.list == "sokdam100" && i.source != "krdict").map(|i| i.text).collect()
}

#[test]
fn rows_pass_the_quality_gate_and_are_unique() {
    let all = builtin();
    assert!(!all.is_empty());
    let (mut heads, mut srcs) = (HashSet::new(), HashSet::new());
    for p in &all {
        assert!(heads.insert((p.kind.clone(), p.headword.clone())), "{} twice", p.headword);
        assert!(srcs.insert((p.src_pack.clone(), p.src_id.clone())), "{}: source entry used twice", p.headword);
        let pr = problems(p);
        assert!(pr.is_empty(), "{}: {pr:?}", p.headword);
    }
}

/// A row rejected because krdict has the word (kept in the file for the record).
fn krdict_rejected(p: &IdiomItem) -> bool {
    p.review.starts_with("rejected: krdict has this entry (")
}

#[test]
fn every_item_is_one_the_lists_ask_for() {
    let seongeo: HashMap<String, String> = wanted_seongeo().into_iter().collect();
    let sokdam: HashSet<String> = wanted_sokdam().into_iter().collect();
    let study = study_idioms::builtin();
    for p in builtin().into_iter().filter(|p| !krdict_rejected(p)) {
        if p.kind == "seongeo" {
            assert_eq!(seongeo.get(&p.headword), Some(&p.hanja), "{} is not a kept krdict-less 성어 (or its hanja differ)", p.headword);
        } else {
            assert!(sokdam.contains(&p.headword), "{} is not a sokdam100 proverb without a krdict entry", p.headword);
        }
        // where a study list row exists it must name the same pack entry
        for s in study.iter().filter(|s| s.text == p.headword) {
            assert_eq!((s.source.as_str(), s.ext_id.as_str()), (p.src_pack.as_str(), p.src_id.as_str()), "{}: study list {} points elsewhere", p.headword, s.list);
        }
    }
}

/// Switched on in the commit that completes the file: nothing the lists need is missing.
#[test]
fn the_file_covers_every_item() {
    // a work-in-progress commit says so in the file ("# PARTIAL" line)
    if include_str!("../data/idiom_extra.tsv").lines().any(|l| l == "# PARTIAL") {
        return;
    }
    let have: HashSet<(String, String)> = builtin().into_iter().filter(|p| !krdict_rejected(p)).map(|p| (p.kind, p.headword)).collect();
    let missing: Vec<String> = wanted_seongeo().into_iter().map(|(t, _)| ("seongeo".to_string(), t)).chain(wanted_sokdam().into_iter().map(|t| ("sokdam".to_string(), t))).filter(|k| !have.contains(k)).map(|k| k.1).collect();
    assert!(missing.is_empty(), "{} items without a row: {missing:?}", missing.len());
    assert_eq!(builtin().iter().filter(|p| !krdict_rejected(p)).count(), wanted_seongeo().len() + wanted_sokdam().len());
}

/// Every recurring mistake becomes a check: the overlay is for words krdict LACKS. An approved item (or a draft that
/// could be approved) whose headword and hanja are a krdict entry is a duplicate -- it must say so in its review column.
/// Offline against the candidate list (krdict_id), and against the core pack's krdict entries when it is built locally.
#[test]
fn no_live_item_duplicates_a_krdict_entry() {
    let cand = table(CAND_SEONGEO);
    for p in builtin().iter().filter(|p| !krdict_rejected(p) && p.kind == "seongeo") {
        let c = cand.iter().find(|c| c["text"] == p.headword && c["hanja"].split('|').any(|h| h == p.hanja)).unwrap_or_else(|| panic!("{} is not a candidate", p.headword));
        assert!(c["krdict_id"].is_empty(), "{}: krdict has this entry (id {}): mark the row `rejected: krdict has this entry (<ext_id>)`", p.headword, c["krdict_id"]);
    }
    let Some(core) = pack_dir().map(|d| d.join("core.sqlite")).filter(|p| p.exists()) else { return };
    let conn = rusqlite::Connection::open_with_flags(core, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mut checked = 0;
    for p in builtin() {
        let dup = krdict_duplicate(&conn, &p).unwrap();
        if krdict_rejected(&p) {
            let ext = dup.unwrap_or_else(|| panic!("{}: rejected as a krdict duplicate but the core pack has no such krdict entry", p.headword));
            assert!(p.review == format!("rejected: krdict has this entry ({ext})"), "{}: {}", p.headword, p.review);
        } else {
            assert!(dup.is_none(), "{} duplicates krdict entry {:?} (review: {})", p.headword, dup, p.review);
        }
        checked += 1;
    }
    eprintln!("idiom_extra: {checked} items checked against krdict in the core pack");
}

fn pack_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("KDICT_PACK_DIR").map(std::path::PathBuf::from).or_else(|| {
        let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        here.ancestors().map(|a| a.join("pipeline/out")).find(|p| p.join("stdict.sqlite").exists())
    })
}

#[test]
fn nothing_ships_before_review() {
    let all = builtin();
    let n_ok = all.iter().filter(|p| is_approved(p)).count();
    assert_eq!(approved().len(), n_ok);
    assert!(all.iter().all(|p| p.review == "draft" || p.review.starts_with("approved") || p.review.starts_with("rejected: ")));
}

#[test]
fn entry_json_flags_every_ai_field() {
    for p in builtin() {
        let e = to_entry(&p);
        assert_eq!((e.source, e.lang), ("hanppuri", "en"));
        assert_eq!(e.kind, if p.kind == "seongeo" { "word" } else { "proverb" });
        assert_eq!(e.hanja.is_some(), p.kind == "seongeo");
        assert_eq!(e.ext_id, format!("idiom-{}-{}", p.src_pack, p.src_id));
        let s = &e.data["senses"][0];
        assert_eq!((s["gloss_ai"].as_bool(), s["def_ai"].as_bool()), (Some(true), Some(true)), "the English is always AI");
        assert_eq!(s["ko_def"], p.ko_def.as_str());
        assert_eq!(s["ko_def_source"], format!("{}:{}", p.src_pack, p.src_id).as_str());
        let exs = s["examples"].as_array().unwrap();
        assert_eq!(exs.len(), p.examples.len());
        for (x, src) in exs.iter().zip(&p.examples) {
            assert_eq!(x["en_ai"], true);
            assert_eq!(x["ko_ai"] == true, src.ko_source == "ai", "{}: {}", p.headword, src.ko);
            assert_eq!(x["type"], "sentence");
        }
    }
}

/// With the built packs at hand (`KDICT_PACK_DIR`, or `pipeline/out` of the checkout this one is a worktree of),
/// the Korean definition and every dictionary example are exactly the pack's text. Skipped without packs;
/// the build runs the same check (`study_idioms::add_items`).
#[test]
fn korean_text_is_verbatim_from_the_packs() {
    let Some(dir) = pack_dir() else { return };
    let open = |n: &str| rusqlite::Connection::open_with_flags(dir.join(format!("{n}.sqlite")), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok();
    let (st, od) = (open("stdict"), open("opendict"));
    let mut checked = 0;
    for p in builtin() {
        let conn = if p.src_pack == "stdict" { st.as_ref() } else { od.as_ref() };
        if let Some(c) = conn {
            match check_against_pack(c, &p) {
                Ok(true) => checked += 1,
                Ok(false) => panic!("{}: {}:{} is not in the pack", p.headword, p.src_pack, p.src_id),
                Err(e) => panic!("{e:#}"),
            }
        }
    }
    eprintln!("idiom_extra: {checked} items checked against the packs in {}", dir.display());
}

#[test]
fn gate_rejects_bad_rows() {
    let good = builtin().remove(0);
    let mut p = good.clone();
    p.review = "ok".into();
    assert!(problems(&p).iter().any(|m| m.contains("review")));
    let mut p = good.clone();
    p.examples.truncate(1);
    assert!(problems(&p).iter().any(|m| m.contains("examples")));
    let mut p = good.clone();
    p.examples[0].ko = "전혀 상관없는 문장입니다.".into();
    assert!(problems(&p).iter().any(|m| m.contains("does not contain")));
    let mut p = good.clone();
    p.examples[0].en = "이건 한국어입니다.".into();
    assert!(problems(&p).iter().any(|m| m.contains("English")));
    let mut p = good.clone();
    p.examples[0].ko_source = "opendict".into();
    assert!(problems(&p).iter().any(|m| m.contains("ko_source")));
    let mut p = good.clone();
    p.en_def = "Something strange.".into();
    assert!(problems(&p).iter().any(|m| m.contains("en_def")) || problems(&p).iter().any(|m| m.contains("lit.")));
    let mut p = good;
    p.en_gloss = "신비".into();
    assert!(problems(&p).iter().any(|m| m.contains("en_gloss")));
    assert!(parse("P\tx\n").is_err());
    assert!(parse("E\t가\t가 나 다.\tai\tA b c.\n").is_err(), "an example needs its P row first");
    assert!(has_unquoted_hangul("He said 말."));
    assert!(!has_unquoted_hangul("He said \"말\" twice."));
}

mod study_rows {
    use super::*;
    use kdict_pipeline::study_idioms::{add_items_with, Item};
    use rusqlite::{params, Connection};

    fn pack(path: &std::path::Path, rows: &[(i64, &str, &str, &str, &str, &str, &str)]) {
        let c = Connection::open(path).unwrap();
        c.execute_batch("CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, source TEXT NOT NULL, ext_id TEXT, gloss TEXT, data TEXT NOT NULL);").unwrap();
        for (id, hw, norm, source, ext, gloss, data) in rows {
            c.execute("INSERT INTO entries VALUES (?1,?2,?3,?4,?5,?6,?7)", params![id, hw, norm, source, ext, gloss, data]).unwrap();
        }
    }

    #[test]
    fn an_approved_row_points_the_study_item_at_the_core_entry() {
        let mut x = builtin().into_iter().find(|p| p.headword == "백문이 불여일견").unwrap();
        x.review = "approved:test".into();
        let entry = to_entry(&x);
        let dir = tempfile::tempdir().unwrap();
        let (core, st) = (dir.path().join("core.sqlite"), dir.path().join("stdict.sqlite"));
        let core_data = serde_json::to_string(&entry.data).unwrap();
        pack(&core, &[(70, "백문이 불여일견", "백문이불여일견", "hanppuri", &entry.ext_id, &x.en_gloss, &core_data)]);
        let st_data = serde_json::json!({"senses": [{"ko_def": x.ko_def}]}).to_string();
        pack(&st, &[(900, "백문이 불여일견(不如一見)", "백문이불여일견", "stdict", &x.src_id, "", &st_data)]);
        let it = Item { list: "sokdam100".into(), rank: 1, text: x.headword.clone(), source: "stdict".into(), ext_id: x.src_id.clone(), headword: "백문이 불여일견".into(), score: 1.0 };
        let stats = add_items_with(&core, Some(&st), None, std::slice::from_ref(&it), std::slice::from_ref(&x)).unwrap();
        assert_eq!(stats.items, 1);
        let c = Connection::open(&core).unwrap();
        let (eid, pk, ext, gloss, ok): (Option<i64>, String, String, String, i64) =
            c.query_row("SELECT entry_id, pack, ext_id, gloss, home_ok FROM study_item", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))).unwrap();
        assert_eq!((eid, pk.as_str(), ext.as_str()), (Some(70), "core", entry.ext_id.as_str()));
        assert_eq!(gloss, x.en_gloss);
        assert_eq!(ok, 1, "an approved idiom with full example sentences is a Home card");
        // without the overlay row the item stays a Korean-only stdict row
        let core2 = dir.path().join("core2.sqlite");
        pack(&core2, &[]);
        let stats = add_items_with(&core2, Some(&st), None, std::slice::from_ref(&it), &[]).unwrap();
        assert_eq!(stats.items, 1);
        let pk: String = Connection::open(&core2).unwrap().query_row("SELECT pack FROM study_item", [], |r| r.get(0)).unwrap();
        assert_eq!(pk, "stdict");
        // a Korean definition that is not the pack's text fails the build
        let mut bad = x.clone();
        bad.ko_def = "바뀐 정의입니다.".into();
        assert!(add_items_with(&core, Some(&st), None, std::slice::from_ref(&it), std::slice::from_ref(&bad)).is_err());
    }
}
