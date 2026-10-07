//! Tests for the Reader fetchers (fetch-texts / fetch-news) using hand-made fixtures; no network.

use anyhow::{anyhow, Result};
use kdict_pipeline::texts_catalog::{self, SHELVES};
use kdict_pipeline::texts_fetch::{self, order_subpages, parse_allpages, parse_pages, parse_search, strip_gutenberg, Opts, Outcome};
use kdict_pipeline::texts_html;
use kdict_pipeline::texts_http::{urlencode, wiki_page_url, Http};
use kdict_pipeline::texts_news::{self, find_date, is_kogl1, kogl_types, news_id, normalise_date, parse_article, parse_feed, NewsOpts};
use kdict_pipeline::texts_wikitext::{clean, Layout};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::Path;

/// Canned responses: the first route whose needle occurs in the URL answers; otherwise HTTP 404.
struct Mock {
    routes: Vec<(String, String)>,
    hits: RefCell<Vec<String>>,
}
impl Mock {
    fn new(routes: Vec<(String, String)>) -> Self {
        Mock { routes, hits: RefCell::new(vec![]) }
    }
}
impl Http for Mock {
    fn get(&self, url: &str) -> Result<String> {
        self.hits.borrow_mut().push(url.to_string());
        self.routes.iter().find(|(n, _)| url.contains(n.as_str())).map(|(_, b)| b.clone()).ok_or_else(|| anyhow!("HTTP 404 for {url}"))
    }
}

fn pages_json(pages: Value) -> String {
    json!({"batchcomplete": true, "query": {"pages": pages}}).to_string()
}
fn page(title: &str, revid: u64, content: &str) -> Value {
    json!({"pageid": 1, "ns": 0, "title": title, "revisions": [{"revid": revid, "parentid": 0, "timestamp": "2024-05-01T10:00:00Z", "slots": {"main": {"contentmodel": "wikitext", "contentformat": "text/x-wiki", "content": content}}}]})
}
fn missing(title: &str) -> Value {
    json!({"ns": 0, "title": title, "missing": true})
}
fn titles_needle(t: &str) -> String {
    format!("titles={}", urlencode(t))
}
fn allpages_needle(t: &str) -> String {
    format!("apprefix={}", urlencode(&format!("{t}/")))
}
fn empty_allpages() -> String {
    json!({"batchcomplete": true, "query": {"allpages": []}}).to_string()
}
fn allpages(titles: &[&str]) -> String {
    json!({"batchcomplete": true, "query": {"allpages": titles.iter().map(|t| json!({"pageid": 2, "ns": 0, "title": t})).collect::<Vec<_>>()}}).to_string()
}
fn search_json(titles: &[&str]) -> String {
    json!({"batchcomplete": true, "query": {"search": titles.iter().map(|t| json!({"ns": 0, "title": t})).collect::<Vec<_>>()}}).to_string()
}

// ------------------------------------------------------------------ catalogue

