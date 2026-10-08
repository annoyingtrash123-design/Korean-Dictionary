//! `texts` pack build (Reader library): catalogue + raw + approved enrichment, with fixtures only.

use kdict_pipeline::build::quality_gate_optional;
use kdict_pipeline::schema;
use kdict_pipeline::texts_pack::{build_texts, TextsOpts};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn entry(id: &str, shelf: &str, script: &str, extra: &str) -> String {
    format!(
        r#"
[[text]]
id = "{id}"
title_ko = "제목 {id}"
title_en = "Title {id}"
author_ko = "작자 미상"
author_en = "Anonymous"
date = "19th century"
year = 1850
period = "joseon-late"
themes = ["joseon"]
shelf = "{shelf}"
script = "{script}"
source = "wikisource-ko"
source_title = "{id}"
pd_basis = "Anonymous pre-modern work."
{extra}
"#
    )
}

fn write(p: &Path, v: &Value) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, serde_json::to_string_pretty(v).unwrap()).unwrap();
}

fn raw(id: &str, text: &str) -> Value {
    json!({"id": id, "source": "wikisource-ko", "url": format!("https://ko.wikisource.org/wiki/{id}"), "page_title": id, "revision_id": 42,
        "revision_timestamp": "2024-05-01T10:00:00Z", "fetched_at": "2026-10-01T00:00:00Z", "licence": "CC BY-SA 4.0 / PD", "text": text, "edition": null})
}

fn enriched(id: &str, status: &str, origs: &[&str]) -> Value {
    json!({
        "id": id,
        "card": {"summary_ko": "요약", "summary_en": "Summary", "level": "classical", "edition_ko": "판본", "edition_en": "Edition"},
        "notes": {"ko": "노트", "en": "Notes"},
        "paragraphs": origs.iter().map(|o| json!({"orig": o, "modern": null, "reading": null, "en": format!("EN {o}")})).collect::<Vec<_>>(),
        "vocab": [{"word": "학교", "gloss_en": "school", "level": 3}, {"word": "갔다", "gloss_en": "went", "level": 1}],
        "labels": {"notes": "ai", "translation": "ai", "modern": null, "text": "original"},
        "review": {"status": status, "by": "content-review", "date": "2026-10-07", "notes": "ok"},
    })
}

fn graded(id: &str, words: &[&str]) -> Value {
    json!({
        "id": id,
        "meta": {"title_ko": "단군", "title_en": "Dangun", "author_ko": "AI", "author_en": "AI", "date": "2026", "year": -2333, "period": "Ancient",
                 "themes": ["Ancient & Goryeo"], "shelf": "graded", "script": "hangul", "excerpt": false, "source": "original", "pd_basis": "Original"},
        "card": {"summary_ko": "요", "summary_en": "S", "level": "TOPIK 2", "edition_ko": "e", "edition_en": "e"},
        "notes": {"ko": "n", "en": "n"},
        "paragraphs": [{"orig": "학교에 갔어요.", "modern": null, "reading": null, "en": "Went to school."}],
        "vocab": words.iter().map(|w| json!({"word": w, "gloss_en": "x", "level": 9})).collect::<Vec<_>>(),
        "questions": [{"q_ko": "?", "q_en": "?", "answer_en": "!"}],
        "labels": {"notes": "ai", "translation": "ai", "modern": null, "text": "ai"},
        "review": {"status": "approved", "by": "content-review", "date": "2026-10-07", "notes": "ok"},
    })
}

fn make_core(path: &Path) {
    let c = Connection::open(path).unwrap();
    c.execute_batch(schema::COMMON).unwrap();
    c.execute_batch(
        "INSERT INTO entries(id,headword,hw_norm,source,lang,level,rank,kind,data) VALUES
           (1,'학교','학교','krdict','ko',1,1,'word','{}'),
           (2,'가다','가다','krdict','ko',1,2,'word','{}'),
           (3,'옛말','옛말','stdict','ko',NULL,3,'word','{}');
         INSERT INTO forms(form, entry_id) VALUES ('갔다', 2);",
    )
    .unwrap();
}

