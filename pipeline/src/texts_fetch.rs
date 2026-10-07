//! `kdict-pipeline fetch-texts`: resolve every catalogue entry against (Korean/Chinese) Wikisource,
//! the OHCHR page and the English public-domain translations, and write raw texts + provenance to
//! `pipeline/texts/raw/`. Every entry is independent: failures are logged and reported, never fatal.

use crate::texts_catalog::{self, Catalog, Entry, EnglishPd};
use crate::texts_html;
use crate::texts_http::{now_iso, urlencode, wiki_page_url, Http, UreqHttp};
use crate::texts_wikitext::{self as wt, Layout};
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const HOST_KO: &str = "ko.wikisource.org";
pub const HOST_ZH: &str = "zh.wikisource.org";

#[derive(Debug, Clone)]
pub struct Opts {
    pub catalog: PathBuf,
    pub out: PathBuf,
    pub only: Vec<String>,
    pub force: bool,
    pub max_subpages: usize,
}

// ---------------------------------------------------------------- MediaWiki client

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub revision_id: u64,
    pub timestamp: String,
    pub content: String,
}

pub struct Wiki<'a> {
    pub http: &'a dyn Http,
    pub host: String,
}

fn follow(map: &HashMap<String, String>, t: &str) -> String {
    let mut cur = t.to_string();
    for _ in 0..6 {
        match map.get(&cur) {
            Some(n) if *n != cur => cur = n.clone(),
            _ => break,
        }
    }
    cur
}

/// Parse a `prop=revisions` response (formatversion=2). Returns, for each requested title, the page
/// it ended up on (after normalisation and redirects) or `None` when it does not exist.
pub fn parse_pages(v: &Value, requested: &[String]) -> Vec<(String, Option<Page>)> {
    let mut map: HashMap<String, String> = HashMap::new();
    for key in ["normalized", "redirects"] {
        if let Some(a) = v["query"][key].as_array() {
            for r in a {
                if let (Some(f), Some(t)) = (r["from"].as_str(), r["to"].as_str()) {
                    map.insert(f.to_string(), t.to_string());
                }
            }
        }
    }
    let mut pages: HashMap<String, Option<Page>> = HashMap::new();
    if let Some(a) = v["query"]["pages"].as_array() {
        for p in a {
            let Some(title) = p["title"].as_str() else { continue };
            let page = p["revisions"][0]["slots"]["main"]["content"].as_str().map(|c| Page {
                title: title.to_string(),
                revision_id: p["revisions"][0]["revid"].as_u64().unwrap_or(0),
                timestamp: p["revisions"][0]["timestamp"].as_str().unwrap_or("").to_string(),
                content: c.to_string(),
            });
            pages.insert(title.to_string(), if p["missing"].as_bool().unwrap_or(false) || p["invalid"].as_bool().unwrap_or(false) { None } else { page });
        }
    }
    requested.iter().map(|r| (r.clone(), pages.get(&follow(&map, r)).cloned().flatten())).collect()
}

pub fn parse_search(v: &Value) -> Vec<String> {
    v["query"]["search"].as_array().map(|a| a.iter().filter_map(|x| x["title"].as_str().map(String::from)).collect()).unwrap_or_default()
}

/// Returns (titles, continuation token).
pub fn parse_allpages(v: &Value) -> (Vec<String>, Option<String>) {
    let t = v["query"]["allpages"].as_array().map(|a| a.iter().filter_map(|x| x["title"].as_str().map(String::from)).collect()).unwrap_or_default();
    (t, v["continue"]["apcontinue"].as_str().map(String::from))
}