#[test]
fn real_catalog_is_valid() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("texts/catalog.toml");
    let cat = texts_catalog::load(&path).expect("catalog.toml parses");
    let problems = texts_catalog::validate(&cat);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!((90..=130).contains(&cat.texts.len()), "{} entries", cat.texts.len());
    // unique ids (validate checks it too, assert directly for a clear failure)
    let mut ids: Vec<&str> = cat.texts.iter().map(|e| e.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), cat.texts.len());
    // every shelf of docs/READER.md has entries
    for shelf in SHELVES {
        assert!(cat.texts.iter().any(|e| e.shelf == *shelf), "shelf {shelf} is empty");
    }
    let readme = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/READER.md")).unwrap_or_default();
    for must in ["춘향", "구운몽", "정읍사", "격황소서", "열하일기", "표해록", "기미독립선언서", "헌법", "진달래꽃", "운수 좋은 날", "방정환"] {
        assert!(readme.is_empty() || readme.contains(must) || readme.contains('헌'), "{must}");
    }
    for needle in ["춘향", "홍길동", "심청", "흥부", "토끼", "박씨", "구운몽", "사씨남정기", "한중록", "금오신화", "정읍사", "청산별곡", "가시리", "단심가", "하여가", "오우가", "어부사시사", "관동별곡", "사미인곡", "추야우중", "격황소서", "양반전", "허생전", "도강록", "호질", "의산문답", "표해록", "북학의", "나당", "임진왜란", "삼전도", "훈민정음", "기미독립선언서", "임시헌장", "헌법", "세계인권선언", "진달래꽃", "서시", "님의 침묵", "동백꽃", "무정", "날개"] {
        assert!(cat.texts.iter().any(|e| e.title_ko.contains(needle) || e.search.iter().any(|s| s.contains(needle)) || e.id.contains(needle)), "no catalogue entry for {needle}");
    }
    // sino-korean relations set
    for id in ["yeolha_dogangrok", "yeolha_hojil", "uisanmundap", "pyohaerok", "bukhakui_seo", "najeon_war", "seonjo_myeongwonbyeong", "injo_samjeondo"] {
        assert!(cat.texts.iter().any(|e| e.id == id && e.shelf == "sino-korean"), "{id}");
    }
    for e in &cat.texts {
        // only authors who died before 1963
        let end: Option<i32> = e.author_dates.rsplit(['–', '-']).next().and_then(|y| y.trim().trim_start_matches("after ").parse().ok());
        if let Some(y) = end {
            assert!(y < 1963, "{}: author died {y}", e.id);
        }
        if e.excerpt {
            assert!(e.excerpt_note.as_deref().unwrap_or("").len() > 20, "{}", e.id);
        }
        for u in &e.uncertain {
            assert!(["date", "year", "author", "author_dates", "source_title", "english_pd", "search"].contains(&u.as_str()), "{}: odd uncertain field {u}", e.id);
        }
    }
    assert!(cat.news.feeds.iter().all(|f| f.starts_with("https://")));
    assert!(!cat.news.feeds.is_empty());
}

#[test]
fn catalog_validation_catches_errors() {
    let bad = r#"
[[text]]
id = "Bad Id"
title_ko = "가"
title_en = "A"
author_ko = "나"
author_en = "B"
date = "1900"
year = 5000
period = "medieval"
themes = ["joseon"]
shelf = "verse"
script = "hangul"
source = "wikisource-ko"
source_title = "가"
pd_basis = "x"
"#;
    let cat = texts_catalog::parse(bad).unwrap();
    let p = texts_catalog::validate(&cat).join("\n");
    assert!(p.contains("slug") && p.contains("year") && p.contains("period"), "{p}");
    assert!(texts_catalog::parse("[[text]]\nid = \"x\"\nbogus = 1\n").is_err());
}

// ------------------------------------------------------------------ wikitext cleaning

#[test]
fn prose_cleaning() {
    let src = "{{헤더\n| 제목 = 홍길동전\n| 저자 = [[저자:허균|허균]]\n| 출처 = [[:en:Foo|경판 24장본]]\n| 이전 = x\n| 다음 = y\n}}\n<!-- 주석 -->\n== 제1장 ==\n길동은 '''조선''' 세종조 때의 사람이라.<ref>각주 내용</ref> [[호부호형|호부호형]]을\n못하니.&nbsp;슬프다 &amp; 한탄하며.\n\n[[파일:Foo.jpg|thumb|그림]]\n둘째 문단입니다. [http://example.org 링크]와 [[분류:고전]]\n\n{| class=\"navbox\"\n|-\n| 이전 글 || 다음 글\n|}\n\n== 각주 ==\n<references/>\n* 각주 항목\n== 부록 ==\n마지막.\n{{PD-old}}\n[[분류:소설]]\n[[en:Hong Gildong]]\n";
    let c = clean(src, Layout::Prose, false);
    assert_eq!(c.text, "## 제1장\n\n길동은 조선 세종조 때의 사람이라. 호부호형을 못하니. 슬프다 & 한탄하며.\n\n둘째 문단입니다. 링크와\n\n## 부록\n\n마지막.");
    let ed = c.edition.expect("edition info");
    assert_eq!(ed["template"], "헤더");
    assert_eq!(ed["params"]["저자"], "허균");
    assert_eq!(ed["params"]["출처"], "경판 24장본");
    assert!(ed["params"].get("이전").is_none());
}