struct Fx {
    dir: tempfile::TempDir,
}

impl Fx {
    fn new() -> Fx {
        let dir = tempfile::tempdir().unwrap();
        let t = dir.path().join("texts");
        let mut cat = String::from("[news]\nfeeds = []\n");
        cat += &entry("good", "classical-prose", "hangul", "");
        cat += &entry("poem", "verse", "hangul", "");
        cat += &entry(
            "excerpted",
            "classical-prose",
            "hangul",
            "excerpt = true\nexcerpt_note = \"opening and ending\"",
        );
        cat += &entry("unapproved", "classical-prose", "hangul", "");
        cat += &entry("noraw", "classical-prose", "hangul", "");
        fs::create_dir_all(&t).unwrap();
        fs::write(t.join("catalog.toml"), cat).unwrap();
        // every raw fixture text has been checked by a reviewer (original-only shipping allowed)
        fs::write(t.join("raw_review.toml"), "[approved]\nids = [\"good\", \"poem\", \"excerpted\", \"unapproved\", \"noraw\"]\n").unwrap();
        write(
            &t.join("raw/good.json"),
            &raw("good", "첫째 문단입니다.\n\n둘째   문단입니다.\n"),
        );
        write(
            &t.join("enriched/good.json"),
            &enriched(
                "good",
                "approved",
                &["첫째 문단입니다.", "둘째 문단입니다."],
            ),
        );
        write(
            &t.join("raw/poem.json"),
            &raw("poem", "가고 싶다\n나는 간다\n\n다시 온다\n"),
        );
        write(
            &t.join("raw/excerpted.json"),
            &raw("excerpted", "하나\n\n둘\n\n셋\n\n넷"),
        );
        write(
            &t.join("enriched/excerpted.json"),
            &enriched("excerpted", "approved", &["하나", "넷"]),
        );
        write(
            &t.join("raw/unapproved.json"),
            &raw("unapproved", "원문 하나\n\n원문 둘"),
        );
        write(
            &t.join("enriched/unapproved.json"),
            &enriched("unapproved", "draft", &["원문 하나", "원문 둘"]),
        );
        write(
            &t.join("enriched/noraw.json"),
            &enriched("noraw", "draft", &["x"]),
        );
        write(
            &t.join("enriched/graded-dangun.json"),
            &graded("graded-dangun", &["학교", "갔다"]),
        );
        write(
            &t.join("raw/news/news-1.json"),
            &json!({"id": "news-1", "source": "korea-kr", "url": "https://www.korea.kr/x?newsId=1", "title": "정책 뉴스", "published": "2026-10-01",
                    "fetched_at": "2026-10-02T00:00:00Z", "kogl_type": 1, "licence": "KOGL Type 1", "text": "첫 문단.\n\n둘째 문단."}),
        );
        make_core(&dir.path().join("core.sqlite"));
        let x = Connection::open(dir.path().join("extra.sqlite")).unwrap();
        x.execute_batch(schema::COMMON).unwrap();
        x.execute_batch("INSERT INTO entries(id,headword,hw_norm,source,lang,rank,kind,data) VALUES (1,'조공','조공','stdict','ko',1,'word','{}');").unwrap();
        Fx { dir }
    }
    fn t(&self) -> std::path::PathBuf {
        self.dir.path().join("texts")
    }
    fn build(
        &self,
        allow_partial: bool,
    ) -> anyhow::Result<Option<kdict_pipeline::texts_pack::TextsResult>> {
        let out = self.dir.path().join("out");
        fs::create_dir_all(&out).unwrap();
        let extra = [self.dir.path().join("extra.sqlite")];
        build_texts(&TextsOpts {
            extra: &extra,
            dir: &self.t(),
            core: &self.dir.path().join("core.sqlite"),
            out: &out,
            version: "t1",
            built_at: "now",
            allow_partial,
        })
    }
}

