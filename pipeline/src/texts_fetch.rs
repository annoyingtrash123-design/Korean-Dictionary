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
use std::cell::RefCell;
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
    /// Do not write texts; write `_discover.json` with candidates for every entry instead.
    pub discover: bool,
}

/// Excerpt entries without `sections` keep only about this many characters.
pub const EXCERPT_CHARS: usize = 15_000;
const CONTENT_BATCH: usize = 25;

// ---------------------------------------------------------------- MediaWiki client

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub revision_id: u64,
    pub timestamp: String,
    pub content: String,
    /// The title that was asked for when it differs from `title` (a redirect was followed).
    pub redirected_from: Option<String>,
}

/// Pages fetched so far, keyed by (host, requested title); shared by the batch pre-pass and the per-entry work.
pub type Cache = RefCell<HashMap<(String, String), Option<Page>>>;

pub struct Wiki<'a> {
    pub http: &'a dyn Http,
    pub host: String,
    pub cache: Option<&'a Cache>,
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
                redirected_from: None,
            });
            pages.insert(title.to_string(), if p["missing"].as_bool().unwrap_or(false) || p["invalid"].as_bool().unwrap_or(false) { None } else { page });
        }
    }
    requested
        .iter()
        .map(|r| {
            let page = pages.get(&follow(&map, r)).cloned().flatten().map(|mut p| {
                if p.title.replace('_', " ") != r.replace('_', " ") {
                    p.redirected_from = Some(r.clone());
                }
                p
            });
            (r.clone(), page)
        })
        .collect()
}

pub fn parse_search(v: &Value) -> Vec<String> {
    parse_search_ns(v).into_iter().map(|x| x.0).collect()
}

/// (title, namespace) of `list=search` / `list=prefixsearch` results.
pub fn parse_search_ns(v: &Value) -> Vec<(String, i64)> {
    ["search", "prefixsearch"]
        .iter()
        .filter_map(|k| v["query"][k].as_array())
        .flatten()
        .filter_map(|x| Some((x["title"].as_str()?.to_string(), x["ns"].as_i64().unwrap_or(0))))
        .collect()
}

/// Returns (titles, continuation token).
pub fn parse_allpages(v: &Value) -> (Vec<String>, Option<String>) {
    let t = v["query"]["allpages"].as_array().map(|a| a.iter().filter_map(|x| x["title"].as_str().map(String::from)).collect()).unwrap_or_default();
    (t, v["continue"]["apcontinue"].as_str().map(String::from))
}