impl Wiki<'_> {
    fn api(&self, query: &str) -> Result<Value> {
        let url = format!("https://{}/w/api.php?{query}&format=json&formatversion=2", self.host);
        let body = self.http.get(&url)?;
        let v: Value = serde_json::from_str(&body).with_context(|| format!("bad JSON from {url}"))?;
        if let Some(err) = v.get("error") {
            bail!("MediaWiki API error: {err}");
        }
        Ok(v)
    }

    pub fn get_pages(&self, titles: &[String]) -> Result<Vec<(String, Option<Page>)>> {
        let mut out = Vec::new();
        for chunk in titles.chunks(20) {
            let t: Vec<String> = chunk.iter().map(|t| urlencode(t)).collect();
            let q = format!("action=query&prop=revisions&rvprop=content%7Cids%7Ctimestamp&rvslots=main&redirects=1&titles={}", t.join("%7C"));
            let v = self.api(&q)?;
            out.extend(parse_pages(&v, chunk));
        }
        Ok(out)
    }

    pub fn get_page(&self, title: &str) -> Result<Option<Page>> {
        Ok(self.get_pages(&[title.to_string()])?.into_iter().next().and_then(|x| x.1))
    }

    pub fn search(&self, term: &str) -> Result<Vec<String>> {
        let v = self.api(&format!("action=query&list=search&srsearch={}&srnamespace=0&srlimit=10", urlencode(term)))?;
        Ok(parse_search(&v))
    }

    pub fn subpages(&self, title: &str) -> Result<Vec<String>> {
        let mut all = Vec::new();
        let mut cont: Option<String> = None;
        for _ in 0..6 {
            let mut q = format!("action=query&list=allpages&apnamespace=0&aplimit=500&apprefix={}", urlencode(&format!("{title}/")));
            if let Some(c) = &cont {
                q.push_str(&format!("&apcontinue={}", urlencode(c)));
            }
            let (t, next) = parse_allpages(&self.api(&q)?);
            all.extend(t);
            match next {
                Some(n) => cont = Some(n),
                None => break,
            }
        }
        all.retain(|t| !(t.ends_with("/doc") || t.ends_with("/sandbox") || t.ends_with("/Archive")));
        Ok(all)
    }
}

// ---------------------------------------------------------------- ordering of subpages

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Tok {
    Num(u64),
    Text(String),
}

fn cjk_digit(c: char) -> Option<u64> {
    "〇一二三四五六七八九".chars().position(|d| d == c).map(|p| p as u64)
}

/// "第十二" -> 12, "二十一" -> 21 (small numbers only).
fn parse_cjk_number(s: &str) -> u64 {
    let (mut total, mut cur) = (0u64, 0u64);
    for c in s.chars() {
        match c {
            '十' => {
                total += cur.max(1) * 10;
                cur = 0;
            }
            '百' => {
                total += cur.max(1) * 100;
                cur = 0;
            }
            c => cur = cur * if s.contains(['十', '百']) { 1 } else { 10 } + cjk_digit(c).unwrap_or(0),
        }
    }
    total + cur
}

fn natural_key(s: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        let is_cjk_num = |c: char| cjk_digit(c).is_some() || c == '十' || c == '百';
        if c.is_ascii_digit() {
            let j = (i..cs.len()).find(|&j| !cs[j].is_ascii_digit()).unwrap_or(cs.len());
            out.push(Tok::Num(cs[i..j].iter().collect::<String>().parse().unwrap_or(0)));
            i = j;
        } else if is_cjk_num(c) {
            let j = (i..cs.len()).find(|&j| !is_cjk_num(cs[j])).unwrap_or(cs.len());
            out.push(Tok::Num(parse_cjk_number(&cs[i..j].iter().collect::<String>())));
            i = j;
        } else {
            let j = (i..cs.len()).find(|&j| cs[j].is_ascii_digit() || is_cjk_num(cs[j])).unwrap_or(cs.len());
            out.push(Tok::Text(cs[i..j].iter().collect()));
            i = j;
        }
    }
    out
}

/// Subpages in reading order: first by where the main page links to them, then naturally sorted.
pub fn order_subpages(main_title: &str, main_wikitext: &str, mut subs: Vec<String>) -> Vec<String> {
    subs.sort_by(|a, b| natural_key(a).cmp(&natural_key(b)));
    let hay = main_wikitext.replace('_', " ");
    let pos = |s: &str| -> Option<usize> {
        let suffix = s.strip_prefix(main_title).unwrap_or(s);
        hay.find(&format!("[[{s}")).or_else(|| hay.find(&format!("[[{}", suffix))).or_else(|| hay.find(s))
    };
    let mut linked: Vec<(usize, String)> = Vec::new();
    let mut rest = Vec::new();
    for s in subs {
        match pos(&s) {
            Some(p) => linked.push((p, s)),
            None => rest.push(s),
        }
    }
    linked.sort_by(|a, b| a.0.cmp(&b.0).then(Ordering::Equal));
    linked.into_iter().map(|x| x.1).chain(rest).collect()
}

