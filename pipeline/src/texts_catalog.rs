//! The curated Reader catalogue (`pipeline/texts/catalog.toml`).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

pub const PERIODS: &[&str] = &[
    "ancient", "three-kingdoms", "unified-silla", "goryeo", "joseon-early", "joseon-late", "korean-empire", "colonial", "modern",
];
pub const THEMES: &[&str] = &["ancient-goryeo", "joseon", "colonial-independence", "modern-korea", "sino-korean"];
pub const SHELVES: &[&str] = &[
    "classical-prose",
    "verse",
    "hanmun",
    "sino-korean",
    "documents",
    "treaties",
    "political-writing",
    "constitution",
    "modern-poetry",
    "modern-fiction",
    "essays-children",
];
pub const SCRIPTS: &[&str] = &["hangul", "hanmun", "mixed"];
pub const SOURCES: &[&str] = &["wikisource-ko", "wikisource-zh", "law", "ohchr", "korea-kr", "history-db", "gutenberg", "archive-org"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnglishPd {
    pub title: String,
    pub translator: String,
    pub year: i32,
    /// "gutenberg" or "archive-org"
    pub source: String,
    /// Gutenberg ebook number, when known.
    pub ebook: Option<u32>,
    /// archive.org identifier, when known.
    pub identifier: Option<String>,
    /// Search terms used to resolve the item when no id is given.
    pub search: Option<String>,
    /// File stem under `raw/en/`.
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub title_ko: String,
    pub title_hanja: Option<String>,
    pub title_en: String,
    pub author_ko: String,
    pub author_en: String,
    #[serde(default)]
    pub author_dates: String,
    pub date: String,
    pub year: i32,
    pub period: String,
    pub themes: Vec<String>,
    pub shelf: String,
    pub script: String,
    pub collection: Option<String>,
    #[serde(default)]
    pub excerpt: bool,
    pub excerpt_note: Option<String>,
    pub source: String,
    pub source_title: String,
    /// Direct URL for sources that are not MediaWiki (e.g. the OHCHR page).
    pub url: Option<String>,
    /// `history-db`: 한국사데이터베이스 leaf ids (original-text items only, in reading order).
    #[serde(default)]
    pub level_ids: Vec<String>,
    #[serde(default)]
    pub search: Vec<String>,
    /// Other exact page titles that are accepted for this text (search results never are).
    #[serde(default)]
    pub alt_titles: Vec<String>,
    /// When the page is a versions/editions index: follow the link whose title contains this ("경판", "완판", …).
    pub prefer_edition: Option<String>,
    /// Subpage names (or heading names) to keep for long works.
    #[serde(default)]
    pub sections: Vec<String>,
    pub pd_basis: String,
    pub note: Option<String>,
    pub english_pd: Option<EnglishPd>,
    #[serde(default)]
    pub uncertain: Vec<String>,
    /// The build fails when this text has raw source but no approved enrichment.
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewsCfg {
    #[serde(default)]
    pub feeds: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    #[serde(default)]
    pub news: NewsCfg,
    #[serde(rename = "text")]
    pub texts: Vec<Entry>,
}

pub fn parse(src: &str) -> Result<Catalog> {
    toml::from_str(src).context("parsing catalogue TOML")
}

pub fn load(path: &Path) -> Result<Catalog> {
    let src = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse(&src).with_context(|| path.display().to_string())
}

/// Structural validation; returns one message per problem (empty = valid).
pub fn validate(cat: &Catalog) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for e in &cat.texts {
        let id = &e.id;
        let mut bad = |m: String| problems.push(format!("{id}: {m}"));
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-') {
            bad("id must be a lowercase ascii slug".into());
        }
        if !seen.insert(id.clone()) {
            bad("duplicate id".into());
        }
        for (name, v) in [("title_ko", &e.title_ko), ("title_en", &e.title_en), ("author_ko", &e.author_ko), ("author_en", &e.author_en), ("date", &e.date), ("source_title", &e.source_title), ("pd_basis", &e.pd_basis)] {
            if v.trim().is_empty() {
                bad(format!("{name} is empty"));
            }
        }
        if !(-2400..=2025).contains(&e.year) {
            bad(format!("implausible year {}", e.year));
        }
        if !PERIODS.contains(&e.period.as_str()) {
            bad(format!("unknown period {:?}", e.period));
        }
        if e.themes.is_empty() {
            bad("no themes".into());
        }
        for t in &e.themes {
            if !THEMES.contains(&t.as_str()) {
                bad(format!("unknown theme {t:?}"));
            }
        }
        if !SHELVES.contains(&e.shelf.as_str()) {
            bad(format!("unknown shelf {:?}", e.shelf));
        }
        if !SCRIPTS.contains(&e.script.as_str()) {
            bad(format!("unknown script {:?}", e.script));
        }
        if e.source == "history-db" && e.level_ids.is_empty() {
            bad("history-db entry without level_ids".into());
        }
        if let Some(t) = e.level_ids.iter().find(|i| crate::texts_historydb::is_translation(i)) {
            bad(format!("level id {t} is a translation (국역) item"));
        }
        if !SOURCES.contains(&e.source.as_str()) {
            bad(format!("unknown source {:?}", e.source));
        }
        if e.excerpt && e.excerpt_note.as_deref().unwrap_or("").is_empty() {
            bad("excerpt = true needs an excerpt_note".into());
        }
        if let Some(p) = &e.english_pd {
            if !matches!(p.source.as_str(), "gutenberg" | "archive-org") {
                bad(format!("english_pd.source {:?}", p.source));
            }
            if p.ebook.is_none() && p.identifier.is_none() && p.search.is_none() {
                bad("english_pd needs ebook, identifier or search".into());
            }
        }
    }
    problems
}
