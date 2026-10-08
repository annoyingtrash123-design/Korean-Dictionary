use kdict_pipeline::build::{self, BuildOpts, Sources};
use kdict_pipeline::common::*;
use kdict_pipeline::xml::{for_each, for_each_file};
use kdict_pipeline::{cedict, freq, kaikki, kengdic, krdict, opendict, pack, stdict, tatoeba, unihan, zhwikt};
use rusqlite::Connection;
use serde_json::Value;
use std::io::{BufReader, Cursor, Write};
use std::path::{Path, PathBuf};

fn fx(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn krdict_entries() -> Vec<Entry> {
    let mut v = Vec::new();
    krdict::parse_file(&fx("krdict_sample.xml"), |e| {
        v.push(e);
        Ok(())
    })
    .unwrap();
    v
}

fn stdict_entries() -> Vec<Entry> {
    let mut v = Vec::new();
    stdict::parse_file(&fx("stdict_sample.xml"), |e| {
        v.push(e);
        Ok(())
    })
    .unwrap();
    v
}

#[test]
fn xml_strips_control_bytes() {
    // the fixture contains raw 0x01 / 0x1f bytes which are illegal in XML 1.0
    let raw = std::fs::read(fx("krdict_sample.xml")).unwrap();
    assert!(raw.contains(&0x01));
    let mut n = 0;
    for_each(Cursor::new(raw), "LexicalEntry", |_| {
        n += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(n, 7);
}

#[test]
fn krdict_parsing() {
    let es = krdict_entries();
    assert_eq!(es.len(), 7);
    let hak = &es[0];
    assert_eq!((hak.headword.as_str(), hak.pos.as_str(), hak.kind.as_str()), ("학교", "noun", "word"));
    assert_eq!(hak.hanja.as_deref(), Some("學校"));
    assert_eq!(hak.level, Some(1));
    assert_eq!(hak.homonym, None);
    assert_eq!(hak.pron.as_deref(), Some("학꾜"));
    let s = &hak.data["senses"][0];
    assert_eq!(s["gloss"], "school"); // French equivalent ignored
    assert_eq!(s["def"], "A place where teachers teach students.");
    // double-escaped annotation is fully unescaped
    assert_eq!(s["note"], "'학교에 가다' is common & useful");
    assert_eq!(s["examples"][0]["type"], "phrase");
    assert_eq!(s["examples"][1]["type"], "dialogue");
    assert_eq!(s["examples"][1]["ko"], "가: 어디 가요?\n나: 학교에 가요.");
    assert_eq!(s["rel"][0]["type"], "synonym");
    assert_eq!(hak.data["category"], "교육 > 기관");

    let meok = &es[1];
    assert_eq!(meok.homonym, Some(2));
    assert_eq!(meok.pos, "verb");
    assert_eq!(meok.forms, vec!["먹어", "먹엉", "먹으니"]);
    assert_eq!(meok.data["senses"][0]["pattern"], "1이 2를 먹다");
    // control bytes removed from the definition
    assert_eq!(meok.data["senses"][0]["ko_def"], "음식 등을 입을 통하여 배 속으로 보내다.");
    assert_eq!(meok.data["senses"][1]["gloss"], "eat; smoke");

    // grammar: ending -> kind grammar, romanisation is not a gloss
    let aseo = &es[2];
    assert_eq!((aseo.kind.as_str(), aseo.pos.as_str()), ("grammar", "ending"));
    assert!(aseo.data["senses"][0].get("gloss").is_none());
    assert_eq!(aseo.data["senses"][0]["roman"], "-aseo");
    assert_eq!(build::entry_gloss(aseo).unwrap(), "A connective ending used to indicate reason.");

    let prov = &es[3];
    assert_eq!((prov.kind.as_str(), prov.pos.as_str()), ("proverb", "proverb"));
    assert_eq!(prov.level, None);

    assert_eq!(es[4].kind, "grammar"); // particle
    // romanised English lemma of an auxiliary verb
    // conjugation-pointer entry: carried as a pointer, not as a normal sense
    let ptr = es[6].pointer.as_ref().expect("pointer");
    assert_eq!(ptr.forms, vec!["먹고", "먹는데", "먹어"]);
    assert_eq!(ptr.targets, vec![("먹다".to_string(), Some(2)), ("먹다".to_string(), Some(9)), ("없다".to_string(), None)]);
    assert!(es[0].pointer.is_none());
    assert_eq!(es[5].pos, "auxiliary verb");
    assert!(es[5].data["senses"][0].get("gloss").is_none());
}

#[test]
fn stdict_parsing() {
    let es = stdict_entries();
    assert_eq!(es.len(), 4);
    let hak = &es[0];
    assert_eq!(hak.headword, "학교");
    assert_eq!(hak.homonym, Some(1));
    assert_eq!(hak.hanja.as_deref(), Some("學校"));
    assert_eq!(hak.forms, vec!["학교가"]);
    let s = &hak.data["senses"][0];
    // the example with a <source> citation is excluded
    let ex = s["examples"].as_array().unwrap();
    assert_eq!(ex.len(), 1);
    assert_eq!(ex[0]["ko"], "학교에 다니다.");
    assert_eq!(s["rel"][0]["word"], "학원");
    assert_eq!(s["tags"][0], "교육");

    let gam = &es[1];
    assert_eq!(gam.headword, "가감하다");
    assert_eq!(gam.homonym, Some(2));
    assert_eq!(gam.hanja.as_deref(), Some("加減하다"));
    assert_eq!(gam.pos, "verb");
    assert_eq!(gam.data["senses"][0]["pattern"], "1이 2를 가감하다");

    // '^' marks a space in the displayed headword; hw_norm strips it
    let gil = &es[2];
    assert_eq!(gil.headword, "가 는 길");
    assert_eq!(hw_norm(&gil.headword), "가는길");
    assert_eq!(gil.kind, "phrase");

    let aseo = &es[3];
    assert_eq!(aseo.headword, "-아서");
    assert_eq!(hw_norm("-아서"), "아서");
    assert_eq!(aseo.kind, "grammar");
    assert!(aseo.hanja.is_none());
    assert_eq!(aseo.data["origin_note"], "영어: ok");
}

#[test]
fn opendict_parsing() {
    let mut es = Vec::new();
    opendict::parse_file(&fx("opendict_sample.xml"), |e| {
        es.push(e);
        Ok(())
    })
    .unwrap();
    // 5 groups, one has no definition and is dropped
    assert_eq!(es.len(), 4);
    let hak = &es[0];
    assert_eq!((hak.headword.as_str(), hak.source, hak.lang), ("학교", "opendict", "ko"));
    assert_eq!(hak.hanja.as_deref(), Some("學校"));
    assert_eq!(hak.forms, vec!["학교가"]);
    // items of one group_code are merged and ordered by group_order
    let senses = hak.data["senses"].as_array().unwrap();
    assert_eq!(senses.len(), 2);
    assert_eq!(senses[0]["ko_def"], "교사가 학생을 가르치는 기관.");
    assert_eq!(senses[0]["rel"][0]["type"], "hypernym");
    assert_eq!(senses[1]["en"], "school");
    // the example with a <source> citation is excluded
    let ex = senses[1]["examples"].as_array().unwrap();
    assert_eq!(ex.len(), 1);
    assert_eq!(ex[0]["ko"], "학교에 다닌다.");
    assert_eq!(hak.data["category"], "교육");
    // dialect label + region as tags, relation type mapped
    let hk = &es[1];
    assert_eq!(hk.data["senses"][0]["tags"], serde_json::json!(["방언", "경상"]));
    assert_eq!(hk.data["senses"][0]["rel"][0]["type"], "synonym");
    // '^' -> space, phrase, 북한어 tag
    assert_eq!(es[2].headword, "가 는 길");
    assert_eq!(es[2].kind, "phrase");
    assert_eq!(es[2].data["senses"][0]["tags"][0], "북한어");
    // composite POS "관·명" uses its first component; 옛말 label
    assert_eq!(es[3].pos, "determiner");
    assert_eq!(es[3].data["senses"][0]["tags"][0], "옛말");
}

#[test]
fn stdict_helpers() {
    assert_eq!(stdict::split_homonym("가03"), ("가".to_string(), Some(3)));
    assert_eq!(stdict::split_homonym("123"), ("123".to_string(), None));
    assert_eq!(stdict::display_word("가감-하다"), "가감하다");
    assert_eq!(stdict::display_word("-가^하다-"), "-가 하다-");
    assert_eq!(hw_norm("a·b c-d^e"), "abcde");
}

#[test]
fn romanisation_heuristic() {
    assert!(looks_romanized("gajida", "가지다"));
    assert!(looks_romanized("-aseo", "-아서"));
    assert!(looks_romanized("deutada", "듯하다"));
    assert!(!looks_romanized("school", "학교"));
    assert!(!looks_romanized("between", "간"));
}

#[test]
fn kengdic_parsing() {
    let k = kengdic::parse(BufReader::new(std::fs::File::open(fx("kengdic_sample.tsv")).unwrap())).unwrap();
    let by: std::collections::HashMap<_, _> = k.entries.iter().map(|e| ((e.headword.clone(), e.hanja.clone()), e)).collect();
    let hak = by[&("학교".to_string(), Some("學校".to_string()))];
    // deduped case-insensitively, junk edges trimmed, level kept (best of A/B)
    let gl: Vec<&str> = hak.data["senses"].as_array().unwrap().iter().map(|s| s["gloss"].as_str().unwrap()).collect();
    assert_eq!(gl, vec!["school", "educational institution"]);
    assert_eq!(hak.data["kengdic_level"], "A");
    let meok = by[&("먹다".to_string(), None)];
    assert_eq!(meok.data["senses"].as_array().unwrap().len(), 1); // junk gloss dropped
    assert_eq!(meok.pos, "verb");
    assert_eq!(by[&("아름답다".to_string(), None)].pos, "adjective");
    // gloss-less rows only feed hanja_by_surface
    assert!(!by.contains_key(&("고기".to_string(), Some("高氣".to_string()))));
    assert_eq!(k.hanja_by_surface["고기"].len(), 1);
    // first hanja variant is the key, others kept
    let gyo = by[&("교착하다".to_string(), Some("交着하다".to_string()))];
    assert_eq!(gyo.data["hanja_alt"][0], "膠着하다");
    // quality filters + capitalisation
    assert!(k.entries.iter().all(|e| e.headword != "TV방송" && e.headword != "한글"));
    let gl_of = |w: &str| by[&(w.to_string(), None)].data["senses"].as_array().unwrap().iter().map(|s| s["gloss"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(gl_of("서울"), vec!["Seoul"]);
    assert_eq!(gl_of("대한민국"), vec!["the Republic of Korea"]);
    assert_eq!(gl_of("고양이"), vec!["cat; feline"]);
    // surface with spaces -> phrase; non-hangul surface skipped
    let phr = k.entries.iter().find(|e| e.headword == "전원의 행진").unwrap();
    assert_eq!((phr.kind.as_str(), phr.pos.as_str()), ("phrase", "phrase"));
    assert!(k.entries.iter().all(|e| e.headword != "abc"));
    // same surface, different hanja -> separate entries
    assert_eq!(k.hanja_by_surface["강"].len(), 1);
    assert!(by.contains_key(&("강".to_string(), Some("江".to_string()))));
    assert!(by.contains_key(&("강".to_string(), None)));
}

#[test]
fn case_normalisation() {
    assert_eq!(kengdic::normalize_case("Go deaf; Eat, chow down on"), "go deaf; eat, chow down on");
    assert_eq!(kengdic::normalize_case("A man"), "a man");
    assert_eq!(kengdic::normalize_case("TV set"), "TV set");
    assert_eq!(kengdic::normalize_case("I think"), "I think");
    assert_eq!(kengdic::normalize_case("Korea"), "Korea");
    assert_eq!(kengdic::normalize_case("To go"), "to go");
}

#[test]
fn freq_ranks() {
    let r = freq::load_ranks(BufReader::new(std::fs::File::open(fx("ko_freq.txt")).unwrap()));
    assert_eq!(r["이"], 1);
    assert_eq!(r["학교"], 2);
    let st = freq::stem_ranks(&r);
    assert_eq!(st["먹"], 4); // 먹 + 어 (rank of 먹어 = 4), 먹었어 is rank 5
    let ranker = build::Ranker::new(r);
    let mk = |hw: &str, pos: &str, level| Entry { source: "krdict", headword: hw.into(), pos: pos.into(), kind: "word".into(), level, ..Default::default() };
    let a = ranker.rank(&mk("학교", "noun", Some(1)));
    let b = ranker.rank(&mk("먹다", "verb", Some(1)));
    let unknown = ranker.rank(&mk("낯선말", "noun", Some(1)));
    assert!(a < b && b < unknown);
    // stdict is always below krdict at equal frequency
    let mut s = mk("학교", "noun", None);
    s.source = "stdict";
    assert!(ranker.rank(&s) > a);
}

#[test]
fn enwikt_parsing() {
    use kdict_pipeline::enwikt;
    let k = enwikt::parse(BufReader::new(std::fs::File::open(fx("kaikki_en_sample.jsonl")).unwrap())).unwrap();
    let rows = |t: &str, pos: &str| -> Vec<(Option<String>, String, Option<String>)> {
        k.rows.iter().filter(|r| r.term_norm == t && r.pos == pos).map(|r| (r.sense.clone(), r.ko.clone(), r.roman.clone())).collect()
    };
    // per-sense items with their own sense label; hanja in brackets dropped; duplicate + Latin + Japanese dropped;
    // a sense without a label falls back to the sense's first gloss; "translations to be checked" items are skipped
    let sense1 = Some("information describing events".to_string());
    assert_eq!(
        rows("report", "noun"),
        vec![
            (sense1.clone(), "보고".into(), Some("bogo".into())),
            (sense1, "보고서".into(), Some("bogoseo".into())),
            (Some("A sharp, loud noise as from an explosion".into()), "폭음".into(), Some("pogeum".into())),
        ]
    );
    // top-level translations; `lang` without `lang_code` still counts
    let verb = rows("report", "verb");
    assert_eq!(verb.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["보고하다", "알리다", "신고하다"]);
    assert_eq!(verb[2].0.as_deref(), Some("to notify authorities"));
    // ranks run on across the lines of one term
    let ranks: Vec<i64> = k.rows.iter().filter(|r| r.term_norm == "report").map(|r| r.rank).collect();
    assert_eq!(ranks, (0..6).collect::<Vec<_>>());
    assert_eq!(rows("report card", "noun").len(), 2);
    assert_eq!(rows("report sick", "verb")[0].1, "병가를 내다");
    // proper noun keeps its capitalised display form; non-English lines and lines without Korean are skipped
    let korea = k.rows.iter().find(|r| r.term_norm == "korea").unwrap();
    assert_eq!((korea.term.as_deref(), korea.pos.as_str()), (Some("Korea"), "proper noun"));
    assert!(k.rows.iter().all(|r| r.term_norm != "rapport" && r.term_norm != "reporter"));
    assert!(k.rows.iter().all(|r| r.term.is_none() || r.term.as_deref() != Some(r.term_norm.as_str())));
    assert_eq!(k.rows.len(), 11);
    assert_eq!(k.dropped, 2); // duplicate 보고서, Latin "bang"
    // helpers
    assert_eq!(enwikt::clean_ko_word("보고(報告)"), Some("보고".into()));
    assert_eq!(enwikt::clean_ko_word("(을) 보고하다"), Some("보고하다".into()));
    assert_eq!(enwikt::clean_ko_word("報告"), None);
    assert_eq!(enwikt::clean_ko_word("TV 보고"), None);
    assert_eq!(enwikt::phrase_words("file a report"), ["file".to_string(), "report".to_string()][1..].to_vec());
    assert_eq!(enwikt::phrase_words("report card"), vec!["card".to_string()]);
    assert!(enwikt::mentions_korean(br#"{"lang_code": "ko"}"#) && !enwikt::mentions_korean(br#"{"lang_code": "ja"}"#));
    // fetch keeps only the lines that mention Korean
    let mut out = Vec::new();
    let (n, kept) = kdict_pipeline::fetch::filter_lines(&b"{\"a\": \"Korean\"}\n{\"a\": \"Japanese\"}\n{\"b\": \"ko\"}"[..], &mut out, enwikt::mentions_korean).unwrap();
    assert_eq!((n, kept), (3, 2));
    assert_eq!(out, b"{\"a\": \"Korean\"}\n{\"b\": \"ko\"}\n");
}

#[test]
fn kaikki_parsing() {
    let k = kaikki::parse(BufReader::new(std::fs::File::open(fx("kaikki_sample.jsonl")).unwrap())).unwrap();
    let by = |w: &str, h: Option<i64>| k.entries.iter().find(|e| e.headword == w && e.homonym == h).unwrap_or_else(|| panic!("{w}"));
    let hak = by("학교", None);
    assert_eq!(hak.hanja.as_deref(), Some("學校"));
    assert_eq!(hak.data["senses"][0]["gloss"], "school");
    assert_eq!(hak.data["senses"][0]["examples"][0]["en"], "I go to school.");
    assert_eq!(hak.data["senses"][0]["examples"][1]["en"], "The school is far."); // 'translation' key
    assert_eq!(hak.data["etym"], "Sino-Korean word from 學校.");
    assert!(hak.data["related"].as_array().unwrap().iter().any(|r| r["type"] == "derived" && r["word"] == "초등학교"));
    assert!(hak.forms.is_empty()); // romanisation is not a form
    // homonyms by etymology number; nested glosses use the last element
    assert_eq!(by("먹다", Some(1)).data["senses"][1]["gloss"], "to consume");
    assert_eq!(by("먹다", Some(2)).data["senses"][0]["gloss"], "to go deaf");
    // multiple POS merged into one entry with per-sense pos
    let ga = by("가다", None);
    assert_eq!(ga.data["senses"].as_array().unwrap().len(), 2);
    assert_eq!(ga.data["senses"][1]["pos"], "noun");
    // pure form-of senses are skipped but recorded
    assert!(k.entries.iter().all(|e| e.headword != "먹었다" && e.headword != "새로" && e.headword != "學" && e.headword != "one"));
    assert!(k.redirects.contains(&("먹었다".to_string(), "먹다".to_string())));
    assert!(k.redirects.contains(&("새로".to_string(), "새로이".to_string())));
    // example translations feed the sentence list
    assert_eq!(k.sentences.len(), 2);
    assert_eq!(k.sentences[0], ("학교에 가요.".to_string(), "I go to school.".to_string()));
}

#[test]
fn tatoeba_parsing() {
    let kor = std::fs::File::open(fx("tatoeba_kor.tsv")).unwrap();
    let eng = std::fs::File::open(fx("tatoeba_eng.tsv")).unwrap();
    let links = std::fs::read(fx("tatoeba_links.csv")).unwrap();
    let pairs = tatoeba::pairs(kor, eng, |cb| tatoeba::iter_links(Cursor::new(links), |a, b| cb(a, b))).unwrap();
    let got: Vec<(&str, &str)> = pairs.iter().map(|p| (p.ko.as_str(), p.en.as_str())).collect();
    // id 2 links to 999 (missing) then 102; id 3 has no English; id 4 has no hangul
    assert_eq!(got, vec![("학교에 가요.", "I go to school."), ("저는 책을 읽어요.", "I read a book.")]);
}

fn make_tatoeba_files(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let bz = |name: &str, data: Vec<u8>| {
        let p = dir.join(name);
        let mut e = bzip2::write::BzEncoder::new(std::fs::File::create(&p).unwrap(), bzip2::Compression::fast());
        e.write_all(&data).unwrap();
        e.finish().unwrap();
        p
    };
    std::fs::create_dir_all(dir).unwrap();
    let kor = bz("kor_sentences.tsv.bz2", std::fs::read(fx("tatoeba_kor.tsv")).unwrap());
    let eng = bz("eng_sentences.tsv.bz2", std::fs::read(fx("tatoeba_eng.tsv")).unwrap());
    let csv = std::fs::read(fx("tatoeba_links.csv")).unwrap();
    let mut tarb = tar::Builder::new(Vec::new());
    let mut h = tar::Header::new_gnu();
    h.set_size(csv.len() as u64);
    h.set_mode(0o644);
    h.set_cksum();
    tarb.append_data(&mut h, "links.csv", &csv[..]).unwrap();
    let links = bz("links.tar.bz2", tarb.into_inner().unwrap());
    (kor, eng, links)
}

#[test]
fn unihan_parsing() {
    let rd = |n: &str| BufReader::new(std::fs::File::open(fx(n)).unwrap());
    let m = unihan::parse_readers(vec![rd("Unihan_Readings.txt"), rd("Unihan_IRGSources.txt")]);
    let hak = &m[&'學'];
    assert_eq!(hak.readings, vec!["학"]);
    assert_eq!(hak.meaning_en.as_deref(), Some("learning; knowledge, school"));
    assert_eq!(hak.strokes, Some(16));
    assert_eq!(hak.radical_num, Some(39));
    assert_eq!(hak.radical.as_deref(), Some("\u{2F26}"));
    assert_eq!(m[&'校'].readings, vec!["교", "효"]);
    assert_eq!(m[&'一'].readings, vec!["il"]); // Yale fallback from kKorean
    assert!(!m[&'一'].has_hangul);
    assert_eq!(m[&'\u{3400}'].radical_num, Some(1)); // "1'.2"
}

#[test]
fn pack_chunking_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("x.sqlite");
    // incompressible-ish data so that several chunks are needed
    let mut data = Vec::new();
    let mut x: u32 = 12345;
    for _ in 0..200_000 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        data.push((x >> 24) as u8);
    }
    std::fs::write(&db, &data).unwrap();
    let v = pack::compress_pack("x", true, &db, dir.path(), &serde_json::json!({"entries": 1}), 50_000).unwrap();
    let chunks = v["chunks"].as_array().unwrap();
    assert!(chunks.len() >= 4);
    let mut gz = Vec::new();
    for (i, c) in chunks.iter().enumerate() {
        assert_eq!(c["file"], format!("x.sqlite.gz.{i:03}"));
        let b = std::fs::read(dir.path().join(c["file"].as_str().unwrap())).unwrap();
        assert!(b.len() <= 50_000);
        assert_eq!(b.len() as u64, c["bytes"].as_u64().unwrap());
        gz.extend(b);
    }
    assert_eq!(gz.len() as u64, v["gz_bytes"].as_u64().unwrap());
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(&gz[..]), &mut out).unwrap();
    assert_eq!(out, data);
    assert_eq!(v["bytes"], data.len());
}

fn q<T: rusqlite::types::FromSql>(c: &Connection, sql: &str) -> T {
    c.query_row(sql, [], |r| r.get(0)).unwrap_or_else(|e| panic!("{sql}: {e}"))
}

#[test]
fn end_to_end_mini_build() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("work");
    let (kor, eng, links) = make_tatoeba_files(&work);
    // Unihan.zip
    let zp = work.join("Unihan.zip");
    {
        let mut zw = zip::ZipWriter::new(std::fs::File::create(&zp).unwrap());
        for n in ["Unihan_Readings.txt", "Unihan_IRGSources.txt"] {
            zw.start_file(n, zip::write::SimpleFileOptions::default()).unwrap();
            zw.write_all(&std::fs::read(fx(n)).unwrap()).unwrap();
        }
        zw.finish().unwrap();
    }
    let src = Sources {
        krdict: vec![fx("krdict_sample.xml")],
        stdict: vec![fx("stdict_sample.xml")],
        opendict: vec![fx("opendict_sample.xml")],
        kaikki: Some(fx("kaikki_sample.jsonl")),
        kengdic: Some(fx("kengdic_sample.tsv")),
        freq: Some(fx("ko_freq.txt")),
        tatoeba: Some((kor, eng, links)),
        unihan: Some(zp),
        cedict: Some(fx("cedict_sample.u8")),
        zhwikt: Some(fx("zhwikt_sample.jsonl")),
        enwikt: Some(fx("kaikki_en_sample.jsonl")),
    };
    let out = dir.path().join("out");
    let manifest = build::run(&BuildOpts { out: out.clone(), sources: src, chunk_bytes: 20_000_000, allow_partial: true, texts: None }).unwrap();

    // --- manifest
    let packs = manifest["packs"].as_array().unwrap();
    assert_eq!(packs.len(), 5);
    assert_eq!((packs[2]["id"].as_str(), packs[2]["required"].as_bool()), (Some("opendict"), Some(false)));
    assert_eq!((packs[0]["id"].as_str(), packs[0]["required"].as_bool()), (Some("core"), Some(true)));
    assert_eq!((packs[1]["id"].as_str(), packs[1]["required"].as_bool()), (Some("stdict"), Some(false)));
    assert_eq!((packs[3]["id"].as_str(), packs[3]["required"].as_bool()), (Some("cedict"), Some(false)));
    assert_eq!((packs[4]["id"].as_str(), packs[4]["required"].as_bool()), (Some("zhwikt"), Some(false)));
    let ver = manifest["version"].as_str().unwrap();
    assert_eq!(ver.len(), 13);
    assert_eq!(ver.as_bytes()[8], b'-');
    let site = out.join("site-data");
    assert!(site.join("manifest.json").exists());
    assert!(std::fs::read_to_string(site.join("ATTRIBUTION.md")).unwrap().contains("krdict"));
    assert!(site.join("core.sqlite.gz.000").exists() && site.join("stdict.sqlite.gz.000").exists());
    // the gzip stream decompresses to the DB with the advertised sha256
    {
        use sha2::{Digest, Sha256};
        let gz = std::fs::read(site.join("core.sqlite.gz.000")).unwrap();
        let mut raw = Vec::new();
        std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(&gz[..]), &mut raw).unwrap();
        let hex: String = Sha256::digest(&raw).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, packs[0]["sha256"].as_str().unwrap());
        assert_eq!(raw.len() as u64, packs[0]["bytes"].as_u64().unwrap());
    }

    // --- core schema + queries
    let c = Connection::open(out.join("core.sqlite")).unwrap();
    assert_eq!(q::<i64>(&c, "PRAGMA page_size"), 4096);
    assert_eq!(q::<String>(&c, "PRAGMA journal_mode"), "delete");
    for t in ["meta", "entries", "forms", "hanja_words", "entries_fts", "hanja_chars", "sentences", "sentences_fts", "grammar", "gloss_terms", "wotd", "entries_hw_rank"] {
        assert_eq!(q::<i64>(&c, &format!("SELECT COUNT(*) FROM sqlite_master WHERE name='{t}'")), 1, "table {t}");
    }
    // gloss_terms: one row per (normalised gloss item, entry); score orders results within a term
    assert_eq!(q::<i64>(&c, "SELECT tier FROM gloss_terms g JOIN entries e ON e.id=g.entry_id WHERE g.term='school' AND e.source='krdict'"), 0);
    assert_eq!(q::<i64>(&c, "SELECT score FROM gloss_terms g JOIN entries e ON e.id=g.entry_id WHERE g.term='school' AND e.source='krdict'"), q::<i64>(&c, "SELECT rank FROM entries WHERE hw_norm='학교' AND source='krdict'"));
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM gloss_terms WHERE term LIKE 'to %' OR term != lower(trim(term)) OR length(term) > 40"), 0);
    // "eat; smoke" on a later sense: 'smoke' is a tier-1 item, 'eat' of the first sense tier 0
    assert!(q::<i64>(&c, "SELECT COUNT(*) FROM gloss_terms WHERE term='smoke' AND tier=1") >= 1);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM (SELECT term, entry_id FROM gloss_terms GROUP BY term, entry_id HAVING COUNT(*) > 1)"), 0);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM wotd"), q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE source='krdict' AND level IN (1,2) AND kind='word' AND gloss IS NOT NULL AND gloss != ''"));
    assert_eq!(q::<i64>(&c, "SELECT MAX(n) + 1 FROM wotd"), q::<i64>(&c, "SELECT COUNT(*) FROM wotd"));
    // en_ko (English Wiktionary translation tables), only with the English extract
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM sqlite_master WHERE name='en_ko_term'"), 1);
    assert_eq!(q::<String>(&c, "SELECT group_concat(ko, ',') FROM (SELECT ko FROM en_ko WHERE term_norm='report' AND pos='noun' ORDER BY rank)"), "보고,보고서,폭음");
    assert_eq!(q::<String>(&c, "SELECT group_concat(term_norm, ',') FROM (SELECT term_norm FROM en_ko_words WHERE word='report' ORDER BY term_norm)"), "file a report");
    assert_eq!(q::<String>(&c, "SELECT term FROM en_ko WHERE term_norm='korea'"), "Korea");
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM en_ko WHERE term IS NOT NULL AND term = term_norm"), 0);
    let core_counts: serde_json::Value = serde_json::from_str(&q::<String>(&c, "SELECT value FROM meta WHERE key='counts'")).unwrap();
    assert_eq!(core_counts["en_ko"].as_i64(), Some(11));
    assert!(core_counts["en_ko_gz_bytes"].as_i64().unwrap() > 0);
    assert!(q::<String>(&c, "SELECT value FROM meta WHERE key='sources'").contains("enwikt"));
    // covering-index plans for the hot queries
    let plan = |sql: &str| -> String {
        let mut st = c.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows: Vec<String> = st.query_map([], |r| r.get::<_, String>(3)).unwrap().map(|r| r.unwrap()).collect();
        rows.join(" | ")
    };
    let p = plan("SELECT id FROM entries INDEXED BY entries_hw_rank WHERE hw_norm >= '가' AND hw_norm < '각' ORDER BY rank LIMIT 50");
    assert!(p.contains("USING COVERING INDEX entries_hw_rank"), "{p}");
    let p = plan("SELECT id FROM entries WHERE hw_norm = '가' ORDER BY rank LIMIT 50");
    assert!(p.contains("USING COVERING INDEX entries_hw_rank") && !p.contains("TEMP B-TREE"), "{p}");
    let p = plan("SELECT tier, score, entry_id FROM gloss_terms WHERE term = 'eat' ORDER BY score LIMIT 60");
    assert!(p.contains("PRIMARY KEY") && !p.contains("TEMP B-TREE"), "{p}");
    assert!(q::<String>(&c, "SELECT sql FROM sqlite_master WHERE name='entries_fts'").contains("content=''"));
    for k in ["pack", "version", "built_at", "counts", "sources"] {
        assert_eq!(q::<i64>(&c, &format!("SELECT COUNT(*) FROM meta WHERE key='{k}'")), 1, "meta {k}");
    }
    assert_eq!(q::<String>(&c, "SELECT value FROM meta WHERE key='pack'"), "core");
    let counts: Value = serde_json::from_str(&q::<String>(&c, "SELECT value FROM meta WHERE key='counts'")).unwrap();
    assert_eq!(counts["krdict"], 6); // pointer entry dropped

    // exact lookup, best first
    let (hw, hanja, pos, level, gloss): (String, String, String, i64, String) = c
        .query_row("SELECT headword,hanja,pos,level,gloss FROM entries WHERE hw_norm='학교' AND source='krdict'", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap();
    assert_eq!((hw.as_str(), hanja.as_str(), pos.as_str(), level, gloss.as_str()), ("학교", "學校", "noun", 1, "school"));
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE hw_norm='학교'"), 2); // krdict + wikt (kengdic duplicate skipped)
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE source='kengdic' AND hw_norm IN ('학교','먹다')"), 0);
    assert_eq!(q::<String>(&c, "SELECT gloss FROM entries WHERE source='kengdic' AND hw_norm='서울'"), "Seoul");
    assert_eq!(q::<i64>(&c, "SELECT quality FROM entries WHERE source='krdict' AND hw_norm='학교'"), 0);
    assert_eq!(q::<i64>(&c, "SELECT quality FROM entries WHERE source='kengdic' AND kind='phrase' LIMIT 1"), 4);
    // the pointer entry is not an entry; its forms point at 먹다 homonym 2 (9 / 없다 do not exist)
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE headword='먹-'"), 0);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form IN ('먹고','먹는데','먹') AND e.hw_norm='먹다' AND e.homonym=2 AND e.source='krdict'"), 3);
    assert_eq!(q::<String>(&c, "SELECT source FROM entries WHERE hw_norm='학교' ORDER BY rank LIMIT 1"), "krdict");
    // grammar entry lookup by normalised key
    assert_eq!(q::<String>(&c, "SELECT headword FROM entries WHERE hw_norm='아서'"), "-아서");
    // conjugated forms (krdict 활용 + wikt form-of redirect)
    assert_eq!(
        q::<i64>(&c, "SELECT COUNT(*) FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form='먹어' AND e.hw_norm='먹다' AND e.source='krdict'"),
        1
    );
    assert!(q::<i64>(&c, "SELECT COUNT(*) FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form='먹었다' AND e.source='wikt'") >= 1);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM forms WHERE form='학교'"), 0); // headword itself is not a form
    // FTS: contentless table with explicit rowid, bm25 then rank
    let top: String = q(
        &c,
        "SELECT e.headword FROM entries_fts f JOIN entries e ON e.id=f.rowid WHERE entries_fts MATCH 'school' ORDER BY bm25(entries_fts), e.rank LIMIT 1",
    );
    assert_eq!(top, "학교");
    assert!(q::<i64>(&c, "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'head:eating OR head:eats'") >= 1); // porter stemming
    assert!(q::<i64>(&c, "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'en:teachers'") >= 1); // definitions are searchable too
    // hanja
    assert!(q::<i64>(&c, "SELECT COUNT(*) FROM hanja_words WHERE ch='學'") >= 2);
    let (rd, mean, strokes, rad, wc): (String, String, i64, String, i64) = c
        .query_row("SELECT readings,meaning_en,strokes,radical,word_count FROM hanja_chars WHERE ch='學'", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap();
    assert_eq!((rd.as_str(), strokes, wc), ("학", 16, 1));
    assert!(mean.contains("school") && !rad.is_empty());
    // kengdic hanja fallback: 고기 has exactly one hanja candidate but is not in krdict -> nothing to fill; no crash
    // sentences: tatoeba + wikt, deduped; trigram search works
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM sentences WHERE source='tatoeba'"), 2);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM sentences WHERE source='wikt'"), 1); // 학교에 가요. already from tatoeba
    assert_eq!(q::<String>(&c, "SELECT en FROM sentences s JOIN sentences_fts f ON f.rowid=s.id WHERE sentences_fts MATCH '\"책을 읽\"'"), "I read a book.");
    // grammar table
    let cats: Vec<(String, i64)> = {
        let mut st = c.prepare("SELECT category, COUNT(*) FROM grammar GROUP BY category ORDER BY category").unwrap();
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect()
    };
    assert_eq!(cats, vec![("Connective endings".to_string(), 1), ("Particles".to_string(), 1)]);
    assert_eq!(q::<String>(&c, "SELECT pattern FROM grammar WHERE category='Connective endings'"), "-아서");
    // entry data is valid JSON with senses
    let d: Value = serde_json::from_str(&q::<String>(&c, "SELECT data FROM entries WHERE hw_norm='먹다' AND source='krdict'")).unwrap();
    assert_eq!(d["senses"].as_array().unwrap().len(), 2);
    // gloss length contract
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE length(gloss) > 120"), 0);
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM entries WHERE source='stdict'"), 0);

    // --- stdict pack
    let s = Connection::open(out.join("stdict.sqlite")).unwrap();
    assert_eq!(q::<i64>(&s, "SELECT COUNT(*) FROM entries"), 4);
    assert_eq!(q::<String>(&s, "SELECT headword FROM entries WHERE hw_norm='가는길'"), "가 는 길");
    assert_eq!(q::<i64>(&s, "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('entries_fts','sentences','grammar','hanja_chars')"), 0);
    assert_eq!(q::<i64>(&s, "SELECT COUNT(*) FROM forms WHERE form='학교가'"), 1);
    assert!(q::<i64>(&s, "SELECT COUNT(*) FROM hanja_words WHERE ch='加'") == 1);
    assert_eq!(q::<String>(&s, "SELECT value FROM meta WHERE key='pack'"), "stdict");
    assert_eq!(q::<String>(&s, "SELECT lang FROM entries LIMIT 1"), "ko");

    // --- opendict pack: 학교 (noun) / 가는길 (phrase) duplicate stdict and are skipped; sourced example never stored
    let o = Connection::open(out.join("opendict.sqlite")).unwrap();
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM entries"), 2); // 학교 + 가는 길 are stdict duplicates
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM entries WHERE hw_norm='학교'"), 0);
    assert_eq!(q::<String>(&o, "SELECT headword FROM entries WHERE hw_norm='핵교'"), "핵교");
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM entries WHERE data LIKE '%어느 신문%' OR data LIKE '%신문에서 인용%'"), 0);
    assert_eq!(q::<String>(&o, "SELECT value FROM meta WHERE key='pack'"), "opendict");
    let oc: Value = serde_json::from_str(&q::<String>(&o, "SELECT value FROM meta WHERE key='counts'")).unwrap();
    assert_eq!(oc["skipped_stdict_duplicates"], 2);
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('entries_fts','sentences','grammar','hanja_chars')"), 0);
}