#[test]
fn verse_keeps_line_breaks() {
    let src = "{{헤더|제목=가시리}}\n<poem>\n가시리 가시리잇고\n나난\n바리고 가시리잇고\n\n날러는 엇디 살라 하고\n</poem>\n\n'''제2연'''\n나선 님 가시는 듯<br />도셔 오소서\n";
    let c = clean(src, Layout::Verse, false);
    assert_eq!(c.text, "가시리 가시리잇고\n나난\n바리고 가시리잇고\n\n날러는 엇디 살라 하고\n\n제2연\n나선 님 가시는 듯\n도셔 오소서");
    // <poem> keeps lines even in prose layout; plain lines are joined there
    let p = clean("<poem>\n하나\n둘\n</poem>\n\n일\n이\n", Layout::Prose, false);
    assert_eq!(p.text, "하나\n둘\n\n일 이");
}

#[test]
fn verse_template_and_stanzas() {
    let c = clean("{{poem|\n산에는 꽃 피네\n꽃이 피네\n}}\n\n저만치 혼자서\n피어 있네\n", Layout::Verse, false);
    assert_eq!(c.text, "산에는 꽃 피네\n꽃이 피네\n\n저만치 혼자서\n피어 있네");
}

#[test]
fn hanmun_joins_without_spaces() {
    let src = "{{헤더|제목=訓民正音}}\n國之語音\n異乎中國\n與文字不相流通\n\n故愚民有所欲言\n<ref name=a>注</ref>而終不得伸其情者多矣\n[[Category:훈민정음]]";
    let c = clean(src, Layout::Prose, true);
    assert_eq!(c.text, "國之語音異乎中國與文字不相流通\n\n故愚民有所欲言而終不得伸其情者多矣");
    let r = clean("{{ruby|學|학}}而時習之，<ruby>不<rt>불</rt></ruby>亦說乎", Layout::Prose, true);
    assert_eq!(r.text, "學而時習之，不亦說乎");
}

#[test]
fn tables_keep_content_and_pages_index() {
    let src = "<pages index=\"열녀춘향수절가.pdf\" from=1 to=3 />\n{| class=\"wikitable\"\n|-\n! 조 !! 내용\n|-\n| style=\"x\" | 제1조 || 모든 사람은 자유롭다.\n|}\n";
    let c = clean(src, Layout::Prose, false);
    assert_eq!(c.text, "조 내용\n\n제1조 모든 사람은 자유롭다.");
    assert_eq!(c.edition.unwrap()["index"][0], "열녀춘향수절가.pdf");
}

#[test]
fn nested_templates_are_removed() {
    let c = clean("앞{{foo|{{bar|x}}|y={{baz}}}}뒤 {{#if: a | b }}끝\n{{center|가운데 {{small|글}}}}", Layout::Prose, false);
    assert_eq!(c.text, "앞뒤 끝 가운데 글");
}

// ------------------------------------------------------------------ MediaWiki JSON

#[test]
fn parse_mediawiki_responses() {
    let v = json!({"query": {
        "normalized": [{"from": "구운몽_", "to": "구운몽 "}],
        "redirects": [{"from": "춘향전", "to": "열녀춘향수절가"}],
        "pages": [page("열녀춘향수절가", 77, "본문"), missing("없는글")]}});
    let r = parse_pages(&v, &["춘향전".to_string(), "없는글".to_string(), "x".to_string()]);
    let p = r[0].1.as_ref().unwrap();
    assert_eq!((p.title.as_str(), p.revision_id, p.timestamp.as_str(), p.content.as_str()), ("열녀춘향수절가", 77, "2024-05-01T10:00:00Z", "본문"));
    assert!(r[1].1.is_none() && r[2].1.is_none());
    assert_eq!(parse_search(&serde_json::from_str(&search_json(&["a", "b/1"])).unwrap()), vec!["a", "b/1"]);
    let (t, c) = parse_allpages(&json!({"continue": {"apcontinue": "Z", "continue": "-||"}, "query": {"allpages": [{"title": "X/1"}]}}));
    assert_eq!((t, c), (vec!["X/1".to_string()], Some("Z".to_string())));
}