impl Wiki<'_> {
    fn api(&self, query: &str) -> Result<Value> {
        let url = format!("https://{}/w/api.php?{query}&maxlag=5&format=json&formatversion=2", self.host);
        let mut last = anyhow!("no attempt");
        for attempt in 0..6u64 {
            let body = self.http.get(&url)?;
            let v: Value = serde_json::from_str(&body).with_context(|| format!("bad JSON from {url}"))?;
            match v.get("error") {
                None => return Ok(v),
                Some(err) if matches!(err["code"].as_str(), Some("maxlag") | Some("ratelimited")) => {
                    let wait = 5 * (attempt + 1);
                    log::warn!("{}: server busy ({}), waiting {wait}s", self.host, err["code"]);
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    last = anyhow!("MediaWiki API error: {err}");
                }
                Some(err) => bail!("MediaWiki API error: {err}"),
            }
        }
        Err(last)
    }

    /// Batched title lookup (cached). Results follow the order of `titles`.
    pub fn get_pages(&self, titles: &[String]) -> Result<Vec<(String, Option<Page>)>> {
        let key = |t: &String| (self.host.clone(), t.clone());
        let todo: Vec<String> = match self.cache {
            Some(c) => {
                let c = c.borrow();
                let mut seen = Vec::new();
                for t in titles {
                    if !c.contains_key(&key(t)) && !seen.contains(t) {
                        seen.push(t.clone());
                    }
                }
                seen
            }
            None => titles.to_vec(),
        };
        let mut fresh: Vec<(String, Option<Page>)> = Vec::new();
        for chunk in todo.chunks(CONTENT_BATCH) {
            let t: Vec<String> = chunk.iter().map(|t| urlencode(t)).collect();
            let q = format!("action=query&prop=revisions&rvprop=content%7Cids%7Ctimestamp&rvslots=main&redirects=1&titles={}", t.join("%7C"));
            let v = self.api(&q)?;
            fresh.extend(parse_pages(&v, chunk));
        }
        if let Some(c) = self.cache {
            let mut c = c.borrow_mut();
            for (t, p) in fresh {
                c.insert((self.host.clone(), t), p);
            }
            Ok(titles.iter().map(|t| (t.clone(), c.get(&key(t)).cloned().flatten())).collect())
        } else {
            Ok(titles.iter().map(|t| (t.clone(), fresh.iter().find(|f| &f.0 == t).and_then(|f| f.1.clone()))).collect())
        }
    }

    pub fn get_page(&self, title: &str) -> Result<Option<Page>> {
        Ok(self.get_pages(&[title.to_string()])?.into_iter().next().and_then(|x| x.1))
    }

    pub fn search(&self, term: &str) -> Result<Vec<(String, i64)>> {
        let v = self.api(&format!("action=query&list=search&srsearch={}&srnamespace=0&srlimit=15", urlencode(term)))?;
        Ok(parse_search_ns(&v))
    }

    pub fn prefix_search(&self, term: &str) -> Result<Vec<(String, i64)>> {
        let v = self.api(&format!("action=query&list=prefixsearch&pssearch={}&psnamespace=0&pslimit=15", urlencode(term)))?;
        Ok(parse_search_ns(&v))
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
    subs.sort_by_key(|a| natural_key(a));
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

// ---------------------------------------------------------------- index / versions pages

const EDITION_WORDS: &[&str] = &["판", "본", "필사", "이본", "edition", "version"];
const VERSION_TEMPLATES: &[&str] = &["판본", "동음이의", "versions", "disambig", "다의어", "중의", "이본", "버전"];

/// Targets of `[[…]]` links (no files/categories/namespaced pages), `[[/x]]` expanded against `title`.
pub fn link_targets(wikitext: &str, title: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while let Some(p) = wikitext[i..].find("[[").map(|x| x + i) {
        let end = wikitext[p + 2..].find("]]").map(|x| x + p + 2).unwrap_or(wikitext.len());
        let inner = &wikitext[p + 2..end];
        let target = inner.split('|').next().unwrap_or("").split('#').next().unwrap_or("").trim().replace('_', " ");
        i = (p + 2).max(end);
        if target.is_empty() || (target.contains(':') && !target.starts_with('/')) {
            continue;
        }
        let full = if target.starts_with('/') { format!("{title}{target}") } else { target };
        if !out.contains(&full) {
            out.push(full);
        }
    }
    out
}

fn template_names(wikitext: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = wikitext[i..].find("{{").map(|x| x + i) {
        let rest = &wikitext[p + 2..];
        let end = rest.find(['|', '}', '\n']).unwrap_or(rest.len());
        out.push(rest[..end].trim().to_lowercase());
        i = p + 2;
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Normal,
    /// A versions/editions listing or an (almost) empty stub; `links` are the pages it points to.
    Index { links: Vec<String>, why: &'static str },
}

fn has_edition_word(t: &str) -> bool {
    let l = t.to_lowercase();
    EDITION_WORDS.iter().any(|w| l.contains(w))
}

/// Decide whether a page is a content page or an index of editions/versions.
pub fn classify(title: &str, wikitext: &str, text: &str, subs: &[String]) -> Kind {
    let chars = text.chars().count();
    let ver_tpl = template_names(wikitext).iter().any(|n| VERSION_TEMPLATES.iter().any(|v| n.contains(v)));
    let related: Vec<String> = link_targets(wikitext, title).into_iter().filter(|l| l != title && l.contains(title)).collect();
    let ed_links: Vec<String> = related.iter().filter(|l| has_edition_word(l)).cloned().collect();
    let ed_subs: Vec<String> = subs.iter().filter(|s| has_edition_word(s)).cloned().collect();
    let collect = |extra_subs: &[String]| {
        let mut v = related.clone();
        for s in extra_subs {
            if !v.contains(s) {
                v.push(s.clone());
            }
        }
        v
    };
    if ver_tpl {
        return Kind::Index { links: collect(subs), why: "versions template" };
    }
    if ed_links.len() + ed_subs.len() >= 2 && chars < 3000 {
        return Kind::Index { links: collect(&ed_subs), why: "links to several editions" };
    }
    if chars < 10 && subs.is_empty() {
        return Kind::Index { links: related, why: "page is almost empty" };
    }
    Kind::Normal
}

// ---------------------------------------------------------------- candidates

fn preview(wikitext: &str, n: usize) -> String {
    let c = wt::clean(wikitext, Layout::Prose, false).text;
    let src = if c.trim().is_empty() { wikitext.to_string() } else { c };
    src.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(n).collect()
}

/// Titles with their size in bytes and the first characters of plain text, for a human to choose from.
fn describe(wiki: &Wiki, items: &[(String, i64)], preview_len: usize) -> Result<Vec<Value>> {
    let titles: Vec<String> = items.iter().map(|x| x.0.clone()).collect();
    let pages = wiki.get_pages(&titles).unwrap_or_else(|err| {
        log::warn!("describing candidates on {} failed: {err:#}", wiki.host);
        titles.iter().map(|t| (t.clone(), None)).collect()
    });
    Ok(items
        .iter()
        .zip(pages)
        .map(|((t, ns), (_, p))| match p {
            Some(p) => json!({"host": wiki.host, "title": t, "namespace": ns, "bytes": p.content.len(), "preview": preview(&p.content, preview_len), "redirected_to": p.redirected_from.as_ref().map(|_| p.title.clone())}),
            None => json!({"host": wiki.host, "title": t, "namespace": ns, "bytes": 0, "preview": ""}),
        })
        .collect())
}

fn accepted_titles(e: &Entry) -> Vec<String> {
    let mut v = vec![e.source_title.clone()];
    v.extend(e.alt_titles.iter().cloned());
    v
}

fn dedupe(items: Vec<(String, i64)>, cap: usize) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    for it in items {
        if !out.iter().any(|o| o.0 == it.0) {
            out.push(it);
        }
    }
    out.truncate(cap);
    out
}

/// Prefix-search and full-text candidates (never auto-accepted). Returns (prefix, search-per-term).
type Hits = Vec<(String, i64)>;

fn search_candidates(wiki: &Wiki, e: &Entry) -> (Hits, Vec<(String, Hits)>) {
    let mut prefix = Vec::new();
    for t in accepted_titles(e) {
        match wiki.prefix_search(&t) {
            Ok(r) => prefix.extend(r),
            Err(err) => log::warn!("{}: prefixsearch {t:?} on {} failed: {err:#}", e.id, wiki.host),
        }
    }
    let mut searched = Vec::new();
    for term in &e.search {
        match wiki.search(term) {
            Ok(r) => searched.push((term.clone(), r)),
            Err(err) => log::warn!("{}: search {term:?} on {} failed: {err:#}", e.id, wiki.host),
        }
    }
    (dedupe(prefix, 15), searched)
}

fn gather_candidates(wiki: &Wiki, e: &Entry, first: Vec<(String, i64)>) -> Result<Vec<Value>> {
    let (prefix, searched) = search_candidates(wiki, e);
    let mut all = first;
    all.extend(prefix);
    for (_, r) in searched {
        all.extend(r.into_iter().take(15));
    }
    describe(wiki, &dedupe(all, 40), 120)
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
    /// Excerpt entry without `sections`: the text was cut to ~15k characters.
    pub needs_sections: bool,
}

#[derive(Debug)]
pub enum Outcome {
    Resolved(Box<Doc>),
    Missing { tried: Vec<String> },
    /// Nothing was accepted; `candidates` = [{host, title, namespace, bytes, preview}].
    Ambiguous { candidates: Vec<Value> },
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

/// Keep only the paragraphs under headings whose title equals/contains one of `names`.
/// One `<pages index="…" from=… to=… fromsection="…" tosection="…" />` transclusion.
#[derive(Debug, Clone, PartialEq)]
pub struct Transclusion {
    pub tag: String,
    pub index: String,
    pub from: u32,
    pub to: u32,
    pub fromsection: Option<String>,
    pub tosection: Option<String>,
}

fn tag_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut at = 0;
    while let Some(i) = lower[at..].find(name).map(|i| i + at) {
        at = i + name.len();
        let before_ok = i == 0 || !lower.as_bytes()[i - 1].is_ascii_alphanumeric();
        let rest = tag[at..].trim_start();
        if !before_ok || !rest.starts_with('=') {
            continue;
        }
        let v = rest[1..].trim_start();
        return Some(match v.chars().next() {
            Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or("").to_string(),
            _ => v.split(|c: char| c.is_whitespace() || c == '/' || c == '>').next().unwrap_or("").to_string(),
        });
    }
    None
}

/// The `<pages …/>` transclusions of a page (ProofreadPage scans), in order.
pub fn transclusions(content: &str) -> Vec<Transclusion> {
    let lower = content.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(i) = lower[at..].find("<pages").map(|i| i + at) {
        let Some(end) = lower[i..].find('>').map(|j| i + j + 1) else { break };
        at = end;
        let tag = &content[i..end];
        let (Some(index), Some(from)) = (tag_attr(tag, "index"), tag_attr(tag, "from").and_then(|v| v.parse().ok())) else { continue };
        let to = tag_attr(tag, "to").and_then(|v| v.parse().ok()).unwrap_or(from);
        out.push(Transclusion { tag: tag.to_string(), index, from, to: to.max(from), fromsection: tag_attr(tag, "fromsection"), tosection: tag_attr(tag, "tosection") });
    }
    out
}

/// The part of a scan page between `<section begin="name"/>` and `<section end="name"/>`
/// (the whole page when the section is not marked).
pub fn page_section(page: &str, name: &str) -> String {
    let find = |kind: &str| {
        let lower = page.to_ascii_lowercase();
        let mut at = 0;
        while let Some(i) = lower[at..].find("<section").map(|i| i + at) {
            let end = lower[i..].find('>').map(|j| i + j + 1).unwrap_or(page.len());
            at = end;
            if tag_attr(&page[i..end], kind).as_deref() == Some(name) {
                return Some((i, end));
            }
        }
        None
    };
    match (find("begin"), find("end")) {
        (Some((_, b)), Some((e, _))) if b <= e => page[b..e].to_string(),
        (Some((_, b)), None) => page[b..].to_string(),
        _ => page.to_string(),
    }
}

/// Replace `<pages …/>` transclusions with the transcribed text of those scan pages, so pages
/// that only embed proofread scans (many sijo) yield their text.
fn expand_pages(wiki: &Wiki, content: &str) -> Result<String> {
    let tr = transclusions(content);
    if tr.is_empty() {
        return Ok(content.to_string());
    }
    let mut out = content.to_string();
    for t in tr {
        let titles: Vec<String> = (t.from..=t.to.min(t.from + 20)).map(|n| format!("Page:{}/{n}", t.index)).collect();
        let pages = wiki.get_pages(&titles)?;
        let last = pages.len().saturating_sub(1);
        let mut text = String::new();
        for (k, (_, page)) in pages.iter().enumerate() {
            let Some(p) = page else { continue };
            let mut part = p.content.clone();
            if k == 0 {
                if let Some(s) = &t.fromsection {
                    part = page_section(&part, s);
                }
            }
            if k == last && k > 0 {
                if let Some(s) = &t.tosection {
                    part = page_section(&part, s);
                }
            }
            text.push_str(&part);
            text.push('\n');
        }
        if text.trim().is_empty() {
            log::warn!("transclusion {} found no scan text", t.tag);
        }
        out = out.replacen(&t.tag, &format!("\n{text}\n"), 1);
    }
    Ok(out)
}

pub fn filter_headings(text: &str, names: &[String]) -> String {
    let mut keep = false;
    let mut out: Vec<&str> = Vec::new();
    for para in text.split("\n\n") {
        if let Some(h) = para.strip_prefix("## ") {
            keep = names.iter().any(|n| h.trim() == n || h.contains(n.as_str()));
        }
        if keep {
            out.push(para);
        }
    }
    out.join("\n\n")
}

/// Cut `text` to about `limit` characters at a paragraph boundary.
pub fn truncate_paragraphs(text: &str, limit: usize) -> (String, bool) {
    if text.chars().count() <= limit {
        return (text.to_string(), false);
    }
    let mut out = String::new();
    for para in text.split("\n\n") {
        if !out.is_empty() && out.chars().count() + para.chars().count() > limit {
            break;
        }
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(para);
        if out.chars().count() > limit {
            out = out.chars().take(limit).collect();
            break;
        }
    }
    (out, true)
}

fn section_match(main_title: &str, sub: &str, names: &[String]) -> bool {
    let suffix = sub.strip_prefix(&format!("{main_title}/")).unwrap_or(sub);
    names.iter().any(|n| n == suffix || n == sub)
}

/// Fetch a resolved page plus its subpages and assemble the raw document.
fn build_doc(wiki: &Wiki, e: &Entry, title: String, main: Option<Page>, via: &str, max_subpages: usize) -> Result<Outcome> {
    let main = match main {
        Some(mut p) => {
            p.content = expand_pages(wiki, &p.content)?;
            Some(p)
        }
        None => None,
    };
    let layout = layout_for(e);
    let nospace = e.script == "hanmun";
    let limited = e.excerpt && e.sections.is_empty();
    let mut main_text = main.as_ref().map(|p| wt::clean(&p.content, layout, nospace));
    let main_title = main.as_ref().map(|p| p.title.clone()).unwrap_or_else(|| title.clone());
    let subs = wiki.subpages(&main_title)?;
    let mut subs = order_subpages(&main_title, main.as_ref().map(|p| p.content.as_str()).unwrap_or(""), subs);
    if !e.sections.is_empty() {
        let before = subs.len();
        subs.retain(|s| section_match(&main_title, s, &e.sections));
        if let Some(c) = main_text.as_mut() {
            c.text = filter_headings(&c.text, &e.sections);
        }
        log::info!("{}: sections {:?} keep {}/{before} subpages", e.id, e.sections, subs.len());
    }
    let mut truncated = subs.len() > max_subpages;
    if truncated {
        log::warn!("{}: {} subpages, keeping the first {max_subpages}", e.id, subs.len());
        subs.truncate(max_subpages);
    }
    let mut text_parts: Vec<String> = Vec::new();
    let mut wikitext_parts: Vec<String> = Vec::new();
    let mut edition = main_text.as_ref().and_then(|c| c.edition.clone());
    let mut sub_meta = Vec::new();
    let mut total_chars = 0usize;
    if let (Some(p), Some(c)) = (&main, &main_text) {
        wikitext_parts.push(p.content.clone());
        if !c.text.is_empty() {
            total_chars += c.text.chars().count();
            text_parts.push(c.text.clone());
        }
    }
    let (mut rev, mut ts) = main.as_ref().map(|p| (p.revision_id, p.timestamp.clone())).unwrap_or((0, String::new()));
    let step = if limited { 3 } else { 10 };
    let mut needs_sections = false;
    for chunk in subs.chunks(step) {
        if limited && total_chars >= EXCERPT_CHARS {
            needs_sections = true;
            break;
        }
        for (req, page) in wiki.get_pages(chunk)? {
            let Some(p) = page else {
                log::warn!("{}: subpage {req} vanished", e.id);
                continue;
            };
            let c = wt::clean(&expand_pages(wiki, &p.content)?, layout, nospace);
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
            total_chars += c.text.chars().count();
            let label = p.title.strip_prefix(&format!("{main_title}/")).unwrap_or(&p.title).to_string();
            text_parts.push(if c.text.starts_with("## ") { c.text } else { format!("## {label}\n\n{}", c.text) });
        }
    }
    let mut text = text_parts.join("\n\n");
    if limited {
        let (t, cut) = truncate_paragraphs(&text, EXCERPT_CHARS);
        text = t;
        needs_sections |= cut;
        truncated |= needs_sections;
    }
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
        needs_sections,
    })))
}