// ---------------------------------------------------------------- resolving one entry

#[derive(Debug)]
pub struct Doc {
    pub source: String,
    pub url: String,
    pub page_title: String,
    pub revision_id: u64,
    pub revision_timestamp: String,
    pub wikitext: String,
    pub text: String,
    pub edition: Option<Value>,
    pub subpages: Vec<Value>,
    pub licence: String,
    pub resolved_via: String,
    pub truncated: bool,
}

#[derive(Debug)]
pub enum Outcome {
    Resolved(Box<Doc>),
    Missing { tried: Vec<String> },
    Ambiguous { candidates: Vec<String> },
    Unsupported(String),
    Failed(String),
}

fn layout_for(e: &Entry) -> Layout {
    if matches!(e.shelf.as_str(), "verse" | "modern-poetry") {
        Layout::Verse
    } else {
        Layout::Prose
    }
}

fn licence_for(e: &Entry, source: &str) -> String {
    match source {
        "ohchr" => format!("{} Source: OHCHR.", e.pd_basis),
        "law" => format!("Statute text, not protected by copyright (Copyright Act art. 7); transcription from Wikisource (CC BY-SA 4.0). {}", e.pd_basis),
        _ => format!("Original work: public domain ({}). Transcription: Wikisource contributors, CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/).", e.pd_basis),
    }
}

fn root_of(t: &str) -> &str {
    t.split('/').next().unwrap_or(t)
}