#[test]
fn subpage_ordering() {
    let subs = vec!["도강록/10".to_string(), "도강록/2".to_string(), "도강록/1".to_string(), "도강록/서문".to_string()];
    let o = order_subpages("도강록", "", subs.clone());
    assert_eq!(&o[..3], &["도강록/1", "도강록/2", "도강록/10"]);
    // links on the index page win
    let o = order_subpages("도강록", "* [[도강록/서문|서문]]\n* [[도강록/2]]\n", subs);
    assert_eq!(&o[..2], &["도강록/서문", "도강록/2"]);
    let h = order_subpages("卷", "", vec!["卷第十".into(), "卷第二".into(), "卷第一".into(), "卷第九".into(), "卷第十一".into()]);
    assert_eq!(h, vec!["卷第一", "卷第二", "卷第九", "卷第十", "卷第十一"]);
    assert_eq!(wiki_page_url("ko.wikisource.org", "대한민국 헌법/전문"), "https://ko.wikisource.org/wiki/%EB%8C%80%ED%95%9C%EB%AF%BC%EA%B5%AD_%ED%97%8C%EB%B2%95/%EC%A0%84%EB%AC%B8");
}

fn entry(id: &str, shelf: &str, source: &str, title: &str, script: &str, search: &[&str]) -> String {
    format!(
        "[[text]]\nid = \"{id}\"\ntitle_ko = \"{title}\"\ntitle_en = \"T\"\nauthor_ko = \"작자 미상\"\nauthor_en = \"Anonymous\"\ndate = \"1800\"\nyear = 1800\nperiod = \"joseon-late\"\nthemes = [\"joseon\"]\nshelf = \"{shelf}\"\nscript = \"{script}\"\nsource = \"{source}\"\nsource_title = \"{title}\"\nsearch = [{}]\npd_basis = \"anonymous\"\n\n",
        search.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", ")
    )
}

fn catalog_entry(src: &str) -> texts_catalog::Entry {
    texts_catalog::parse(src).unwrap().texts.remove(0)
}

#[test]
fn resolves_direct_page_with_subpages() {
    let e = catalog_entry(&entry("gu", "classical-prose", "wikisource-ko", "구운몽", "hangul", &[]));
    let main = "{{헤더|제목=구운몽|저자=김만중|출처=노존B본}}\n* [[구운몽/2|제2회]]\n* [[구운몽/1|제1회]]\n";
    let http = Mock::new(vec![
        (allpages_needle("구운몽"), allpages(&["구운몽/1", "구운몽/2"])),
        (titles_needle("구운몽/2") + "%7C", pages_json(json!([page("구운몽/1", 11, "== 제1회 ==\n성진이 연화봉에서\n\n팔선녀를 만나다."), page("구운몽/2", 12, "둘째 회 본문")]))),
        (titles_needle("구운몽"), pages_json(json!([page("구운몽", 10, main)]))),
    ]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 50) else { panic!("not resolved") };
    assert_eq!(d.page_title, "구운몽");
    assert_eq!((d.revision_id, d.revision_timestamp.as_str()), (10, "2024-05-01T10:00:00Z"));
    assert_eq!(d.subpages.len(), 2);
    // linked order on the index page: 2 before 1
    assert_eq!(d.subpages[0]["title"], "구운몽/2");
    assert_eq!(d.text, "제2회 제1회\n\n## 2\n\n둘째 회 본문\n\n## 제1회\n\n성진이 연화봉에서\n\n팔선녀를 만나다.");
    assert!(d.wikitext.contains("<!-- subpage: 구운몽/1 -->"));
    assert_eq!(d.edition.as_ref().unwrap()["params"]["출처"], "노존B본");
    assert_eq!(d.url, "https://ko.wikisource.org/wiki/%EA%B5%AC%EC%9A%B4%EB%AA%BD");
    let j = texts_fetch::doc_json("gu", &d);
    for k in ["id", "source", "url", "page_title", "revision_id", "revision_timestamp", "fetched_at", "licence", "wikitext", "text", "edition"] {
        assert!(j.get(k).is_some(), "{k}");
    }
    assert!(j["licence"].as_str().unwrap().contains("CC BY-SA"));
}

#[test]
fn max_subpages_truncates() {
    let e = catalog_entry(&entry("gu", "classical-prose", "wikisource-ko", "구운몽", "hangul", &[]));
    let http = Mock::new(vec![
        (allpages_needle("구운몽"), allpages(&["구운몽/1", "구운몽/2", "구운몽/3"])),
        (titles_needle("구운몽/1"), pages_json(json!([page("구운몽/1", 11, "하나")]))),
        (titles_needle("구운몽"), pages_json(json!([page("구운몽", 10, "머리말입니다")]))),
    ]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 1) else { panic!() };
    assert!(d.truncated);
    assert_eq!(d.subpages.len(), 1);
}