/// Only exact titles (`source_title`, then `alt_titles`; redirects followed) are accepted.
/// Anything else - search hits, edition lists - is reported as candidates for a human to pin.
fn try_host(http: &dyn Http, cache: Option<&Cache>, host: &str, e: &Entry, max_subpages: usize) -> Result<Outcome> {
    let wiki = Wiki { http, host: host.to_string(), cache };
    let mut found: Option<(String, Option<Page>)> = None;
    for t in accepted_titles(e) {
        let main = match wiki.get_page(&t)? {
            // scan transclusions first, so a page that only embeds proofread scans is not "empty"
            Some(mut p) => {
                p.content = expand_pages(&wiki, &p.content)?;
                Some(p)
            }
            None => None,
        };
        if main.is_some() || !wiki.subpages(&t)?.is_empty() {
            found = Some((t, main));
            break;
        }
    }
    let Some((title, main)) = found else {
        let cands = gather_candidates(&wiki, e, vec![])?;
        return Ok(if cands.is_empty() {
            Outcome::Missing { tried: accepted_titles(e).into_iter().map(|t| format!("{host}:{t}")).collect() }
        } else {
            Outcome::Ambiguous { candidates: cands }
        });
    };
    if let Some(p) = &main {
        let subs = wiki.subpages(&p.title)?;
        let c = wt::clean(&p.content, layout_for(e), e.script == "hanmun");
        if let Kind::Index { links, why } = classify(&p.title, &p.content, &c.text, &subs) {
            log::warn!("{}: {:?} looks like an index page ({why})", e.id, p.title);
            let pick = e.prefer_edition.as_ref().and_then(|pref| links.iter().find(|l| l.contains(pref.as_str())).cloned());
            return match pick {
                Some(t) => match wiki.get_page(&t)? {
                    Some(target) => build_doc(&wiki, e, target.title.clone(), Some(target), "prefer_edition", max_subpages),
                    None => Ok(Outcome::Failed(format!("preferred edition page {t:?} does not exist"))),
                },
                None => {
                    let items: Vec<(String, i64)> = links.into_iter().map(|l| (l, 0)).collect();
                    let mut cands = describe(&wiki, &dedupe(items, 40), 120)?;
                    cands.insert(0, json!({"host": host, "title": p.title, "namespace": 0, "bytes": p.content.len(), "preview": preview(&p.content, 120), "note": format!("index page: {why}")}));
                    Ok(Outcome::Ambiguous { candidates: cands })
                }
            };
        }
    }
    let via = if main.as_ref().map(|p| p.redirected_from.is_some()).unwrap_or(false) { "redirect" } else { "title" };
    build_doc(&wiki, e, title, main, via, max_subpages)
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
        needs_sections: false,
    })))
}