#[test]
fn mini_build_without_optional_sources() {
    let dir = tempfile::tempdir().unwrap();
    let src = Sources { krdict: vec![fx("krdict_sample.xml")], ..Default::default() };
    let out = dir.path().join("out");
    let m = build::run(&BuildOpts { out: out.clone(), sources: src, chunk_bytes: 20_000_000, allow_partial: true, texts: None }).unwrap();
    assert_eq!(m["packs"].as_array().unwrap().len(), 1);
    assert!(!out.join("site-data/stdict.sqlite.gz.000").exists());
}

#[test]
fn bundled_sqlite_has_fts5_tokenizers() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE VIRTUAL TABLE a USING fts5(x, tokenize='trigram'); CREATE VIRTUAL TABLE b USING fts5(x, tokenize='porter unicode61');").unwrap();
    println!("bundled SQLite {}", rusqlite::version());
}

#[test]
fn xml_file_missing_is_error() {
    assert!(for_each_file(Path::new("/nonexistent/x.xml"), "a", |_| Ok(())).is_err());
}

#[test]
fn stamp_format() {
    let (v, b) = build::stamp(1_700_000_000);
    assert_eq!(v, "20231114-2213");
    assert_eq!(b, "2023-11-14T22:13:20Z");
}

// --- older-text packs: 훈음, CC-CEDICT, Wiktionary Chinese, 옛말 flag -------------------------