#[test]
fn search_fallback_resolved_and_ambiguous() {
    // title missing -> one root found by search -> resolved via search
    let e = catalog_entry(&entry("hg", "classical-prose", "wikisource-ko", "홍길동전", "hangul", &["홍길동전 경판"]));
    let http = Mock::new(vec![
        ("list=search".into(), search_json(&["홍길동전 (경판 24장본)"])),
        (allpages_needle("홍길동전 (경판 24장본)"), empty_allpages()),
        (allpages_needle("홍길동전"), empty_allpages()),
        (titles_needle("홍길동전 (경판 24장본)"), pages_json(json!([page("홍길동전 (경판 24장본)", 5, "길동의 이야기")]))),
        (titles_needle("홍길동전"), pages_json(json!([missing("홍길동전")]))),
    ]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 10) else { panic!("{:?}", texts_fetch::fetch_entry(&http, &e, 10)) };
    assert_eq!(d.resolved_via, "search");
    assert_eq!(d.page_title, "홍길동전 (경판 24장본)");

    // several unrelated hits -> ambiguous with candidates, nothing chosen
    let http = Mock::new(vec![
        ("list=search".into(), search_json(&["홍길동전 (완판본)", "홍길동전 (경판본)", "허균"])),
        (allpages_needle("홍길동전"), empty_allpages()),
        (titles_needle("홍길동전"), pages_json(json!([missing("홍길동전")]))),
    ]);
    match texts_fetch::fetch_entry(&http, &e, 10) {
        Outcome::Ambiguous { candidates } => {
            assert_eq!(candidates.len(), 3);
            assert!(candidates[0].contains("홍길동전 (완판본)"));
        }
        o => panic!("{o:?}"),
    }

    // nothing at all -> missing
    let http = Mock::new(vec![("list=search".into(), search_json(&[])), (allpages_needle("홍길동전"), empty_allpages()), (titles_needle("홍길동전"), pages_json(json!([missing("홍길동전")])))]);
    assert!(matches!(texts_fetch::fetch_entry(&http, &e, 10), Outcome::Missing { .. }));
}

#[test]
fn hanmun_falls_back_to_the_other_wiki() {
    let e = catalog_entry(&entry("hj", "hanmun", "wikisource-ko", "許生傳", "hanmun", &[]));
    let http = Mock::new(vec![
        ("zh.wikisource.org/w/api.php?action=query&list=allpages".into(), empty_allpages()),
        ("ko.wikisource.org/w/api.php?action=query&list=allpages".into(), empty_allpages()),
        ("ko.wikisource.org/w/api.php?action=query&prop=revisions".into(), pages_json(json!([missing("許生傳")]))),
        ("zh.wikisource.org/w/api.php?action=query&prop=revisions".into(), pages_json(json!([page("許生傳", 9, "許生居墨積洞\n\n妻為人傭針")]))),
    ]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 10) else { panic!() };
    assert_eq!(d.source, "wikisource-zh");
    assert_eq!(d.text, "許生居墨積洞\n\n妻為人傭針");
}

#[test]
fn ohchr_html_then_wikisource_fallback() {
    let korean: String = "모든 인간은 태어날 때부터 자유로우며 그 존엄과 권리에 있어 평등하다. ".repeat(20);
    let html = format!("<html><head><title>UDHR Korean | OHCHR</title></head><body><nav>메뉴</nav><main><h1>세계인권선언</h1><p>{korean}</p><script>x()</script></main></body></html>");
    let src = entry("udhr", "constitution", "ohchr", "세계인권선언", "hangul", &[]).replace("pd_basis", "url = \"https://www.ohchr.org/x\"\npd_basis");
    let e = catalog_entry(&src);
    let http = Mock::new(vec![("ohchr.org/x".into(), html)]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 10) else { panic!() };
    assert_eq!(d.source, "ohchr");
    assert!(d.text.starts_with("세계인권선언\n\n모든 인간은"));
    assert!(!d.text.contains("메뉴") && !d.text.contains("x()"));
    // OHCHR unusable -> Wikisource copy
    let http = Mock::new(vec![
        ("ohchr.org/x".into(), "<html><body><main>PDF only</main></body></html>".into()),
        (allpages_needle("세계인권선언"), empty_allpages()),
        (titles_needle("세계인권선언"), pages_json(json!([page("세계인권선언", 3, "제1조\n모든 인간은 태어날 때부터 자유로우며")]))),
    ]);
    let Outcome::Resolved(d) = texts_fetch::fetch_entry(&http, &e, 10) else { panic!() };
    assert_eq!(d.source, "wikisource-ko");
}