fn hosts_for(e: &Entry) -> Option<Vec<&'static str>> {
    Some(match e.source.as_str() {
        "wikisource-ko" if e.script != "hangul" => vec![HOST_KO, HOST_ZH],
        "wikisource-zh" => vec![HOST_ZH, HOST_KO],
        "wikisource-ko" | "law" | "ohchr" => vec![HOST_KO],
        _ => return None,
    })
}

pub fn fetch_entry(http: &dyn Http, e: &Entry, max_subpages: usize) -> Outcome {
    fetch_entry_cached(http, None, e, max_subpages)
}

pub fn fetch_entry_cached(http: &dyn Http, cache: Option<&Cache>, e: &Entry, max_subpages: usize) -> Outcome {
    let Some(hosts) = hosts_for(e) else {
        return Outcome::Unsupported(format!("source {:?} is not fetched by fetch-texts", e.source));
    };
    let mut tried: Vec<String> = Vec::new();
    let mut ambiguous: Option<Vec<Value>> = None;
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
        match try_host(http, cache, host, e, max_subpages) {
            Ok(Outcome::Resolved(d)) => return Outcome::Resolved(d),
            Ok(Outcome::Missing { tried: t }) => tried.extend(t),
            Ok(Outcome::Ambiguous { candidates }) => match &mut ambiguous {
                Some(a) => a.extend(candidates),
                None => ambiguous = Some(candidates),
            },
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
    m.insert("needs_sections".into(), json!(d.needs_sections));
    Value::Object(m)
}

// ---------------------------------------------------------------- discover

fn discover_host(http: &dyn Http, cache: Option<&Cache>, host: &str, e: &Entry) -> Result<Value> {
    let wiki = Wiki { http, host: host.to_string(), cache };
    let titles = accepted_titles(e);
    let mut exact = Vec::new();
    for (req, page) in wiki.get_pages(&titles)? {
        exact.push(match page {
            None => json!({"requested": req, "found": false}),
            Some(p) => {
                let subs = wiki.subpages(&p.title)?;
                let c = wt::clean(&p.content, layout_for(e), e.script == "hanmun");
                let kind = classify(&p.title, &p.content, &c.text, &subs);
                let (kind_s, links) = match &kind {
                    Kind::Normal => ("content", vec![]),
                    Kind::Index { links, why } => (*why, links.clone()),
                };
                let versions = describe(&wiki, &links.into_iter().take(30).map(|l| (l, 0)).collect::<Vec<_>>(), 120)?;
                json!({"requested": req, "found": true, "title": p.title, "redirected_from": p.redirected_from, "bytes": p.content.len(), "chars": c.text.chars().count(), "revision_id": p.revision_id, "preview": preview(&p.content, 200), "kind": kind_s, "versions_links": versions, "subpage_count": subs.len(), "subpages": subs.iter().take(60).collect::<Vec<_>>()})
            }
        });
    }
    let (prefix, searched) = search_candidates(&wiki, e);
    let search: Vec<Value> = searched.into_iter().map(|(term, r)| Ok(json!({"term": term, "results": describe(&wiki, &dedupe(r, 15), 120)?}))).collect::<Result<_>>()?;
    Ok(json!({"host": host, "exact": exact, "prefix_search": describe(&wiki, &prefix, 120)?, "search": search}))
}

pub fn discover_entry(http: &dyn Http, cache: Option<&Cache>, e: &Entry) -> Value {
    let mut hosts = Vec::new();
    if e.source == "ohchr" {
        hosts.push(match e.url.as_ref().map(|u| http.get(u)) {
            Some(Ok(h)) => json!({"host": "ohchr", "url": e.url, "bytes": h.len(), "preview": texts_html::html_to_text(&h).chars().take(200).collect::<String>()}),
            Some(Err(err)) => json!({"host": "ohchr", "error": format!("{err:#}")}),
            None => json!({"host": "ohchr", "error": "no url"}),
        });
    }
    match hosts_for(e) {
        None => hosts.push(json!({"error": format!("source {:?} is not fetched", e.source)})),
        Some(hs) => {
            for h in hs {
                hosts.push(discover_host(http, cache, h, e).unwrap_or_else(|err| json!({"host": h, "error": format!("{err:#}")})));
            }
        }
    }
    json!({"id": e.id, "source": e.source, "source_title": e.source_title, "alt_titles": e.alt_titles, "prefer_edition": e.prefer_edition, "sections": e.sections, "hosts": hosts})
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
    if let Some(n) = report["discovered"].as_u64() {
        return format!("fetch-texts --discover: {n} entries written to _discover.json");
    }
    let n = |k: &str| report[k].as_array().map(|a| a.len()).unwrap_or(0);
    let ids = |k: &str| report[k].as_array().map(|a| a.iter().filter_map(|r| r["id"].as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
    let mut s = format!("fetch-texts: resolved {} | missing {} | ambiguous {} | unsupported {} | errors {} | needs_sections {}\n", n("resolved"), n("missing"), n("ambiguous"), n("unsupported"), n("errors"), n("needs_sections"));
    for k in ["missing", "ambiguous", "unsupported", "errors", "needs_sections"] {
        if n(k) > 0 {
            s.push_str(&format!("  {k}: {}\n", ids(k)));
        }
    }
    let en = |k: &str| report["english"][k].as_array().map(|a| a.len()).unwrap_or(0);
    s.push_str(&format!("english: resolved {} | missing {} | ambiguous {} | errors {}", en("resolved"), en("missing"), en("ambiguous"), en("errors")));
    s
}

fn panic_message(p: Box<dyn std::any::Any + Send>) -> String {
    p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "unknown panic".into())
}

/// Run `f`, turning a panic into an error message so one bad page cannot stop the run.
fn isolated<T>(f: impl FnOnce() -> T) -> std::result::Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|p| format!("panic: {}", panic_message(p)))
}

