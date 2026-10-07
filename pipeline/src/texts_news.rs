//! `kdict-pipeline fetch-news`: recent 정책브리핑 (korea.kr) articles, keeping ONLY those whose page
//! shows the 공공누리 제1유형 (KOGL Type 1) mark.

use crate::texts_catalog;
use crate::texts_html::{self, element_by_name, element_by_token, html_to_text, meta_content, title_tag};
use crate::texts_http::{now_iso, Http, UreqHttp};
use crate::texts_wikitext::{decode_entities, remove_element};
use anyhow::Result;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_FEEDS: &[&str] = &["https://www.korea.kr/rss/policy.xml"];

#[derive(Debug, Clone)]
pub struct NewsOpts {
    pub catalog: PathBuf,
    pub out: PathBuf,
    pub max: usize,
    pub force: bool,
    /// Overrides the feeds from the catalogue when non-empty.
    pub feeds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub published: Option<String>,
}

fn unwrap_cdata(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix("<![CDATA[").and_then(|x| x.strip_suffix("]]>")).unwrap_or(t);
    decode_entities(t.trim())
}

fn tag_text(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let mut from = 0;
    while let Some(p) = block[from..].find(&open).map(|x| x + from) {
        let after = block.as_bytes().get(p + open.len()).copied();
        if matches!(after, Some(b'>') | Some(b' ') | Some(b'\n')) {
            let gt = block[p..].find('>')? + p;
            if block[..gt].ends_with('/') {
                return None;
            }
            let end = block[gt..].find(&format!("</{tag}>"))? + gt;
            return Some(unwrap_cdata(&block[gt + 1..end]));
        }
        from = p + open.len();
    }
    None
}

/// `YYYY-MM-DD` from an RFC 822 (`Tue, 07 Oct 2026 …`) or ISO-ish date string.
pub fn normalise_date(s: &str) -> Option<String> {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() >= 8 && b[..4].iter().all(|c| c.is_ascii_digit()) && matches!(b[4], b'-' | b'.' | b'/') {
        return find_date(s);
    }
    const MON: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let words: Vec<&str> = s.split(|c: char| c.is_whitespace() || c == ',').filter(|w| !w.is_empty()).collect();
    for w in words.windows(3) {
        if let (Ok(d), Some(m), Ok(y)) = (w[0].parse::<u32>(), MON.iter().position(|m| w[1].to_ascii_lowercase().starts_with(m)), w[2].parse::<u32>()) {
            return Some(format!("{y:04}-{:02}-{d:02}", m + 1));
        }
    }
    None
}

/// First `YYYY.MM.DD` / `YYYY-MM-DD` / `YYYY/MM/DD` in `s`, as `YYYY-MM-DD`.
pub fn find_date(s: &str) -> Option<String> {
    let cs: Vec<char> = s.chars().collect();
    for i in 0..cs.len().saturating_sub(7) {
        if !cs[i..i + 4].iter().all(|c| c.is_ascii_digit()) || !matches!(cs[i + 4], '-' | '.' | '/') {
            continue;
        }
        let mut j = i + 5;
        let num = |j: &mut usize| {
            let st = *j;
            while *j < cs.len() && cs[*j].is_ascii_digit() && *j - st < 2 {
                *j += 1;
            }
            cs[st..*j].iter().collect::<String>().parse::<u32>().ok()
        };
        let Some(m) = num(&mut j) else { continue };
        if j >= cs.len() || !matches!(cs[j], '-' | '.' | '/') {
            continue;
        }
        j += 1;
        let Some(d) = num(&mut j) else { continue };
        let y: String = cs[i..i + 4].iter().collect();
        if (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Some(format!("{y}-{m:02}-{d:02}"));
        }
    }
    None
}