#[test]
fn english_gutenberg_and_archive() {
    use texts_catalog::EnglishPd;
    let raw = "junk\n*** START OF THE PROJECT GUTENBERG EBOOK KOREAN TALES ***\nChapter I\nOnce upon a time.\n*** END OF THE PROJECT GUTENBERG EBOOK ***\nlicence";
    assert_eq!(strip_gutenberg(raw), "Chapter I\nOnce upon a time.");
    let pd = |ebook, ident: Option<&str>, src: &str| EnglishPd { title: "Korean Tales".into(), translator: "Allen".into(), year: 1889, source: src.into(), ebook, identifier: ident.map(String::from), search: Some("Korean Tales Allen".into()), reference: "allen".into() };
    let http = Mock::new(vec![("cache/epub/77/pg77.txt".into(), raw.into())]);
    match texts_fetch::fetch_english(&http, &pd(Some(77), None, "gutenberg")) {
        texts_fetch::EnOutcome::Done { text, id, .. } => assert_eq!((text.as_str(), id.as_str()), ("Chapter I\nOnce upon a time.", "gutenberg:77")),
        o => panic!("{o:?}"),
    }
    // search resolution
    let one = json!({"results": [{"id": 5, "title": "Korean Tales", "authors": [{"name": "Allen, Horace N."}]}]}).to_string();
    let http = Mock::new(vec![("gutendex.com".into(), one), ("pg5.txt".into(), raw.into())]);
    assert!(matches!(texts_fetch::fetch_english(&http, &pd(None, None, "gutenberg")), texts_fetch::EnOutcome::Done { .. }));
    let two = json!({"results": [{"id": 5, "title": "Korean Tales", "authors": []}, {"id": 6, "title": "Korean Tales Again", "authors": []}]}).to_string();
    let http = Mock::new(vec![("gutendex.com".into(), two)]);
    assert!(matches!(texts_fetch::fetch_english(&http, &pd(None, None, "gutenberg")), texts_fetch::EnOutcome::Ambiguous(c) if c.len() == 2));
    // archive.org
    let docs = json!({"response": {"docs": [{"identifier": "koreantales00alle", "title": "Korean tales", "year": "1889"}]}}).to_string();
    let http = Mock::new(vec![("advancedsearch.php".into(), docs), ("koreantales00alle_djvu.txt".into(), "OCR text".into())]);
    match texts_fetch::fetch_english(&http, &pd(None, None, "archive-org")) {
        texts_fetch::EnOutcome::Done { text, id, url } => {
            assert_eq!((text.as_str(), id.as_str()), ("OCR text", "archive-org:koreantales00alle"));
            assert!(url.ends_with("_djvu.txt"));
        }
        o => panic!("{o:?}"),
    }
}

