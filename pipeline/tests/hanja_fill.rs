//! Guards for the hanja fill from 표준국어대사전: `apply` on small synthetic core + stdict packs
//! (homograph pick, native words stay empty, nothing overwritten, derived words, kengdic) and the
//! committed QA sample.

use kdict_pipeline::hanja_fill::{apply, is_valid_fill, Rule};
use kdict_pipeline::schema;
use rusqlite::{params, Connection};
use serde_json::json;
use std::path::Path;

fn insert(c: &Connection, source: &str, hw: &str, hanja: Option<&str>, pos: &str, ko_def: &str, gloss: &str, extra: serde_json::Value) {
    let mut sense = json!({ "ko_def": ko_def });
    if !gloss.is_empty() {
        sense["gloss"] = gloss.into();
    }
    let mut data = json!({ "senses": [sense] });
    if let (Some(o), Some(e)) = (data.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    c.execute(
        "INSERT INTO entries(headword,hw_norm,hanja,pos,source,lang,rank,kind,data) VALUES(?,?,?,?,?,?,?,'word',?)",
        params![hw, kdict_pipeline::common::hw_norm(hw), hanja, pos, source, if source == "stdict" { "ko" } else { "en" }, 1, data.to_string()],
    )
    .unwrap();
}

fn packs(dir: &Path) -> (Connection, std::path::PathBuf) {
    let sp = dir.join("stdict.sqlite");
    let s = Connection::open(&sp).unwrap();
    s.execute_batch(schema::COMMON).unwrap();
    let n = json!({});
    for (hw, h, d) in [
        ("사기", Some("士氣"), "의욕이나 자신감 따위로 충만하여 굽힐 줄 모르는 기세."),
        ("사기", Some("沙器"), "진흙이나 광물 따위의 가루로 빚어서 구운 희고 매끄러운 그릇."),
        ("사기", Some("詐欺"), "나쁜 꾀로 남을 속임."),
        ("사기", Some("史記"), "중국 한나라의 사마천이 엮은 역사책."),
        ("하늘", None, "땅 위로 끝없이 펼쳐진 무한대의 공간."),
        ("하늘", Some("何訥"), "사람 이름."),
        ("나이", None, "사람이나 동·식물 따위가 세상에 나서 살아온 햇수."),
        ("나이", Some("梛移那移"), "돈이나 물건을 잠시 둘러댐."),
        ("범죄율", Some("犯罪率"), "범죄가 일어나는 비율."),
        ("환초", Some("環礁"), "고리 모양으로 배열된 산호초."),
        ("중앙아시아", Some("中央"), "유라시아 대륙의 중앙부에 위치한 건조 지대."),
        ("고유전학", Some("古遺傳學"), "화석화한 동식물의 유전에 관하여 연구하는 학문."),
        ("동선", Some("動線"), "건축물에서 사람이 움직이는 자취나 방향을 나타내는 선."),
        ("동선", Some("銅線"), "구리로 만든 줄."),
        ("시원하다", None, "덥지도 춥지도 않고 적당하게 서늘하다."),
    ] {
        insert(&s, "stdict", hw, h, "noun", d, "", n.clone());
    }
    // 시원하다 is an adjective in stdict
    s.execute("UPDATE entries SET pos = 'adjective' WHERE hw_norm IN ('시원하다')", []).unwrap();

    let core = Connection::open(dir.join("core.sqlite")).unwrap();
    core.execute_batch(schema::COMMON).unwrap();
    // 사기: two krdict entries already have hanja, the third has none
    insert(&core, "krdict", "사기", Some("士氣"), "noun", "의욕이나 자신감이 충만한 기세.", "morale", json!({}));
    insert(&core, "krdict", "사기", Some("詐欺"), "noun", "자신의 이익을 위해 남을 속임.", "fraud", json!({}));
    insert(&core, "krdict", "사기", None, "noun", "진흙이나 광물 등의 가루로 빚어서 구운 희고 매끄러운 그릇. 또는 그 재료로 만든 물건.", "porcelain", json!({}));
    insert(&core, "krdict", "하늘", None, "noun", "땅 위로 펼쳐진 무한히 넓은 공간.", "sky", json!({}));
    insert(&core, "krdict", "나이", None, "noun", "사람이나 동물 또는 식물 등이 세상에 나서 살아온 햇수.", "age", json!({}));
    insert(&core, "krdict", "범죄율", None, "noun", "한 나라나 지역에서 일정 기간 동안 범죄가 일어난 비율.", "crime rate", json!({}));
    insert(&core, "krdict", "중앙아시아", None, "noun", "아시아의 중앙부.", "Central Asia", json!({}));
    // an entry that already has hanja is never touched, even when stdict disagrees
    insert(&core, "krdict", "동선", Some("同船"), "noun", "한 배를 탐.", "sharing a ship", json!({}));
    insert(&core, "krdict", "동선", None, "noun", "건축물에서 사람이 움직이는 방향을 나타내는 선.", "flow line", json!({}));
    // derived: 신중 has one hanja in krdict, 신중하다 has none (and stdict has no hanja for it)
    insert(&core, "krdict", "신중", Some("愼重"), "noun", "매우 조심스러움.", "prudence", json!({}));
    insert(&core, "krdict", "신중하다", None, "adjective", "매우 조심스럽다.", "prudent", json!({}));
    // native, marked by the origin note
    insert(&core, "krdict", "아기", None, "noun", "어린아이.", "baby", json!({ "origin_note": "고유어: 아기" }));
    // kengdic-only
    insert(&core, "kengdic", "환초", None, "other", "", "atoll", json!({}));
    insert(&core, "kengdic", "고유전학", None, "other", "", "paleogenetics", json!({}));
    insert(&core, "kengdic", "시원하다", None, "adjective", "", "to be cool", json!({}));
    core.execute_batch("CREATE TABLE IF NOT EXISTS x(a)").unwrap();
    (core, sp)
}

fn hanja_of(c: &Connection, hw: &str, gloss: &str) -> Option<String> {
    c.query_row("SELECT hanja FROM entries WHERE hw_norm = ?1 AND gloss IS NULL AND data LIKE ?2", params![hw, format!("%{gloss}%")], |r| r.get(0)).unwrap()
}

#[test]
fn fills_follow_the_rules_and_never_invent() {
    let dir = tempfile::tempdir().unwrap();
    let (core, sp) = packs(dir.path());
    let (stats, records) = apply(&core, &sp).unwrap();

    // 사기 (porcelain) picks 沙器 among four homographs; the other two krdict entries are untouched
    assert_eq!(hanja_of(&core, "사기", "porcelain").as_deref(), Some("沙器"));
    assert_eq!(hanja_of(&core, "사기", "morale").as_deref(), Some("士氣"));
    assert_eq!(hanja_of(&core, "사기", "fraud").as_deref(), Some("詐欺"));
    // native words stay empty: 하늘, 나이 (its homograph with hanja is another word), 아기 (origin note)
    assert_eq!(hanja_of(&core, "하늘", "sky"), None);
    assert_eq!(hanja_of(&core, "나이", "age"), None);
    assert_eq!(hanja_of(&core, "아기", "baby"), None);
    // a unique, matching entry; a partial hanja (中央 for 중앙아시아) is not copied
    assert_eq!(hanja_of(&core, "범죄율", "crime rate").as_deref(), Some("犯罪率"));
    assert_eq!(hanja_of(&core, "중앙아시아", "Central Asia"), None);
    // existing hanja is never overwritten
    assert_eq!(hanja_of(&core, "동선", "sharing a ship").as_deref(), Some("同船"));
    assert_eq!(hanja_of(&core, "동선", "flow line").as_deref(), Some("動線"));
    // derived: stem hanja + native suffix
    assert_eq!(hanja_of(&core, "신중하다", "prudent").as_deref(), Some("愼重하다"));
    // kengdic: one stdict candidate; 시원하다 has a native stdict entry (no hanja) and stays empty
    let k = |hw: &str| -> Option<String> { core.query_row("SELECT hanja FROM entries WHERE source='kengdic' AND hw_norm=?1", [hw], |r| r.get(0)).unwrap() };
    assert_eq!(k("환초").as_deref(), Some("環礁"));
    assert_eq!(k("고유전학").as_deref(), Some("古遺傳學"));
    assert_eq!(k("시원하다"), None);

    // every fill is CJK + allowed suffix, marked in the data, and indexed by character
    assert_eq!(records.len(), stats.total());
    assert_eq!(stats.krdict_homograph + stats.krdict_unique, 3, "{stats:?}"); // 사기, 동선, 범죄율
    for r in &records {
        assert!(is_valid_fill(&r.hanja), "{r:?}");
        let (h, data): (String, String) = core.query_row("SELECT hanja, data FROM entries WHERE id = ?1", [r.id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(h, r.hanja);
        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        assert_eq!(v["hanja_src"], "stdict");
        assert_eq!(v["hanja_rule"], r.rule.as_str());
        let n: i64 = core.query_row("SELECT COUNT(*) FROM hanja_words WHERE entry_id = ?1", [r.id], |r| r.get(0)).unwrap();
        assert_eq!(n as usize, kdict_pipeline::common::cjk_chars(&r.hanja).len(), "{r:?}");
    }
    assert!(records.iter().any(|r| r.rule == Rule::Homograph && r.headword == "사기"));
    // entries that were not filled have no marker
    let marked: i64 = core.query_row("SELECT COUNT(*) FROM entries WHERE data LIKE '%hanja_src%'", [], |r| r.get(0)).unwrap();
    assert_eq!(marked as usize, records.len());
}

#[test]
fn a_second_run_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (core, sp) = packs(dir.path());
    let first = apply(&core, &sp).unwrap().1.len();
    assert!(first > 0);
    // what is filled now has hanja: nothing new
    let again = apply(&core, &sp).unwrap();
    assert_eq!(again.1.len(), 0, "{:?}", again.1);
}

#[test]
fn committed_qa_sample_is_well_formed() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("qa/hanja_fill_sample.tsv");
    let Ok(text) = std::fs::read_to_string(&p) else { return }; // written by a full build, committed with the review
    let mut lines = text.lines();
    assert_eq!(lines.next().unwrap(), "headword\thanja\tsource\tcore_definition\tstdict_definition_used\trule");
    let mut n = std::collections::HashMap::new();
    for l in lines {
        let f: Vec<&str> = l.split('\t').collect();
        assert_eq!(f.len(), 6, "{l}");
        assert!(is_valid_fill(f[1]), "{l}");
        assert!(["krdict", "kengdic"].contains(&f[2]), "{l}");
        assert!(["unique", "homograph", "kengdic", "derived"].contains(&f[5]), "{l}");
        *n.entry(f[5]).or_insert(0) += 1;
    }
    assert!(n.get("homograph").copied().unwrap_or(0) <= 60 && n.get("derived").copied().unwrap_or(0) <= 30, "{n:?}");
    assert!(text.lines().count() <= 151);
}