#[test]
fn kaikki_hanja_hun_eum() {
    let k = kaikki::parse(BufReader::new(std::fs::File::open(fx("kaikki_sample.jsonl")).unwrap())).unwrap();
    // {{ko-hanja|배울|학}} args; the second line of 學 would only add a reading
    let h = &k.hanja[&'學'];
    assert_eq!(h.hun, vec!["배울"]);
    assert_eq!(h.eumhun, vec!["배울 학"]);
    assert_eq!(h.eum, vec!["학"]);
    // no args: eumhun taken from the template expansion text
    assert_eq!(k.hanja[&'校'].eumhun, vec!["학교 교"]);
    // two etymology lines: arg pair + forms[] tagged eumhun, joined later with "; "
    let le = &k.hanja[&'樂'];
    assert_eq!(le.eumhun, vec!["즐길 락", "풍류 악"]);
    assert_eq!(le.hun, vec!["즐길", "풍류"]);
    // a bare "생 (saeng): life" gloss yields a reading only
    assert_eq!(k.hanja[&'生'].eum, vec!["생"]);
    assert!(k.hanja[&'生'].hun.is_empty());
    assert!(!k.hanja.contains_key(&'木'));
    assert!(k.entries.iter().all(|e| !has_cjk(&e.headword)));
    assert_eq!(kaikki::split_eumhun("배울 학 (baeul hak)"), Some(("배울".into(), "학".into())));
    assert_eq!(kaikki::split_eumhun("hello world"), None);
    assert_eq!(kaikki::split_eumhun("학"), None);
}