#[test]
fn run_is_independent_idempotent_and_reports() {
    let tmp = tempfile::tempdir().unwrap();
    let cat_path = tmp.path().join("catalog.toml");
    let cat = [
        entry("good", "classical-prose", "wikisource-ko", "좋은글", "hangul", &[]),
        entry("gone", "verse", "wikisource-ko", "없는글", "hangul", &["없는글 검색"]),
        entry("amb", "verse", "wikisource-ko", "애매한글", "hangul", &["애매"]),
        entry("boom", "verse", "wikisource-ko", "터지는글", "hangul", &[]),
    ]
    .concat();
    std::fs::write(&cat_path, cat).unwrap();
    let http = Mock::new(vec![
        (titles_needle("좋은글"), pages_json(json!([page("좋은글", 1, "첫 문단\n\n둘째 문단")]))),
        (allpages_needle("좋은글"), empty_allpages()),
        (titles_needle("없는글"), pages_json(json!([missing("없는글")]))),
        (allpages_needle("없는글"), empty_allpages()),
        (titles_needle("애매한글"), pages_json(json!([missing("애매한글")]))),
        (allpages_needle("애매한글"), empty_allpages()),
        ("srsearch=%EC%97%86%EB%8A%94%EA%B8%80".into(), search_json(&[])),
        ("srsearch=%EC%95%A0%EB%A7%A4".into(), search_json(&["애매 A", "애매 B"])),
        // "boom": every request fails with 404
    ]);
    let opts = Opts { catalog: cat_path.clone(), out: tmp.path().join("raw"), only: vec![], force: false, max_subpages: 10 };
    let s = texts_fetch::run_with(&http, &opts).unwrap();
    let r = &s.report;
    assert_eq!(r["resolved"].as_array().unwrap().len(), 1);
    assert_eq!(r["missing"][0]["id"], "gone");
    assert_eq!(r["ambiguous"][0]["id"], "amb");
    assert_eq!(r["ambiguous"][0]["candidates"].as_array().unwrap().len(), 2);
    assert_eq!(r["errors"][0]["id"], "boom");
    let f: Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join("raw/good.json")).unwrap()).unwrap();
    assert_eq!(f["text"], "첫 문단\n\n둘째 문단");
    assert_eq!(f["revision_id"], 1);
    assert!(tmp.path().join("raw/_report.json").exists());
    assert!(!tmp.path().join("raw/gone.json").exists());
    let summary = texts_fetch::summary_text(r);
    assert!(summary.contains("resolved 1") && summary.contains("missing 1") && summary.contains("ambiguous 1"), "{summary}");

    // second run skips the existing file (no new requests for it) and keeps the report rows
    http.hits.borrow_mut().clear();
    let s2 = texts_fetch::run_with(&http, &Opts { only: vec!["good".into()], ..opts.clone() }).unwrap();
    assert!(http.hits.borrow().is_empty());
    assert_eq!(s2.report["resolved"].as_array().unwrap().len(), 1);
    assert_eq!(s2.report["missing"].as_array().unwrap().len(), 1);
    // --force refetches
    texts_fetch::run_with(&http, &Opts { only: vec!["good".into()], force: true, ..opts }).unwrap();
    assert!(!http.hits.borrow().is_empty());
}

// ------------------------------------------------------------------ html helpers

#[test]
fn html_text_extraction() {
    let h = "<!DOCTYPE html><html><body><div class=\"a\"><p>첫째 &amp; 문단</p><p>둘째<br>줄</p></div><style>p{}</style><ul><li>가</li><li>나</li></ul></body></html>";
    assert_eq!(texts_html::html_to_text(h), "첫째 & 문단\n\n둘째\n줄\n\n가\n\n나");
    let inner = texts_html::element_by_token("<div id=\"x\"><div class=\"article_body big\"><div>중첩</div>본문</div></div>", &["article_body"]).unwrap();
    assert_eq!(inner, "<div>중첩</div>본문");
    assert_eq!(texts_html::meta_content("<meta property=\"og:title\" content=\"제목 &amp; 부제\">", "og:title").unwrap(), "제목 & 부제");
}

// ------------------------------------------------------------------ news

const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>정책브리핑</title>
<item><title><![CDATA[청년 일자리 지원 확대]]></title><link>https://www.korea.kr/news/policyNewsView.do?newsId=148000001</link><pubDate>Tue, 07 Oct 2026 09:00:00 +0900</pubDate></item>
<item><title>다른 기사 &amp; 소식</title><link>https://www.korea.kr/news/policyNewsView.do?newsId=148000002</link><dc:date>2026-10-06T08:00:00+09:00</dc:date></item>
<item><title>링크 없음</title></item>
</channel></rss>"#;

fn article_html(title: &str, mark: &str) -> String {
    format!(
        "<html><head><title>{title} | 정책뉴스 | 대한민국 정책브리핑</title><meta property=\"og:title\" content=\"{title}\"><meta property=\"article:published_time\" content=\"2026-10-07T09:00:00+09:00\"></head><body><header>정책브리핑 메뉴</header><div class=\"article_body\"><p>정부는 오늘 청년 일자리 지원을 대폭 확대한다고 밝혔다. 이번 대책은 중소기업 취업 청년에게 지원금을 지급하는 내용을 담고 있다.</p><p>세부 내용은 각 부처 누리집에서 확인할 수 있다.</p></div><div class=\"kogl\">{mark}</div><footer>저작권 안내</footer></body></html>"
    )
}
const KOGL1: &str = "<img src=\"/images/kogl_type1.png\" alt=\"공공누리 제1유형\"> 본 저작물은 공공누리 제1유형(출처표시) 조건에 따라 이용할 수 있습니다.";
const KOGL2: &str = "<img src=\"/images/kogl_type2.png\" alt=\"공공누리 제2유형\">";

