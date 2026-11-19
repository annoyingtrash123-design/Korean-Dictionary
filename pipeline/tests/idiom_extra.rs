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

#[test]
fn every_item_is_one_the_lists_ask_for() {
    let seongeo: HashMap<String, String> = wanted_seongeo().into_iter().collect();
    let sokdam: HashSet<String> = wanted_sokdam().into_iter().collect();
    let study = study_idioms::builtin();
    for p in builtin() {
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
    let have: HashSet<(String, String)> = builtin().into_iter().map(|p| (p.kind, p.headword)).collect();
    let missing: Vec<String> = wanted_seongeo().into_iter().map(|(t, _)| ("seongeo".to_string(), t)).chain(wanted_sokdam().into_iter().map(|t| ("sokdam".to_string(), t))).filter(|k| !have.contains(k)).map(|k| k.1).collect();
    assert!(missing.is_empty(), "{} items without a row: {missing:?}", missing.len());
    assert_eq!(builtin().len(), wanted_seongeo().len() + wanted_sokdam().len());
}

#[test]
fn nothing_ships_before_review() {
    let all = builtin();
    let n_ok = all.iter().filter(|p| is_approved(p)).count();
    assert_eq!(approved().len(), n_ok);
    assert!(all.iter().all(|p| p.review == "draft" || p.review.starts_with("approved")));
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
    let dir = std::env::var_os("KDICT_PACK_DIR").map(std::path::PathBuf::from).or_else(|| {
        let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        here.ancestors().map(|a| a.join("pipeline/out")).find(|p| p.join("stdict.sqlite").exists())
    });
    let Some(dir) = dir else { return };
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