#[test]
fn cedict_pinyin_and_sino_korean() {
    assert_eq!(cedict::pinyin_marked("xue2 xiao4"), "xué xiào");
    assert_eq!(cedict::pinyin_marked("nu:3 zi3"), "nǚ zǐ");
    assert_eq!(cedict::pinyin_marked("lu:e4 liu2 gui1 hui4"), "lüè liú guī huì");
    assert_eq!(cedict::pinyin_marked("hua1 r5"), "huā r");
    assert_eq!(cedict::pinyin_marked("xue2 sheng5"), "xué sheng");
    assert_eq!(cedict::pinyin_marked("Zhong1 guo2 ou1 A1 Q"), "Zhōng guó ōu Ā Q");
    // 두음법칙 + 렬/률
    assert_eq!(cedict::dueum("녀자"), "여자");
    assert_eq!(cedict::dueum("리유"), "이유");
    assert_eq!(cedict::dueum("로사"), "노사");
    assert_eq!(cedict::dueum("례의"), "예의");
    assert_eq!(cedict::dueum("규률"), "규율");
    assert_eq!(cedict::dueum("비률"), "비율");
    assert_eq!(cedict::dueum("학교"), "학교");
    assert_eq!(cedict::dueum("소녀"), "소녀");
    assert_eq!(cedict::dueum("선률"), "선율");
    let r: std::collections::HashMap<char, String> = [('學', "학"), ('校', "교"), ('女', "녀"), ('子', "자")].iter().map(|(c, s)| (*c, s.to_string())).collect();
    assert_eq!(cedict::sino_korean("學校", &r).as_deref(), Some("학교"));
    assert_eq!(cedict::sino_korean("女子", &r).as_deref(), Some("여자"));
    assert_eq!(cedict::sino_korean("學A", &r), None); // non-hanja
    assert_eq!(cedict::sino_korean("學生", &r), None); // unknown reading
}