enum Resolve {
    Found { main: Option<Page>, title: String, via: &'static str },
    Missing { tried: Vec<String> },
    Ambiguous(Vec<String>),
}

/// Direct title lookup first; otherwise the entry's search terms.
fn resolve_title(wiki: &Wiki, e: &Entry) -> Result<Resolve> {
    let mut tried = vec![format!("{}:{}", wiki.host, e.source_title)];
    let main = wiki.get_page(&e.source_title)?;
    if let Some(p) = main {
        return Ok(Resolve::Found { title: p.title.clone(), main: Some(p), via: "title" });
    }
    if !wiki.subpages(&e.source_title)?.is_empty() {
        return Ok(Resolve::Found { title: e.source_title.clone(), main: None, via: "title" });
    }
    let mut cands: Vec<String> = Vec::new();
    let mut first_hit: Option<String> = None;
    for term in &e.search {
        tried.push(format!("{}:search:{term}", wiki.host));
        match wiki.search(term) {
            Ok(r) => {
                if first_hit.is_none() && r.first().map(|t| t == term).unwrap_or(false) {
                    first_hit = r.first().cloned();
                }
                for t in r {
                    if !cands.contains(&t) {
                        cands.push(t);
                    }
                }
            }
            Err(err) => log::warn!("{}: search {term:?} on {} failed: {err:#}", e.id, wiki.host),
        }
    }
    if cands.is_empty() {
        return Ok(Resolve::Missing { tried });
    }
    let roots: Vec<&str> = {
        let mut r: Vec<&str> = cands.iter().map(|c| root_of(c)).collect();
        r.dedup();
        let mut seen = Vec::new();
        r.retain(|x| {
            let dup = seen.contains(x);
            seen.push(*x);
            !dup
        });
        r
    };
    let chosen: Option<String> = if cands.len() == 1 {
        Some(cands[0].clone())
    } else if roots.len() == 1 && cands.iter().any(|c| c == roots[0]) {
        Some(roots[0].to_string())
    } else {
        first_hit
    };
    match chosen {
        Some(t) => {
            let main = wiki.get_page(&t)?;
            Ok(Resolve::Found { title: main.as_ref().map(|p| p.title.clone()).unwrap_or(t), main, via: "search" })
        }
        None => Ok(Resolve::Ambiguous(cands.into_iter().take(10).map(|c| format!("{}:{c}", wiki.host)).collect())),
    }
}

/// Fetch a resolved page plus its subpages and assemble the raw document.
fn build_doc(wiki: &Wiki, e: &Entry, title: String, main: Option<Page>, via: &str, max_subpages: usize) -> Result<Outcome> {
    let layout = layout_for(e);
    let nospace = e.script == "hanmun";
    let main_text = main.as_ref().map(|p| wt::clean(&p.content, layout, nospace));
    let main_title = main.as_ref().map(|p| p.title.clone()).unwrap_or_else(|| title.clone());
    let subs = wiki.subpages(&main_title)?;
    let mut subs = order_subpages(&main_title, main.as_ref().map(|p| p.content.as_str()).unwrap_or(""), subs);
    let truncated = subs.len() > max_subpages;
    if truncated {
        log::warn!("{}: {} subpages, keeping the first {max_subpages}", e.id, subs.len());
        subs.truncate(max_subpages);
    }
    let fetched = wiki.get_pages(&subs)?;
    let mut text_parts: Vec<String> = Vec::new();
    let mut wikitext_parts: Vec<String> = Vec::new();
    let mut edition = main_text.as_ref().and_then(|c| c.edition.clone());
    let mut sub_meta = Vec::new();
    if let (Some(p), Some(c)) = (&main, &main_text) {
        wikitext_parts.push(p.content.clone());
        if !c.text.is_empty() {
            text_parts.push(c.text.clone());
        }
    }
    let (mut rev, mut ts) = main.as_ref().map(|p| (p.revision_id, p.timestamp.clone())).unwrap_or((0, String::new()));
    for (req, page) in fetched {
        let Some(p) = page else {
            log::warn!("{}: subpage {req} vanished", e.id);
            continue;
        };
        let c = wt::clean(&p.content, layout, nospace);
        if rev == 0 {
            rev = p.revision_id;
            ts = p.timestamp.clone();
        }
        if edition.is_none() {
            edition = c.edition.clone();
        }
        sub_meta.push(json!({"title": p.title, "revision_id": p.revision_id, "revision_timestamp": p.timestamp}));
        wikitext_parts.push(format!("<!-- subpage: {} -->\n{}", p.title, p.content));
        if c.text.is_empty() {
            continue;
        }
        let label = p.title.strip_prefix(&format!("{main_title}/")).unwrap_or(&p.title).to_string();
        text_parts.push(if c.text.starts_with("## ") { c.text } else { format!("## {label}\n\n{}", c.text) });
    }
    let text = text_parts.join("\n\n");
    if text.trim().is_empty() {
        return Ok(Outcome::Failed(format!("page {main_title:?} resolved but its cleaned text is empty")));
    }
    Ok(Outcome::Resolved(Box::new(Doc {
        source: if wiki.host == HOST_ZH { "wikisource-zh".into() } else if e.source == "law" { "law".into() } else { "wikisource-ko".into() },
        url: wiki_page_url(&wiki.host, &main_title),
        page_title: main_title,
        revision_id: rev,
        revision_timestamp: ts,
        wikitext: wikitext_parts.join("\n\n"),
        text,
        edition,
        subpages: sub_meta,
        licence: licence_for(e, &e.source),
        resolved_via: via.to_string(),
        truncated,
    })))
}

fn try_host(http: &dyn Http, host: &str, e: &Entry, max_subpages: usize) -> Result<Outcome> {
    let wiki = Wiki { http, host: host.to_string() };
    Ok(match resolve_title(&wiki, e)? {
        Resolve::Found { main, title, via } => build_doc(&wiki, e, title, main, via, max_subpages)?,
        Resolve::Missing { tried } => Outcome::Missing { tried },
        Resolve::Ambiguous(c) => Outcome::Ambiguous { candidates: c },
    })
}

fn hangul_count(s: &str) -> usize {
    s.chars().filter(|c| ('\u{AC00}'..='\u{D7A3}').contains(c)).count()
}

fn try_ohchr(http: &dyn Http, e: &Entry) -> Result<Outcome> {
    let url = e.url.clone().ok_or_else(|| anyhow!("no url for ohchr entry"))?;
    let html = http.get(&url)?;
    let body = texts_html::element_by_name(&html, "main").or_else(|| texts_html::element_by_name(&html, "article")).unwrap_or_else(|| html.clone());
    let text = texts_html::html_to_text(&body);
    if hangul_count(&text) < 400 {
        return Ok(Outcome::Missing { tried: vec![format!("{url} (no Korean article text found)")] });
    }
    Ok(Outcome::Resolved(Box::new(Doc {
        source: "ohchr".into(),
        url,
        page_title: texts_html::title_tag(&html).unwrap_or_default(),
        revision_id: 0,
        revision_timestamp: String::new(),
        wikitext: html,
        text,
        edition: None,
        subpages: vec![],
        licence: licence_for(e, "ohchr"),
        resolved_via: "url".into(),
        truncated: false,
    })))
}

pub fn fetch_entry(http: &dyn Http, e: &Entry, max_subpages: usize) -> Outcome {
    let hosts: Vec<&str> = match e.source.as_str() {
        "wikisource-ko" if e.script != "hangul" => vec![HOST_KO, HOST_ZH],
        "wikisource-zh" => vec![HOST_ZH, HOST_KO],
        "wikisource-ko" | "law" | "ohchr" => vec![HOST_KO],
        other => return Outcome::Unsupported(format!("source {other:?} is not fetched by fetch-texts")),
    };
    let mut tried: Vec<String> = Vec::new();
    let mut ambiguous: Option<Vec<String>> = None;
    let mut errors: Vec<String> = Vec::new();
    if e.source == "ohchr" {
        match try_ohchr(http, e) {
            Ok(Outcome::Resolved(d)) => return Outcome::Resolved(d),
            Ok(Outcome::Missing { tried: t }) => tried.extend(t),
            Ok(_) => {}
            Err(err) => errors.push(format!("ohchr: {err:#}")),
        }
    }
    for host in hosts {
        match try_host(http, host, e, max_subpages) {
            Ok(Outcome::Resolved(d)) => return Outcome::Resolved(d),
            Ok(Outcome::Missing { tried: t }) => tried.extend(t),
            Ok(Outcome::Ambiguous { candidates }) => ambiguous = ambiguous.or(Some(candidates)),
            Ok(Outcome::Failed(m)) => errors.push(m),
            Ok(Outcome::Unsupported(_)) => {}
            Err(err) => errors.push(format!("{host}: {err:#}")),
        }
    }
    if let Some(candidates) = ambiguous {
        return Outcome::Ambiguous { candidates };
    }
    if !errors.is_empty() {
        return Outcome::Failed(errors.join("; "));
    }
    Outcome::Missing { tried }
}

pub fn doc_json(id: &str, d: &Doc) -> Value {
    let mut m = Map::new();
    m.insert("id".into(), json!(id));
    m.insert("source".into(), json!(d.source));
    m.insert("url".into(), json!(d.url));
    m.insert("page_title".into(), json!(d.page_title));
    m.insert("revision_id".into(), json!(d.revision_id));
    m.insert("revision_timestamp".into(), json!(d.revision_timestamp));
    m.insert("fetched_at".into(), json!(now_iso()));
    m.insert("licence".into(), json!(d.licence));
    m.insert("wikitext".into(), json!(d.wikitext));
    m.insert("text".into(), json!(d.text));
    m.insert("edition".into(), d.edition.clone().unwrap_or(Value::Null));
    m.insert("subpages".into(), Value::Array(d.subpages.clone()));
    m.insert("resolved_via".into(), json!(d.resolved_via));
    m.insert("truncated".into(), json!(d.truncated));
    Value::Object(m)
}

// ---------------------------------------------------------------- English translations

#[derive(Debug)]
pub enum EnOutcome {
    Done { url: String, text: String, id: String },
    Missing(String),
    Ambiguous(Vec<String>),
    Failed(String),
}

/// Remove the Project Gutenberg licence header/footer.
pub fn strip_gutenberg(raw: &str) -> String {
    let start = raw.find("*** START OF").and_then(|p| raw[p..].find('\n').map(|n| p + n + 1)).unwrap_or(0);
    let end = raw.find("*** END OF").unwrap_or(raw.len());
    raw[start..end.max(start)].trim().to_string()
}

pub fn fetch_english(http: &dyn Http, p: &EnglishPd) -> EnOutcome {
    match p.source.as_str() {
        "gutenberg" => {
            let n = match p.ebook {
                Some(n) => n,
                None => {
                    let q = p.search.clone().unwrap_or_else(|| p.title.clone());
                    let body = match http.get(&format!("https://gutendex.com/books/?search={}", urlencode(&q))) {
                        Ok(b) => b,
                        Err(e) => return EnOutcome::Failed(format!("{e:#}")),
                    };
                    let v: Value = match serde_json::from_str(&body) {
                        Ok(v) => v,
                        Err(e) => return EnOutcome::Failed(format!("gutendex JSON: {e}")),
                    };
                    let items: Vec<(u64, String)> = v["results"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|x| Some((x["id"].as_u64()?, format!("{} — {}", x["title"].as_str()?, x["authors"][0]["name"].as_str().unwrap_or("?"))))).collect())
                        .unwrap_or_default();
                    let want = p.title.to_lowercase();
                    let exact: Vec<&(u64, String)> = items.iter().filter(|(_, t)| t.to_lowercase().starts_with(&want)).collect();
                    match (items.len(), exact.len()) {
                        (0, _) => return EnOutcome::Missing(format!("gutendex search {q:?}")),
                        (1, _) => items[0].0 as u32,
                        (_, 1) => exact[0].0 as u32,
                        _ => return EnOutcome::Ambiguous(items.iter().map(|(i, t)| format!("gutenberg:{i} {t}")).collect()),
                    }
                }
            };
            for url in [format!("https://www.gutenberg.org/cache/epub/{n}/pg{n}.txt"), format!("https://www.gutenberg.org/ebooks/{n}.txt.utf-8")] {
                if let Ok(t) = http.get(&url) {
                    return EnOutcome::Done { url, text: strip_gutenberg(&t), id: format!("gutenberg:{n}") };
                }
            }
            EnOutcome::Failed(format!("could not download Gutenberg ebook {n}"))
        }
        "archive-org" => {
            let ident = match &p.identifier {
                Some(i) => i.clone(),
                None => {
                    let queries = [format!("title:(\"{}\") AND mediatype:texts", p.title), format!("({}) AND mediatype:texts", p.search.clone().unwrap_or_else(|| p.title.clone()))];
                    let mut items: Vec<(String, String)> = Vec::new();
                    for q in queries {
                        let url = format!("https://archive.org/advancedsearch.php?q={}&fl%5B%5D=identifier&fl%5B%5D=title&fl%5B%5D=year&rows=10&output=json", urlencode(&q));
                        let body = match http.get(&url) {
                            Ok(b) => b,
                            Err(e) => return EnOutcome::Failed(format!("{e:#}")),
                        };
                        let v: Value = match serde_json::from_str(&body) {
                            Ok(v) => v,
                            Err(e) => return EnOutcome::Failed(format!("archive.org JSON: {e}")),
                        };
                        items = v["response"]["docs"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|d| Some((d["identifier"].as_str()?.to_string(), format!("{} ({})", d["title"].as_str().unwrap_or("?"), d["year"].as_str().map(String::from).or_else(|| d["year"].as_i64().map(|y| y.to_string())).unwrap_or_default())))).collect())
                            .unwrap_or_default();
                        if !items.is_empty() {
                            break;
                        }
                    }
                    let by_year: Vec<&(String, String)> = items.iter().filter(|(_, t)| t.contains(&format!("({})", p.year))).collect();
                    match (items.len(), by_year.len()) {
                        (0, _) => return EnOutcome::Missing("archive.org advancedsearch".into()),
                        (1, _) => items[0].0.clone(),
                        (_, 1) => by_year[0].0.clone(),
                        _ => return EnOutcome::Ambiguous(items.iter().map(|(i, t)| format!("archive-org:{i} {t}")).collect()),
                    }
                }
            };
            let url = format!("https://archive.org/download/{ident}/{ident}_djvu.txt");
            match http.get(&url) {
                Ok(t) => EnOutcome::Done { url, text: t, id: format!("archive-org:{ident}") },
                Err(e) => EnOutcome::Failed(format!("{e:#}")),
            }
        }
        other => EnOutcome::Failed(format!("unsupported english_pd source {other:?}")),
    }
}