fn q(c: &Connection, sql: &str) -> Vec<String> {
    let mut st = c.prepare(sql).unwrap();
    let n = st.column_count();
    st.query_map([], |r| {
        Ok((0..n)
            .map(|i| match r.get::<_, rusqlite::types::Value>(i).unwrap() {
                rusqlite::types::Value::Null => "∅".to_string(),
                rusqlite::types::Value::Integer(n) => n.to_string(),
                rusqlite::types::Value::Text(t) => t,
                o => format!("{o:?}"),
            })
            .collect::<Vec<_>>()
            .join("|"))
    })
    .unwrap()
    .map(|r| r.unwrap())
    .collect()
}

#[test]
fn unreviewed_raw_texts_are_not_packed() {
    let fx = Fx::new();
    fs::write(fx.t().join("raw_review.toml"), "[approved]\nids = [\"poem\"]\n").unwrap();
    let r = fx.build(false).unwrap().unwrap();
    assert_eq!(r.counts["original_only"], 1); // poem only; 'unapproved' (raw, no approved enrichment) waits
    assert_eq!(r.counts["raw_unreviewed"], 1);
}

#[test]
fn builds_enriched_original_only_graded_and_news() {
    let fx = Fx::new();
    let r = fx.build(false).unwrap().unwrap();
    assert_eq!(r.counts["texts"], 6);
    assert_eq!(r.counts["enriched"], 2);
    assert_eq!(r.counts["original_only"], 2);
    assert_eq!(r.counts["graded"], 1);
    assert_eq!(r.counts["news"], 1);
    assert_eq!(r.counts["unapproved_ignored"], 2, "{}", r.counts);
    let c = Connection::open(&r.path).unwrap();

    // chronological sort: graded (-2333) first, news (2026) last
    assert_eq!(
        q(&c, "SELECT id FROM texts ORDER BY sort").first().unwrap(),
        "graded-dangun"
    );
    assert_eq!(
        q(&c, "SELECT id FROM texts ORDER BY sort").last().unwrap(),
        "news-1"
    );
    // text without raw and with a draft enrichment is absent
    assert!(q(&c, "SELECT id FROM texts WHERE id='noraw'").is_empty());

    // enriched text: paragraphs, en, vocab levels recomputed (3 -> 1 via headword, 1 via form)
    assert_eq!(
        q(
            &c,
            "SELECT orig, en FROM paragraphs WHERE text_id='good' ORDER BY n"
        ),
        vec![
            "첫째 문단입니다.|EN 첫째 문단입니다.",
            "둘째 문단입니다.|EN 둘째 문단입니다."
        ]
    );
    let vocab: Value =
        serde_json::from_str(&q(&c, "SELECT vocab FROM texts WHERE id='good'")[0]).unwrap();
    assert_eq!(vocab[0]["level"], 1);
    assert_eq!(vocab[1]["level"], 1);
    assert_eq!(q(&c, "SELECT chars FROM texts WHERE id='good'")[0], "18");
    let prov: Value =
        serde_json::from_str(&q(&c, "SELECT provenance FROM texts WHERE id='good'")[0]).unwrap();
    assert_eq!(prov["revision_id"], 42);
    assert_eq!(prov["url"], "https://ko.wikisource.org/wiki/good");

    // verse keeps line breaks; original-only has null en/modern and the minimal labels
    assert_eq!(
        q(
            &c,
            "SELECT orig FROM paragraphs WHERE text_id='poem' ORDER BY n"
        ),
        vec!["가고 싶다\n나는 간다", "다시 온다"]
    );
    assert_eq!(
        q(
            &c,
            "SELECT en, modern, reading FROM paragraphs WHERE text_id='poem' AND n=0"
        )[0],
        "∅|∅|∅"
    );
    assert_eq!(
        q(&c, "SELECT labels FROM texts WHERE id='poem'")[0],
        r#"{"text":"original"}"#
    );
    assert_eq!(
        q(&c, "SELECT level, vocab FROM texts WHERE id='poem'")[0],
        "classical|∅"
    );
    let card: Value =
        serde_json::from_str(&q(&c, "SELECT card FROM texts WHERE id='poem'")[0]).unwrap();
    assert!(card["edition_en"].as_str().unwrap().contains("revision 42"));
    // an unapproved enrichment is ignored: the raw text is packed as original-only
    assert_eq!(
        q(
            &c,
            "SELECT orig, en FROM paragraphs WHERE text_id='unapproved' ORDER BY n"
        ),
        vec!["원문 하나|∅", "원문 둘|∅"]
    );
    assert_eq!(
        q(
            &c,
            "SELECT json_extract(review,'$.status') FROM texts WHERE id='unapproved'"
        )[0],
        "original-only"
    );
    // excerpt: ordered selection of raw paragraphs is allowed
    assert_eq!(
        q(
            &c,
            "SELECT orig FROM paragraphs WHERE text_id='excerpted' ORDER BY n"
        ),
        vec!["하나", "넷"]
    );

    // graded reader: own meta, slugged period/theme, shelf graded, level recomputed, questions kept
    assert_eq!(
        q(
            &c,
            "SELECT shelf, period, year, level FROM texts WHERE id='graded-dangun'"
        )[0],
        "graded|ancient|-2333|TOPIK 2"
    );
    assert_eq!(
        q(
            &c,
            "SELECT json_extract(meta,'$.themes[0]') FROM texts WHERE id='graded-dangun'"
        )[0],
        "ancient-goryeo"
    );
    assert_eq!(
        q(
            &c,
            "SELECT json_extract(vocab,'$[0].level') FROM texts WHERE id='graded-dangun'"
        )[0],
        "1"
    );
    assert_eq!(
        q(&c, "SELECT count(*) FROM texts WHERE questions IS NOT NULL")[0],
        "1"
    );
    assert_eq!(
        q(
            &c,
            "SELECT json_extract(labels,'$.text') FROM texts WHERE id='graded-dangun'"
        )[0],
        "ai"
    );

    // news
    assert_eq!(
        q(
            &c,
            "SELECT shelf, period, year FROM texts WHERE id='news-1'"
        )[0],
        "news|modern|2026"
    );
    assert_eq!(
        q(&c, "SELECT review FROM texts WHERE id='news-1'")[0],
        r#"{"status":"approved","by":"source","notes":"KOGL Type 1 original"}"#
    );
    assert_eq!(
        q(&c, "SELECT labels FROM texts WHERE id='news-1'")[0],
        r#"{"text":"original"}"#
    );
    assert_eq!(
        q(&c, "SELECT count(*) FROM paragraphs WHERE text_id='news-1'")[0],
        "2"
    );
    assert_eq!(q(&c, "SELECT value FROM meta WHERE key='pack'")[0], "texts");
}