pub fn parse_feed(xml: &str) -> Vec<FeedItem> {
    let mut items = Vec::new();
    for (open, close) in [("<item", "</item>"), ("<entry", "</entry>")] {
        let mut i = 0;
        while let Some(p) = xml[i..].find(open).map(|x| x + i) {
            let after = xml.as_bytes().get(p + open.len()).copied();
            if !matches!(after, Some(b'>') | Some(b' ') | Some(b'\n') | Some(b'\r')) {
                i = p + open.len();
                continue;
            }
            let end = xml[p..].find(close).map(|x| x + p).unwrap_or(xml.len());
            let block = &xml[p..end];
            i = end.max(p + 1);
            let link = tag_text(block, "link").filter(|l| !l.is_empty()).or_else(|| {
                // Atom: <link href="…"/>
                let lp = block.find("<link")?;
                let gt = block[lp..].find('>')? + lp;
                texts_html::attr_value(&block[lp + 5..gt], "href")
            }).or_else(|| tag_text(block, "guid").filter(|g| g.starts_with("http")));
            let Some(link) = link else { continue };
            let title = tag_text(block, "title").unwrap_or_default();
            let published = ["pubDate", "dc:date", "published", "updated"].iter().find_map(|t| tag_text(block, t)).and_then(|d| normalise_date(&d));
            items.push(FeedItem { title, link, published });
        }
    }
    items
}

/// Which KOGL types (1–4) a page mentions, by badge image name or Korean label.
pub fn kogl_types(html: &str) -> BTreeSet<u8> {
    let norm: String = html.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
    let mut set = BTreeSet::new();
    for t in 1..=4u8 {
        let pats = [format!("공공누리제{t}유형"), format!("공공누리{t}유형"), format!("kogltype{t}"), format!("koglcode{t}"), format!("kogl{t}")];
        if pats.iter().any(|p| norm.contains(p.as_str())) {
            set.insert(t);
        }
    }
    set
}

/// KOGL Type 1 only: the mark must be present and no other type may appear.
pub fn is_kogl1(html: &str) -> bool {
    let stripped = remove_element(&remove_element(html, "footer"), "header");
    let mut t = kogl_types(&stripped);
    if t.is_empty() {
        t = kogl_types(html);
    }
    t.len() == 1 && t.contains(&1)
}

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub published: Option<String>,
    pub text: String,
    pub kogl1: bool,
    pub body_source: String,
}

const BODY_TOKENS: &[&str] = &[
    "articlebody", "article_body", "article-body", "view_cont", "view-cont", "article_content", "article-content", "news_content", "newsview", "view_body", "contents_view", "news_body",
];

pub fn parse_article(html: &str, item: Option<&FeedItem>) -> Article {
    let title = meta_content(html, "og:title")
        .or_else(|| title_tag(html))
        .map(|t| t.split(" | ").next().unwrap_or(&t).trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| item.map(|i| i.title.clone()))
        .unwrap_or_default();
    let published = meta_content(html, "article:published_time")
        .or_else(|| meta_content(html, "datePublished"))
        .and_then(|d| normalise_date(&d))
        .or_else(|| element_by_token(html, &["date", "regdate", "write"]).and_then(|e| find_date(&html_to_text(&e))))
        .or_else(|| item.and_then(|i| i.published.clone()));
    let (body_html, body_source) = if let Some(b) = element_by_token(html, BODY_TOKENS) {
        (b, "container".to_string())
    } else if let Some(b) = element_by_name(html, "article") {
        (b, "article".to_string())
    } else if let Some(b) = element_by_name(html, "main") {
        (b, "main".to_string())
    } else {
        (element_by_name(html, "body").unwrap_or_else(|| html.to_string()), "body-fallback".to_string())
    };
    let text: Vec<String> = html_to_text(&body_html).lines().filter(|l| !l.contains("공공누리") && !l.to_lowercase().contains("kogl")).map(String::from).collect();
    Article { title, published, text: text.join("\n").trim().to_string(), kogl1: is_kogl1(html), body_source }
}