// ---------------------------------------------------------------- run

fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(d) = path.parent() {
        fs::create_dir_all(d)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn write_json(path: &Path, v: &Value) -> Result<()> {
    write_atomic(path, serde_json::to_string_pretty(v)?.as_bytes())
}

/// Replace report rows for `ids` (processed this run) and keep the rest of the previous report.
fn merge_rows(prev: &Value, key: &str, processed: &[String], fresh: Vec<Value>) -> Vec<Value> {
    let mut rows: Vec<Value> = prev[key].as_array().cloned().unwrap_or_default().into_iter().filter(|r| r["id"].as_str().map(|i| !processed.iter().any(|p| p == i)).unwrap_or(true)).collect();
    rows.extend(fresh);
    rows
}

pub struct RunSummary {
    pub report: Value,
}

pub fn run(opts: &Opts) -> Result<()> {
    let http = UreqHttp::new();
    let s = run_with(&http, opts)?;
    println!("{}", summary_text(&s.report));
    Ok(())
}

pub fn summary_text(report: &Value) -> String {
    let n = |k: &str| report[k].as_array().map(|a| a.len()).unwrap_or(0);
    let ids = |k: &str| report[k].as_array().map(|a| a.iter().filter_map(|r| r["id"].as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
    let mut s = format!("fetch-texts: resolved {} | missing {} | ambiguous {} | unsupported {} | errors {}\n", n("resolved"), n("missing"), n("ambiguous"), n("unsupported"), n("errors"));
    for k in ["missing", "ambiguous", "unsupported", "errors"] {
        if n(k) > 0 {
            s.push_str(&format!("  {k}: {}\n", ids(k)));
        }
    }
    let en = |k: &str| report["english"][k].as_array().map(|a| a.len()).unwrap_or(0);
    s.push_str(&format!("english: resolved {} | missing {} | ambiguous {} | errors {}", en("resolved"), en("missing"), en("ambiguous"), en("errors")));
    s
}

pub fn run_with(http: &dyn Http, opts: &Opts) -> Result<RunSummary> {
    let cat: Catalog = texts_catalog::load(&opts.catalog)?;
    let problems = texts_catalog::validate(&cat);
    if !problems.is_empty() {
        bail!("catalogue is invalid:\n{}", problems.join("\n"));
    }
    fs::create_dir_all(&opts.out)?;
    for id in &opts.only {
        if !cat.texts.iter().any(|e| &e.id == id) {
            log::warn!("--only: unknown id {id:?}");
        }
    }
    let selected: Vec<&Entry> = cat.texts.iter().filter(|e| opts.only.is_empty() || opts.only.contains(&e.id)).collect();
    let report_path = opts.out.join("_report.json");
    let prev: Value = fs::read_to_string(&report_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);

    let (mut resolved, mut missing, mut ambiguous, mut unsupported, mut errors) = (vec![], vec![], vec![], vec![], vec![]);
    let mut processed: Vec<String> = Vec::new();
    for (n, e) in selected.iter().enumerate() {
        let path = opts.out.join(format!("{}.json", e.id));
        if path.exists() && !opts.force {
            log::info!("[{}/{}] {}: already fetched, skipping", n + 1, selected.len(), e.id);
            continue;
        }
        log::info!("[{}/{}] {} ({})", n + 1, selected.len(), e.id, e.source_title);
        processed.push(e.id.clone());
        match fetch_entry(http, e, opts.max_subpages) {
            Outcome::Resolved(d) => {
                let wrote = write_json(&path, &doc_json(&e.id, &d));
                match wrote {
                    Ok(()) => {
                        log::info!("  ok: {} ({} chars, {} subpages, via {})", d.page_title, d.text.chars().count(), d.subpages.len(), d.resolved_via);
                        resolved.push(json!({"id": e.id, "source": d.source, "page_title": d.page_title, "url": d.url, "revision_id": d.revision_id, "chars": d.text.chars().count(), "subpages": d.subpages.len(), "resolved_via": d.resolved_via, "truncated": d.truncated}));
                    }
                    Err(err) => errors.push(json!({"id": e.id, "error": format!("writing output: {err:#}")})),
                }
            }
            Outcome::Missing { tried } => {
                log::warn!("  {}: not found (tried {})", e.id, tried.join(", "));
                missing.push(json!({"id": e.id, "source_title": e.source_title, "tried": tried}));
            }
            Outcome::Ambiguous { candidates } => {
                log::warn!("  {}: ambiguous, candidates: {}", e.id, candidates.join(" | "));
                ambiguous.push(json!({"id": e.id, "source_title": e.source_title, "candidates": candidates}));
            }
            Outcome::Unsupported(m) => unsupported.push(json!({"id": e.id, "reason": m})),
            Outcome::Failed(m) => {
                log::warn!("  {}: failed: {m}", e.id);
                errors.push(json!({"id": e.id, "error": m}));
            }
        }
    }

    // English translations (deduplicated by ref)
    let en_dir = opts.out.join("en");
    let mut done_refs: Vec<String> = Vec::new();
    let (mut en_ok, mut en_missing, mut en_amb, mut en_err) = (vec![], vec![], vec![], vec![]);
    for e in &selected {
        let Some(p) = &e.english_pd else { continue };
        if done_refs.contains(&p.reference) {
            continue;
        }
        done_refs.push(p.reference.clone());
        let txt = en_dir.join(format!("{}.txt", p.reference));
        if txt.exists() && !opts.force {
            continue;
        }
        log::info!("english: {} ({})", p.reference, p.title);
        match fetch_english(http, p) {
            EnOutcome::Done { url, text, id } => {
                let side = json!({"ref": p.reference, "title": p.title, "translator": p.translator, "year": p.year, "source": p.source, "identifier": id, "url": url, "fetched_at": now_iso(), "licence": format!("Public domain (published {}; translator {} died long ago). Full text from {}.", p.year, p.translator, p.source), "chars": text.chars().count(), "for": e.id});
                let r = write_atomic(&txt, text.as_bytes()).and_then(|_| write_json(&en_dir.join(format!("{}.json", p.reference)), &side));
                match r {
                    Ok(()) => en_ok.push(json!({"id": p.reference, "url": url, "chars": text.chars().count()})),
                    Err(err) => en_err.push(json!({"id": p.reference, "error": format!("{err:#}")})),
                }
            }
            EnOutcome::Missing(m) => en_missing.push(json!({"id": p.reference, "tried": m})),
            EnOutcome::Ambiguous(c) => en_amb.push(json!({"id": p.reference, "candidates": c})),
            EnOutcome::Failed(m) => en_err.push(json!({"id": p.reference, "error": m})),
        }
    }

    let ids_en = done_refs.clone();
    let english = json!({
        "resolved": merge_rows(&prev["english"], "resolved", &ids_en, en_ok),
        "missing": merge_rows(&prev["english"], "missing", &ids_en, en_missing),
        "ambiguous": merge_rows(&prev["english"], "ambiguous", &ids_en, en_amb),
        "errors": merge_rows(&prev["english"], "errors", &ids_en, en_err),
    });
    let report = json!({
        "generated_at": now_iso(),
        "resolved": merge_rows(&prev, "resolved", &processed, resolved),
        "missing": merge_rows(&prev, "missing", &processed, missing),
        "ambiguous": merge_rows(&prev, "ambiguous", &processed, ambiguous),
        "unsupported": merge_rows(&prev, "unsupported", &processed, unsupported),
        "errors": merge_rows(&prev, "errors", &processed, errors),
        "english": english,
    });
    write_json(&report_path, &report)?;
    Ok(RunSummary { report })
}
