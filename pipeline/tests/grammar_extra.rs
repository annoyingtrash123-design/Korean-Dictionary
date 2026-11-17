//! Quality gate of the reviewed grammar overlay (`data/grammar_extra.tsv`): structure, provenance
//! flags, and that every pattern is one the NIKL curriculum list asks for.

use kdict_pipeline::curriculum::{builtin_grammar, grammar_forms, grammar_key};
use kdict_pipeline::grammar_extra::*;

#[test]
fn overlay_rows_pass_the_quality_gate() {
    let all = builtin();
    assert!(!all.is_empty());
    let mut seen = std::collections::HashSet::new();
    for p in &all {
        assert!(seen.insert(p.pattern.clone()), "{} twice", p.pattern);
        let pr = problems(p);
        assert!(pr.is_empty(), "{}: {pr:?}", p.pattern);
    }
}

#[test]
fn every_pattern_is_a_curriculum_row_of_the_same_level_and_category() {
    let list = builtin_grammar();
    for p in builtin() {
        let key = grammar_key(&p.pattern);
        let g = list
            .iter()
            .find(|g| grammar_forms(g).iter().any(|f| grammar_key(f) == key))
            .unwrap_or_else(|| panic!("{} is not in the curriculum list", p.pattern));
        assert_eq!(g.level, p.nikl_level, "{}", p.pattern);
        let want = match g.category.as_str() {
            "연결어미" => "Connective endings",
            "종결어미" => "Final endings",
            "표현" => "Expressions",
            "조사" => "Particles",
            c => panic!("{c}"),
        };
        assert_eq!(p.category, want, "{}", p.pattern);
    }
}

#[test]
fn entry_json_flags_every_ai_field() {
    for (i, p) in builtin().iter().enumerate() {
        let e = to_entry(p, i);
        assert_eq!((e.source, e.kind.as_str(), e.lang), ("hanppuri", "grammar", "en"));
        assert_eq!(e.level, Some((p.nikl_level + 1) / 2));
        let s = &e.data["senses"][0];
        assert_eq!(s["def_ai"], true, "the English summary is always AI");
        assert_eq!(s["ko_def_ai"] == true, p.ko_def_source == "ai", "{}", p.pattern);
        assert_eq!(s["ko_def_source"], p.ko_def_source.as_str());
        let exs = s["examples"].as_array().unwrap();
        assert_eq!(exs.len(), p.examples.len());
        for (x, src) in exs.iter().zip(&p.examples) {
            assert_eq!(x["en_ai"], true);
            assert_eq!(x["ko_ai"] == true, src.ko_source == "ai", "{}: {}", p.pattern, src.ko);
        }
    }
}

#[test]
fn gate_rejects_bad_rows() {
    let good = builtin().remove(0);
    let mut p = good.clone();
    p.ko_def_source = "wikipedia:1".into();
    assert!(problems(&p).iter().any(|m| m.contains("ko_def_source")));
    let mut p = good.clone();
    p.examples.truncate(1);
    assert!(problems(&p).iter().any(|m| m.contains("examples")));
    let mut p = good.clone();
    p.examples[0].ko = "전혀 상관없는 문장입니다.".into();
    assert!(problems(&p).iter().any(|m| m.contains("does not contain")));
    let mut p = good;
    p.examples[0].en = "이건 한국어입니다.".into();
    assert!(problems(&p).iter().any(|m| m.contains("English")));
    assert!(parse("P\tx\n").is_err());
    assert!(parse("E\t-가\t가 나 다.\tai\tA b c.\textra\n").is_err());
    assert!(parse("E\t-가\t가 나 다.\tai\tA b c.\n").is_err(), "an example needs its P row first");
}