pub fn news_id(url: &str) -> String {
    for key in ["newsId=", "newsid=", "news_id="] {
        if let Some(p) = url.find(key) {
            let id: String = url[p + key.len()..].chars().take_while(|c| c.is_ascii_digit()).collect();
            if !id.is_empty() {
                return format!("news-{id}");
            }
        }
    }
    let h = Sha256::digest(url.as_bytes());
    format!("news-{}", h.iter().take(6).map(|b| format!("{b:02x}")).collect::<String>())
}

pub struct NewsSummary {
    pub report: Value,
    pub kept: usize,
}

pub fn run(opts: &NewsOpts) -> Result<()> {
    let http = UreqHttp::new();
    let s = run_with(&http, opts)?;
    let n = |k: &str| s.report[k].as_array().map(|a| a.len()).unwrap_or(0);
    println!("fetch-news: kept {} | rejected (not KOGL type 1) {} | errors {}", s.kept, n("rejected"), n("errors"));
    Ok(())
}

pub fn run_with(http: &dyn Http, opts: &NewsOpts) -> Result<NewsSummary> {
    let feeds: Vec<String> = if !opts.feeds.is_empty() {
        opts.feeds.clone()
    } else {
        let cfg = texts_catalog::load(&opts.catalog).map(|c| c.news.feeds).unwrap_or_default();
        if cfg.is_empty() {
            DEFAULT_FEEDS.iter().map(|s| s.to_string()).collect()
        } else {
            cfg
        }
    };
    fs::create_dir_all(&opts.out)?;
    let (mut kept, mut rejected, mut errors, mut feed_rows) = (vec![], vec![], vec![], vec![]);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut count = 0usize;
    let mut examined = 0usize;
    'feeds: for feed in &feeds {
        let items = match http.get(feed).map(|x| parse_feed(&x)) {
            Ok(i) => i,
            Err(e) => {
                log::warn!("feed {feed}: {e:#}");
                feed_rows.push(json!({"url": feed, "error": format!("{e:#}")}));
                continue;
            }
        };
        feed_rows.push(json!({"url": feed, "items": items.len()}));
        for item in items {
            if count >= opts.max || examined >= opts.max * 4 {
                break 'feeds;
            }
            if !seen.insert(item.link.clone()) {
                continue;
            }
            let id = news_id(&item.link);
            let path = opts.out.join(format!("{id}.json"));
            if path.exists() && !opts.force {
                count += 1;
                kept.push(json!({"id": id, "url": item.link, "title": item.title, "note": "already present"}));
                continue;
            }
            examined += 1;
            let html = match http.get(&item.link) {
                Ok(h) => h,
                Err(e) => {
                    log::warn!("article {}: {e:#}", item.link);
                    errors.push(json!({"url": item.link, "error": format!("{e:#}")}));
                    continue;
                }
            };
            let a = parse_article(&html, Some(&item));
            if !a.kogl1 {
                log::info!("rejected (no KOGL type 1 mark): {}", item.link);
                rejected.push(json!({"url": item.link, "title": a.title, "reason": "KOGL type 1 mark not found (or another KOGL type present)"}));
                continue;
            }
            if a.text.chars().count() < 80 {
                errors.push(json!({"url": item.link, "error": "article body too short / not extracted"}));
                continue;
            }
            let doc = json!({
                "id": id, "source": "korea-kr", "url": item.link, "feed": feed, "title": a.title, "published": a.published,
                "fetched_at": now_iso(), "kogl_type": 1,
                "licence": "KOGL Type 1 (공공누리 제1유형): attribution required, commercial use and modification allowed",
                "attribution": "출처: 정책브리핑 (korea.kr), 대한민국 정부",
                "text": a.text, "body_source": a.body_source,
            });
            fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
            log::info!("kept {id}: {}", a.title);
            count += 1;
            kept.push(json!({"id": id, "url": item.link, "title": a.title}));
        }
    }
    let report = json!({"generated_at": now_iso(), "feeds": feed_rows, "kept": kept, "rejected": rejected, "errors": errors});
    fs::write(Path::new(&opts.out).join("_report.json"), serde_json::to_string_pretty(&report)?)?;
    Ok(NewsSummary { report, kept: count })
}