#[test]
fn paragraph_mismatch_fails_unless_partial() {
    let fx = Fx::new();
    write(
        &fx.t().join("enriched/good.json"),
        &enriched(
            "good",
            "approved",
            &["첫째 문단입니다.", "둘째 문단이 바뀌었습니다."],
        ),
    );
    let err = format!("{:#}", fx.build(false).unwrap_err());
    assert!(
        err.contains("good") && err.contains("does not match the raw source"),
        "{err}"
    );
    // partial: falls back to the original text
    let r = fx.build(true).unwrap().unwrap();
    let c = Connection::open(&r.path).unwrap();
    assert_eq!(
        q(
            &c,
            "SELECT orig, en FROM paragraphs WHERE text_id='good' AND n=1"
        )[0],
        "둘째   문단입니다.|∅"
    );
}

#[test]
fn missing_paragraph_in_non_excerpt_fails() {
    let fx = Fx::new();
    write(
        &fx.t().join("enriched/good.json"),
        &enriched("good", "approved", &["첫째 문단입니다."]),
    );
    assert!(fx.build(false).is_err());
}

#[test]
fn unresolved_vocab_fails_listing_words() {
    let fx = Fx::new();
    write(
        &fx.t().join("enriched/graded-dangun.json"),
        &graded("graded-dangun", &["학교", "조공", "없는말", "또없다"]),
    );
    let err = format!("{:#}", fx.build(false).unwrap_err());
    assert!(
        err.contains("graded-dangun") && err.contains("없는말, 또없다"),
        "{err}"
    );
    let r = fx.build(true).unwrap().unwrap();
    let c = Connection::open(&r.path).unwrap();
    assert_eq!(
        q(
            &c,
            "SELECT json_extract(vocab,'$[1].level') FROM texts WHERE id='graded-dangun'"
        )[0],
        "∅"
    );
}