/// One batched request per host for every `source_title`/`alt_titles` (the first resolution pass).
fn prepass(http: &dyn Http, cache: &Cache, entries: &[&Entry]) {
    let mut by_host: HashMap<&str, Vec<String>> = HashMap::new();
    for e in entries {
        if let Some(h) = hosts_for(e).and_then(|h| h.first().copied()) {
            if e.source != "ohchr" || e.url.is_none() {
                by_host.entry(h).or_default().extend(accepted_titles(e));
            }
        }
    }
    for (host, mut titles) in by_host {
        titles.sort();
        titles.dedup();
        let wiki = Wiki { http, host: host.to_string(), cache: Some(cache) };
        log::info!("pre-pass: {} titles on {host}", titles.len());
        if let Err(e) = wiki.get_pages(&titles) {
            log::warn!("pre-pass on {host} failed (entries will be looked up one by one): {e:#}");
        }
    }
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
    let cache: Cache = RefCell::new(HashMap::new());

    if opts.discover {
        let path = opts.out.join("_discover.json");
        let prev: Value = fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
        prepass(http, &cache, &selected);
        let mut fresh = Vec::new();
        for (n, e) in selected.iter().enumerate() {
            log::info!("[{}/{}] discover {} ({})", n + 1, selected.len(), e.id, e.source_title);
            fresh.push(isolated(|| discover_entry(http, Some(&cache), e)).unwrap_or_else(|m| json!({"id": e.id, "error": m})));
            // write after every entry so a crash or cancellation keeps the partial result
            let ids: Vec<String> = fresh.iter().filter_map(|r| r["id"].as_str().map(String::from)).collect();
            write_json(&path, &json!({"generated_at": now_iso(), "entries": merge_rows(&prev, "entries", &ids, fresh.clone())}))?;
        }
        return Ok(RunSummary { report: json!({"discovered": fresh.len()}) });
    }

    let report_path = opts.out.join("_report.json");
    let prev: Value = fs::read_to_string(&report_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let todo: Vec<&Entry> = selected.iter().copied().filter(|e| opts.force || !opts.out.join(format!("{}.json", e.id)).exists()).collect();
    prepass(http, &cache, &todo);

    let (mut resolved, mut missing, mut ambiguous, mut unsupported, mut errors, mut needs) = (vec![], vec![], vec![], vec![], vec![], vec![]);
    let mut processed: Vec<String> = Vec::new();
    for (n, e) in selected.iter().enumerate() {
        let path = opts.out.join(format!("{}.json", e.id));
        if path.exists() && !opts.force {
            log::info!("[{}/{}] {}: already fetched, skipping", n + 1, selected.len(), e.id);
            continue;
        }
        log::info!("[{}/{}] {} ({})", n + 1, selected.len(), e.id, e.source_title);
        processed.push(e.id.clone());
        let outcome = isolated(|| fetch_entry_cached(http, Some(&cache), e, opts.max_subpages)).unwrap_or_else(Outcome::Failed);
        match outcome {
            Outcome::Resolved(d) => match write_json(&path, &doc_json(&e.id, &d)) {
                Ok(()) => {
                    log::info!("  ok: {} ({} chars, {} subpages, via {})", d.page_title, d.text.chars().count(), d.subpages.len(), d.resolved_via);
                    if d.needs_sections {
                        log::warn!("  {}: excerpt without `sections` - cut to ~{EXCERPT_CHARS} chars", e.id);
                        needs.push(json!({"id": e.id, "page_title": d.page_title, "chars_kept": d.text.chars().count(), "subpages": d.subpages.iter().filter_map(|s| s["title"].as_str()).collect::<Vec<_>>()}));
                    }
                    resolved.push(json!({"id": e.id, "source": d.source, "page_title": d.page_title, "url": d.url, "revision_id": d.revision_id, "chars": d.text.chars().count(), "subpages": d.subpages.len(), "resolved_via": d.resolved_via, "truncated": d.truncated, "needs_sections": d.needs_sections}));
                }
                Err(err) => errors.push(json!({"id": e.id, "error": format!("writing output: {err:#}")})),
            },
            Outcome::Missing { tried } => {
                log::warn!("  {}: not found (tried {})", e.id, tried.join(", "));
                missing.push(json!({"id": e.id, "source_title": e.source_title, "tried": tried}));
            }
            Outcome::Ambiguous { candidates } => {
                let names: Vec<String> = candidates.iter().filter_map(|c| c["title"].as_str().map(String::from)).collect();
                log::warn!("  {}: nothing accepted; candidates: {}", e.id, names.join(" | "));
                ambiguous.push(json!({"id": e.id, "source_title": e.source_title, "candidates": candidates}));
            }
            Outcome::Unsupported(m) => unsupported.push(json!({"id": e.id, "reason": m})),
            Outcome::Failed(m) => {
                log::warn!("  {}: failed: {m}", e.id);
                errors.push(json!({"id": e.id, "error": m}));
            }
        }
        // keep the report current after every entry (partial results survive a crash)
        let _ = write_report(&report_path, &prev, &processed, &resolved, &missing, &ambiguous, &unsupported, &errors, &needs, &prev["english"]);
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
        match isolated(|| fetch_english(http, p)).unwrap_or_else(EnOutcome::Failed) {
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
    let english = json!({
        "resolved": merge_rows(&prev["english"], "resolved", &done_refs, en_ok),
        "missing": merge_rows(&prev["english"], "missing", &done_refs, en_missing),
        "ambiguous": merge_rows(&prev["english"], "ambiguous", &done_refs, en_amb),
        "errors": merge_rows(&prev["english"], "errors", &done_refs, en_err),
    });
    let report = write_report(&report_path, &prev, &processed, &resolved, &missing, &ambiguous, &unsupported, &errors, &needs, &english)?;
    Ok(RunSummary { report })
}

#[allow(clippy::too_many_arguments)]
fn write_report(path: &Path, prev: &Value, processed: &[String], resolved: &[Value], missing: &[Value], ambiguous: &[Value], unsupported: &[Value], errors: &[Value], needs: &[Value], english: &Value) -> Result<Value> {
    let report = json!({
        "generated_at": now_iso(),
        "resolved": merge_rows(prev, "resolved", processed, resolved.to_vec()),
        "missing": merge_rows(prev, "missing", processed, missing.to_vec()),
        "ambiguous": merge_rows(prev, "ambiguous", processed, ambiguous.to_vec()),
        "unsupported": merge_rows(prev, "unsupported", processed, unsupported.to_vec()),
        "errors": merge_rows(prev, "errors", processed, errors.to_vec()),
        "needs_sections": merge_rows(prev, "needs_sections", processed, needs.to_vec()),
        "english": english,
    });
    write_json(path, &report)?;
    Ok(report)
}