#[test]
fn cedict_parsing() {
    let sino: std::collections::HashMap<char, String> = [('學', "학"), ('校', "교")].iter().map(|(c, s)| (*c, s.to_string())).collect();
    let mut es = Vec::new();
    cedict::parse_file(&fx("cedict_sample.u8"), &sino, |e| {
        es.push(e);
        Ok(())
    })
    .unwrap();
    assert_eq!(es.len(), 11); // comment lines skipped
    let x = &es[0];
    assert_eq!((x.headword.as_str(), x.hanja.as_deref(), x.source, x.lang), ("學校", Some("學校"), "cedict", "en"));
    assert_eq!(x.pron.as_deref(), Some("학교"));
    assert_eq!(x.forms, vec!["学校"]);
    assert_eq!(x.data["simplified"], "学校");
    assert_eq!(x.data["pinyin"], "xué xiào");
    assert_eq!(x.data["pinyin_num"], "xue2 xiao4");
    assert_eq!(x.data["senses"].as_array().unwrap().len(), 1);
    assert_eq!(x.data["senses"][0]["gloss"], "school");
    assert_eq!(x.data["cl"], serde_json::json!(["家[jia1]", "所[suo3]"]));
    // two readings of 校 are two entries, same headword
    assert_eq!(es.iter().filter(|e| e.headword == "校").count(), 2);
    assert_eq!(es[1].data["senses"].as_array().unwrap().len(), 4);
    assert_eq!(es[1].forms, vec!["学"]);
    assert!(es[2].forms.is_empty()); // 校: simplified == traditional
    assert!(es[2].data.get("simplified").is_none());
    assert_eq!(es[2].data["senses"].as_array().unwrap().len(), 2); // CL: is not a sense
    assert!(es.iter().find(|e| e.headword == "阿Q").unwrap().pron.is_none());
    // CEDICT zip container
    let dir = tempfile::tempdir().unwrap();
    let zp = dir.path().join("cedict.zip");
    {
        let mut zw = zip::ZipWriter::new(std::fs::File::create(&zp).unwrap());
        zw.start_file("cedict_ts.u8", zip::write::SimpleFileOptions::default()).unwrap();
        zw.write_all(&std::fs::read(fx("cedict_sample.u8")).unwrap()).unwrap();
        zw.finish().unwrap();
    }
    let mut n = 0;
    cedict::parse_file(&zp, &sino, |_| {
        n += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(n, 11);
}

#[test]
fn zhwikt_parsing() {
    let sino: std::collections::HashMap<char, String> = [('女', "녀"), ('子', "자")].iter().map(|(c, s)| (*c, s.to_string())).collect();
    let mut es = Vec::new();
    let redirects = zhwikt::parse(BufReader::new(std::fs::File::open(fx("zhwikt_sample.jsonl")).unwrap()), &sino, |e, rank| {
        es.push((e, rank));
        Ok(())
    })
    .unwrap();
    // 學校, 學 (two lines merged by etymology), 校, 女子; the simplified redirect lines and "hello" yield none
    let hw: Vec<&str> = es.iter().map(|(e, _)| e.headword.as_str()).collect();
    assert_eq!(hw, vec!["學校", "學", "校", "女子"]);
    assert!(redirects.contains(&("学校".to_string(), "學校".to_string())) && redirects.contains(&("学".to_string(), "學".to_string())));
    let (x, _) = &es[0];
    assert_eq!((x.source, x.lang, x.hanja.as_deref()), ("zhwikt", "en", Some("學校")));
    assert_eq!(x.data["simplified"], "学校");
    assert_eq!(x.forms, vec!["学校"]);
    assert_eq!(x.data["senses"][0]["gloss"], "school");
    assert!(x.data["senses"][0].get("examples").is_none() && x.data.get("translations").is_none());
    assert_eq!(x.data["pron"]["mandarin"], serde_json::json!(["xuéxiào"]));
    assert_eq!(x.data["pron"]["sino_korean"], serde_json::json!(["학교"]));
    assert_eq!(x.data["pron"]["middle_chinese"], serde_json::json!(["haewk gaewH"]));
    assert_eq!(x.data["pron"]["cantonese"], serde_json::json!(["hok6 haau6"]));
    assert_eq!(x.pron.as_deref(), Some("학교"));
    assert!(x.data["etym"].as_str().unwrap().starts_with("From 學"));
    let (g, _) = &es[1];
    assert_eq!(g.data["senses"].as_array().unwrap().len(), 3); // POS lines merged
    assert_eq!(g.data["senses"][2]["pos"], "verb");
    assert_eq!(g.data["classical"], true);
    assert_eq!(g.data["pron"]["middle_chinese"], serde_json::json!(["ɦɔk̚"]));
    assert_eq!(g.data["pron"]["sino_vietnamese"], serde_json::json!(["học"]));
    assert_eq!(g.pron.as_deref(), Some("학"));
    // Sino-Korean computed from the hanja readings when the entry has none (dueum applied)
    assert_eq!(es[3].0.pron.as_deref(), Some("여자"));
    // ranks: shorter headword first
    assert!(es[1].1 < es[0].1);
}

#[test]
fn historical_flag() {
    use serde_json::json;
    let mut e = Entry { source: "stdict", ..Default::default() };
    e.data = json!({"senses": [{"tags": ["옛말"], "ko_def": "x"}]});
    assert!(build::is_historical(&e));
    e.data = json!({"senses": [{"ko_def": "‘아무’의 옛말."}]});
    assert!(build::is_historical(&e));
    e.data = json!({"senses": [{"tags": ["옛말"]}, {"ko_def": "현대어 뜻."}]});
    assert!(!build::is_historical(&e)); // only entries that are old in every sense
    e.data = json!({"senses": []});
    assert!(!build::is_historical(&e));
    e.source = "krdict";
    e.data = json!({"senses": [{"tags": ["옛말"]}]});
    assert!(!build::is_historical(&e));
}

#[test]
fn quality_gate_optional_packs() {
    use serde_json::json;
    let ok = json!({"entries": 120000});
    let bad = json!({"entries": 10});
    assert!(build::quality_gate_optional(&[("cedict", &ok), ("zhwikt", &ok)]).is_empty());
    assert!(build::quality_gate_optional(&[]).is_empty()); // not built -> not gated
    let p = build::quality_gate_optional(&[("cedict", &bad), ("zhwikt", &ok)]);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("cedict.entries"));
}

#[test]
fn chinese_packs_are_built_from_fixtures() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("work");
    std::fs::create_dir_all(&work).unwrap();
    let zp = work.join("Unihan.zip");
    {
        let mut zw = zip::ZipWriter::new(std::fs::File::create(&zp).unwrap());
        for n in ["Unihan_Readings.txt", "Unihan_IRGSources.txt"] {
            zw.start_file(n, zip::write::SimpleFileOptions::default()).unwrap();
            zw.write_all(&std::fs::read(fx(n)).unwrap()).unwrap();
        }
        zw.finish().unwrap();
    }
    let src = Sources {
        krdict: vec![fx("krdict_sample.xml")],
        stdict: vec![fx("stdict_sample.xml")],
        opendict: vec![fx("opendict_sample.xml")],
        kaikki: Some(fx("kaikki_sample.jsonl")),
        unihan: Some(zp),
        cedict: Some(fx("cedict_sample.u8")),
        zhwikt: Some(fx("zhwikt_sample.jsonl")),
        ..Default::default()
    };
    let out = dir.path().join("out");
    build::run(&BuildOpts { out: out.clone(), sources: src, chunk_bytes: 20_000_000, allow_partial: true, texts: None }).unwrap();

    // core: 훈음 columns (Unihan reading kept, Wiktionary adds hun/eumhun)
    let c = Connection::open(out.join("core.sqlite")).unwrap();
    assert_eq!(q::<String>(&c, "SELECT hun FROM hanja_chars WHERE ch='學'"), "배울");
    assert_eq!(q::<String>(&c, "SELECT eumhun FROM hanja_chars WHERE ch='學'"), "배울 학");
    assert_eq!(q::<String>(&c, "SELECT readings FROM hanja_chars WHERE ch='學'"), "학");
    assert_eq!(q::<String>(&c, "SELECT eumhun FROM hanja_chars WHERE ch='樂'"), "즐길 락; 풍류 악");
    assert_eq!(q::<String>(&c, "SELECT readings FROM hanja_chars WHERE ch='樂'"), "락,악"); // Wiktionary readings (no Unihan row)
    assert_eq!(q::<String>(&c, "SELECT hun FROM hanja_chars WHERE ch='樂'"), "즐길");
    assert_eq!(q::<i64>(&c, "SELECT COUNT(*) FROM hanja_chars WHERE ch='校' AND hun IS NOT NULL"), 1);
    assert_eq!(q::<i64>(&c, "SELECT hist FROM entries LIMIT 1"), 0);

    // opendict: 옛말 entry flagged and indexed
    let o = Connection::open(out.join("opendict.sqlite")).unwrap();
    assert!(q::<i64>(&o, "SELECT COUNT(*) FROM entries WHERE hist = 1") >= 1);
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM sqlite_master WHERE name = 'entries_hist'"), 1);
    assert_eq!(q::<i64>(&o, "SELECT COUNT(*) FROM entries WHERE hist = 1 AND data NOT LIKE '%옛말%'"), 0);
    let s = Connection::open(out.join("stdict.sqlite")).unwrap();
    assert_eq!(q::<i64>(&s, "SELECT COUNT(*) FROM sqlite_master WHERE name = 'entries_hist'"), 1);

    // cedict: traditional headword, simplified via forms, Sino-Korean in pron
    let d = Connection::open(out.join("cedict.sqlite")).unwrap();
    // only Korean-attested words/characters are kept (the fixture Korean packs know 學校, 學, 校)
    let kept = |c: &Connection| -> Vec<String> { c.prepare("SELECT headword FROM entries ORDER BY id").unwrap().query_map([], |r| r.get(0)).unwrap().flatten().collect() };
    assert_eq!(kept(&d), ["學校", "學", "校", "校"]);
    assert_eq!(q::<i64>(&d, "SELECT json_extract(value, '$.dropped_not_korean') FROM meta WHERE key='counts'"), 7); // 女子 理由 老師 規律 學生 阿Q 花兒
    assert_eq!(q::<String>(&d, "SELECT pron FROM entries WHERE hw_norm='學校'"), "학교");
    assert_eq!(q::<String>(&d, "SELECT e.headword FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form='学校'"), "學校");
    assert_eq!(q::<String>(&d, "SELECT hanja FROM entries WHERE hw_norm='學校'"), "學校");
    assert_eq!(q::<i64>(&d, "SELECT COUNT(*) FROM hanja_words WHERE ch='校'"), 3);
    assert_eq!(q::<String>(&d, "SELECT value FROM meta WHERE key='pack'"), "cedict");
    assert_eq!(q::<i64>(&d, "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('entries_fts','gloss_terms','sentences')"), 0);

    // zhwikt: form-of redirects become forms of the target
    let z = Connection::open(out.join("zhwikt.sqlite")).unwrap();
    assert_eq!(kept(&z), ["學校", "學", "校"]); // "hello", 女子: not Korean hanja words in the fixtures
    assert_eq!(q::<String>(&z, "SELECT e.headword FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form='学校'"), "學校");
    assert_eq!(q::<String>(&z, "SELECT e.headword FROM forms f JOIN entries e ON e.id=f.entry_id WHERE f.form='学'"), "學");
    assert_eq!(q::<String>(&z, "SELECT value FROM meta WHERE key='pack'"), "zhwikt");
}

#[test]
fn chinese_sources_missing_means_no_pack() {
    let dir = tempfile::tempdir().unwrap();
    let src = Sources { krdict: vec![fx("krdict_sample.xml")], ..Default::default() };
    let out = dir.path().join("out");
    let m = build::run(&BuildOpts { out: out.clone(), sources: src, chunk_bytes: 20_000_000, allow_partial: true, texts: None }).unwrap();
    assert_eq!(m["packs"].as_array().unwrap().len(), 1);
    assert!(!out.join("cedict.sqlite").exists() && !out.join("zhwikt.sqlite").exists());
}

#[test]
fn quality_gate_en_ko_only_when_built() {
    use serde_json::json;
    let base = json!({"entries": 200000, "krdict": 60000, "wikt": 30000, "sentences": 20000, "hanja_chars": 6000});
    assert!(build::quality_gate(&base, None).is_empty());
    let mut thin = base.clone();
    thin["en_ko"] = json!(5);
    assert_eq!(build::quality_gate(&thin, None).len(), 1);
    let mut full = base.clone();
    full["en_ko"] = json!(200000);
    assert!(build::quality_gate(&full, None).is_empty());
}