#[test]
fn required_text_without_approved_enrichment_fails() {
    let fx = Fx::new();
    let cat = fs::read_to_string(fx.t().join("catalog.toml"))
        .unwrap()
        .replace(
            "id = \"unapproved\"",
            "id = \"unapproved\"\nrequired = true",
        );
    fs::write(fx.t().join("catalog.toml"), cat).unwrap();
    let err = format!("{:#}", fx.build(false).unwrap_err());
    assert!(
        err.contains("unapproved") && err.contains("required"),
        "{err}"
    );
    assert!(fx.build(true).is_ok());
}

#[test]
fn stray_enriched_file_and_missing_texts_dir() {
    let fx = Fx::new();
    write(
        &fx.t().join("enriched/not-in-catalog.json"),
        &enriched("not-in-catalog", "approved", &["x"]),
    );
    assert!(format!("{:#}", fx.build(false).unwrap_err()).contains("not in the catalogue"));
    // no catalogue at all: nothing to build, no error
    fs::remove_file(fx.t().join("catalog.toml")).unwrap();
    assert!(fx.build(false).unwrap().is_none());
}

#[test]
fn quality_gate_wants_forty_texts() {
    let few = json!({"texts": 6});
    let ok = json!({"texts": 45});
    assert_eq!(quality_gate_optional(&[("texts", &few)]).len(), 1);
    assert!(quality_gate_optional(&[("texts", &ok)]).is_empty());
}

#[test]
fn card_and_notes_only_enrichment_uses_raw_paragraphs() {
    let fx = Fx::new();
    let mut v = enriched("poem", "approved", &[]);
    v["paragraphs"] = json!("raw");
    v["labels"]["translation"] = Value::Null;
    write(&fx.t().join("enriched/poem.json"), &v);
    let r = fx.build(true).unwrap().unwrap();
    let c = Connection::open(&r.path).unwrap();
    let rows = q(&c, "SELECT orig, en FROM paragraphs WHERE text_id='poem' ORDER BY n");
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| r.ends_with("|∅")), "{rows:?}");
    assert_eq!(q(&c, "SELECT json_extract(card,'$.summary_en') FROM texts WHERE id='poem'")[0], "Summary");
    // a translation label with raw paragraphs is refused
    v["labels"]["translation"] = json!("ai");
    write(&fx.t().join("enriched/poem.json"), &v);
    assert!(format!("{:#}", fx.build(false).unwrap_err()).contains("needs labels.translation = null"));
}

#[test]
fn heading_only_source_is_left_out_not_fatal() {
    let fx = Fx::new();
    write(&fx.t().join("raw/poem.json"), &raw("poem", "## 청구영언\n\n## 가곡원류\n"));
    let r = fx.build(false).unwrap().unwrap();
    let c = Connection::open(&r.path).unwrap();
    assert!(q(&c, "SELECT id FROM texts WHERE id='poem'").is_empty());
}