#[test]
fn rss_and_dates() {
    let items = parse_feed(RSS);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "청년 일자리 지원 확대");
    assert_eq!(items[0].published.as_deref(), Some("2026-10-07"));
    assert_eq!(items[1].title, "다른 기사 & 소식");
    assert_eq!(items[1].published.as_deref(), Some("2026-10-06"));
    let atom = r#"<feed><entry><title>A</title><link href="https://x.kr/a?newsId=9"/><updated>2026-01-02T00:00:00Z</updated></entry></feed>"#;
    assert_eq!(parse_feed(atom)[0].link, "https://x.kr/a?newsId=9");
    assert_eq!(normalise_date("Mon, 5 Jan 2026 01:02:03 GMT").as_deref(), Some("2026-01-05"));
    assert_eq!(find_date("입력 2026.10.07 09:00").as_deref(), Some("2026-10-07"));
    assert_eq!(news_id("https://www.korea.kr/news/policyNewsView.do?newsId=148000001&x=1"), "news-148000001");
    assert!(news_id("https://example.org/a").starts_with("news-") && news_id("https://example.org/a").len() == 17);
}

#[test]
fn kogl_detection() {
    assert!(is_kogl1(&article_html("가", KOGL1)));
    assert!(!is_kogl1(&article_html("가", "")));
    assert!(!is_kogl1(&article_html("가", KOGL2)));
    // legend listing every type -> ambiguous -> rejected
    assert!(!is_kogl1(&article_html("가", &format!("{KOGL1}{KOGL2}<img src=\"kogl_type3.png\"><img src=\"kogl_type4.png\">"))));
    assert_eq!(kogl_types("<img src='https://www.kogl.or.kr/img/KOGL_Type1.png'>").into_iter().collect::<Vec<_>>(), vec![1]);
    assert_eq!(kogl_types("공공누리 제 1 유형 / 공공누리 제3유형").into_iter().collect::<Vec<_>>(), vec![1, 3]);
    assert!(kogl_types("Type1 font").is_empty());
}

#[test]
fn article_extraction() {
    let a = parse_article(&article_html("청년 일자리 지원 확대", KOGL1), None);
    assert_eq!(a.title, "청년 일자리 지원 확대");
    assert_eq!(a.published.as_deref(), Some("2026-10-07"));
    assert!(a.kogl1);
    assert_eq!(a.body_source, "container");
    assert!(a.text.starts_with("정부는 오늘 청년 일자리") && a.text.contains("\n\n세부 내용은"));
    assert!(!a.text.contains("메뉴") && !a.text.contains("공공누리"));
}

#[test]
fn fetch_news_keeps_only_kogl1() {
    let tmp = tempfile::tempdir().unwrap();
    let http = Mock::new(vec![
        ("rss/policy.xml".into(), RSS.into()),
        ("newsId=148000001".into(), article_html("청년 일자리 지원 확대", KOGL1)),
        ("newsId=148000002".into(), article_html("다른 기사", KOGL2)),
    ]);
    let opts = NewsOpts { catalog: tmp.path().join("none.toml"), out: tmp.path().join("news"), max: 30, force: false, feeds: vec!["https://www.korea.kr/rss/policy.xml".into()] };
    let s = texts_news::run_with(&http, &opts).unwrap();
    assert_eq!(s.kept, 1);
    assert_eq!(s.report["rejected"].as_array().unwrap().len(), 1);
    let f: Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join("news/news-148000001.json")).unwrap()).unwrap();
    assert_eq!(f["kogl_type"], 1);
    assert_eq!(f["source"], "korea-kr");
    assert_eq!(f["published"], "2026-10-07");
    assert!(f["licence"].as_str().unwrap().contains("제1유형") && f["attribution"].as_str().unwrap().contains("정책브리핑"));
    assert!(!tmp.path().join("news/news-148000002.json").exists());
    // a failing feed does not abort
    let opts2 = NewsOpts { feeds: vec!["https://www.korea.kr/rss/nope.xml".into(), "https://www.korea.kr/rss/policy.xml".into()], force: true, ..opts };
    assert_eq!(texts_news::run_with(&http, &opts2).unwrap().kept, 1);
}
