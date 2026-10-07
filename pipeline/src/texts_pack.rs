//! The optional `texts` pack (Reader library): catalogue + raw sources + approved enrichment.
//! See docs/READER.md, "Pack build rules" and "`texts` pack + engine API".

use crate::build::{finish_db, open_db, set_meta};
use crate::common::hw_norm;
use crate::texts_catalog::{self, Entry};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCHEMA: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE texts (
  id TEXT PRIMARY KEY, sort INTEGER NOT NULL,
  shelf TEXT NOT NULL, period TEXT, year INTEGER, script TEXT NOT NULL,
  level TEXT, chars INTEGER NOT NULL,
  meta TEXT NOT NULL,
  card TEXT NOT NULL, notes TEXT NOT NULL,
  provenance TEXT NOT NULL,
  labels TEXT NOT NULL, review TEXT NOT NULL,
  vocab TEXT, questions TEXT
);
CREATE TABLE paragraphs (text_id TEXT NOT NULL, n INTEGER NOT NULL, orig TEXT NOT NULL,
  modern TEXT, reading TEXT, en TEXT, PRIMARY KEY (text_id, n)) WITHOUT ROWID;
"#;

/// A healthy full build ships at least this many texts (41 graded readers + the library).
pub const MIN_TEXTS: i64 = 40;

/// Shelves whose line breaks are meaningful.
const VERSE_SHELVES: &[&str] = &["verse", "modern-poetry"];

pub struct TextsOpts<'a> {
    /// `pipeline/texts` (holds `catalog.toml`, `raw/`, `enriched/`).
    pub dir: &'a Path,
    /// The freshly built core pack, for vocab resolution and levels.
    pub core: &'a Path,
    /// Other Korean dictionary packs (stdict, opendict): a word found there resolves, with no level.
    pub extra: &'a [PathBuf],
    pub out: &'a Path,
    pub version: &'a str,
    pub built_at: &'a str,
    pub allow_partial: bool,
}

#[derive(Debug)]
pub struct TextsResult {
    pub path: PathBuf,
    pub counts: Value,
}

/// One text ready to be written.
struct Packed {
    id: String,
    shelf: String,
    period: String,
    year: i64,
    script: String,
    level: Option<String>,
    meta: Value,
    card: Value,
    notes: Value,
    provenance: Value,
    labels: Value,
    review: Value,
    vocab: Option<Value>,
    questions: Option<Value>,
    paragraphs: Vec<Para>,
}

#[derive(Clone)]
struct Para {
    orig: String,
    modern: Option<String>,
    reading: Option<String>,
    en: Option<String>,
}

// ---------------------------------------------------------------- helpers

/// Split raw text into paragraphs on blank lines. Verse keeps its line breaks; prose lines are
/// joined (with a space, or directly for hanmun).
pub fn split_paragraphs(text: &str, keep_lines: bool, hanmun: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let flush = |cur: &mut Vec<&str>, out: &mut Vec<String>| {
        if !cur.is_empty() {
            let sep = if keep_lines {
                "\n"
            } else if hanmun {
                ""
            } else {
                " "
            };
            out.push(cur.join(sep));
            cur.clear();
        }
    };
    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            flush(&mut cur, &mut out);
        } else {
            cur.push(l);
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// Whitespace-normalised form used to compare enriched paragraphs with the raw source.
pub fn norm_ws(s: &str, hanmun: bool) -> String {
    if hanmun {
        s.split_whitespace().collect()
    } else {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// Check the enriched `orig` paragraphs against the raw paragraphs. Exact sequence, or for
/// catalogue excerpts an ordered selection of raw paragraphs. Returns a problem description.
pub fn check_integrity(
    enriched: &[String],
    raw: &[String],
    excerpt: bool,
    hanmun: bool,
) -> Option<String> {
    let e: Vec<String> = enriched.iter().map(|p| norm_ws(p, hanmun)).collect();
    let r: Vec<String> = raw.iter().map(|p| norm_ws(p, hanmun)).collect();
    if !excerpt {
        if e.len() != r.len() {
            return Some(format!(
                "{} enriched paragraphs but {} raw paragraphs",
                e.len(),
                r.len()
            ));
        }
        for (i, (a, b)) in e.iter().zip(&r).enumerate() {
            if a != b {
                return Some(format!(
                    "paragraph {i} differs from the raw source: enriched {:?} vs raw {:?}",
                    clip(a),
                    clip(b)
                ));
            }
        }
        return None;
    }
    let mut at = 0usize;
    for (i, a) in e.iter().enumerate() {
        match r[at..].iter().position(|b| b == a) {
            Some(k) => at += k + 1,
            None => {
                return Some(format!(
                    "excerpt paragraph {i} ({:?}) is not an in-order paragraph of the raw source",
                    clip(a)
                ))
            }
        }
    }
    None
}

fn clip(s: &str) -> String {
    s.chars().take(24).collect()
}

fn s_of(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(Value::as_str).map(str::to_string)
}

fn read_json(p: &Path) -> Result<Value> {
    let s = fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    serde_json::from_str(&s).with_context(|| format!("parsing {}", p.display()))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect()
        })
        .unwrap_or_default();
    v.retain(|p| !p.file_name().unwrap().to_string_lossy().starts_with('_'));
    v.sort();
    v
}

/// Slug of the human-readable periods / themes used by the graded readers' `meta`.
fn period_slug(label: &str, year: i64) -> String {
    match label {
        "Ancient" => "ancient",
        "Goryeo" => "goryeo",
        "Joseon" => {
            if year < 1700 {
                "joseon-early"
            } else {
                "joseon-late"
            }
        }
        "Colonial era" => "colonial",
        "Modern" => "modern",
        other => return other.to_string(),
    }
    .to_string()
}

fn theme_slug(label: &str) -> String {
    match label {
        "Ancient & Goryeo" => "ancient-goryeo",
        "Joseon" => "joseon",
        "Colonial era & independence" => "colonial-independence",
        "Modern Korea" => "modern-korea",
        "Sino-Korean relations" => "sino-korean",
        other => return other.to_string(),
    }
    .to_string()
}

/// Level label for a text without enrichment, from catalogue metadata only.
fn derived_level(e: &Entry) -> &'static str {
    if e.script == "hanmun"
        || matches!(
            e.period.as_str(),
            "ancient"
                | "three-kingdoms"
                | "unified-silla"
                | "goryeo"
                | "joseon-early"
                | "joseon-late"
        )
    {
        "classical"
    } else {
        "advanced"
    }
}

// ---------------------------------------------------------------- vocab resolution

struct Resolver {
    conn: Connection,
    extra: Vec<Connection>,
}

impl Resolver {
    fn open(core: &Path, extra: &[PathBuf]) -> Result<Resolver> {
        let ro = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY;
        let conn = Connection::open_with_flags(core, ro).with_context(|| format!("opening {}", core.display()))?;
        let extra = extra
            .iter()
            .filter(|p| p.exists())
            .map(|p| Connection::open_with_flags(p, ro).with_context(|| format!("opening {}", p.display())))
            .collect::<Result<Vec<_>>>()?;
        Ok(Resolver { conn, extra })
    }
    /// `None` = unresolved; `Some(level)` = resolved (level may be absent).
    fn resolve(&self, word: &str) -> Result<Option<Option<i64>>> {
        let w = hw_norm(word);
        let (n, lvl): (i64, Option<i64>) = self.conn.query_row(
            "SELECT count(*), min(level) FROM entries WHERE hw_norm = ?1",
            params![w],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if n > 0 {
            return Ok(Some(lvl));
        }
        let (n, lvl): (i64, Option<i64>) = self.conn.query_row("SELECT count(*), min(e.level) FROM forms f JOIN entries e ON e.id = f.entry_id WHERE f.form = ?1", params![w], |r| Ok((r.get(0)?, r.get(1)?)))?;
        if n > 0 {
            return Ok(Some(lvl));
        }
        for c in &self.extra {
            let n: i64 = c.query_row("SELECT (SELECT count(*) FROM entries WHERE hw_norm = ?1) + (SELECT count(*) FROM forms WHERE form = ?1)", params![w], |r| r.get(0))?;
            if n > 0 {
                return Ok(Some(None));
            }
        }
        Ok(None)
    }
}

/// Recompute `vocab[].level` in place; returns the unresolved words.
fn fix_vocab(vocab: &mut Value, res: &Resolver) -> Result<Vec<String>> {
    let mut bad = Vec::new();
    if let Some(arr) = vocab.as_array_mut() {
        for item in arr {
            let Some(w) = s_of(item, "word") else {
                bad.push("(vocab item without a word)".to_string());
                continue;
            };
            match res.resolve(&w)? {
                Some(l) => {
                    item["level"] = l.map_or(Value::Null, |l| json!(l));
                }
                None => {
                    item["level"] = Value::Null;
                    bad.push(w);
                }
            }
        }
    }
    Ok(bad)
}

// ---------------------------------------------------------------- building the texts

fn validate_enriched(v: &Value, graded: bool) -> Vec<String> {
    let mut p = Vec::new();
    for k in ["card", "notes", "labels", "review"] {
        if !v.get(k).is_some_and(Value::is_object) {
            p.push(format!("missing object `{k}`"));
        }
    }
    match v.get("paragraphs").and_then(Value::as_array) {
        Some(a) if !a.is_empty() => {
            for (i, para) in a.iter().enumerate() {
                if s_of(para, "orig").is_none_or(|s| s.trim().is_empty()) {
                    p.push(format!("paragraph {i} has no `orig`"));
                }
            }
        }
        _ => p.push("`paragraphs` missing or empty".into()),
    }
    if graded && !v.get("meta").is_some_and(Value::is_object) {
        p.push("graded reader without `meta`".into());
    }
    let text_label = v.pointer("/labels/text").and_then(Value::as_str);
    if graded && text_label != Some("ai") {
        p.push("graded reader must have labels.text = \"ai\"".into());
    }
    if !graded && text_label != Some("original") {
        p.push("labels.text must be \"original\" for catalogue texts".into());
    }
    p
}

fn paras_from_enriched(v: &Value) -> Vec<Para> {
    v["paragraphs"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|p| Para {
                    orig: s_of(p, "orig").unwrap_or_default(),
                    modern: s_of(p, "modern"),
                    reading: s_of(p, "reading"),
                    en: s_of(p, "en"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn provenance_from_raw(raw: &Value, enriched: Option<&Value>, e: &Entry) -> Value {
    let mut m = Map::new();
    for k in [
        "source",
        "url",
        "page_title",
        "revision_id",
        "revision_timestamp",
        "edition",
        "licence",
        "fetched_at",
    ] {
        m.insert(k.into(), raw.get(k).cloned().unwrap_or(Value::Null));
    }
    if let (Some(pd), Some(en)) = (&e.english_pd, enriched) {
        if en
            .pointer("/labels/translation")
            .and_then(Value::as_str)
            .is_some_and(|t| t.starts_with("pd:"))
        {
            m.insert("english_source".into(), json!({"title": pd.title, "translator": pd.translator, "year": pd.year, "source": pd.source, "ref": pd.reference}));
        }
    }
    Value::Object(m)
}

fn catalog_meta(e: &Entry) -> Value {
    let mut v = serde_json::to_value(e).unwrap_or(Value::Null);
    if let Some(m) = v.as_object_mut() {
        for k in ["search", "uncertain", "english_pd", "required"] {
            m.remove(k);
        }
    }
    v
}

fn edition_line(raw: &Value) -> String {
    let title = s_of(raw, "page_title").unwrap_or_default();
    let src = match s_of(raw, "source").as_deref() {
        Some("wikisource-zh") => "Chinese Wikisource",
        Some("wikisource-ko") => "Korean Wikisource",
        Some("law") => "Korean statute (Ministry of Government Legislation)",
        Some("ohchr") => "OHCHR",
        Some(o) => return format!("{o}: {title}"),
        None => "Source",
    };
    let rev = raw
        .get("revision_id")
        .and_then(Value::as_u64)
        .map(|r| format!(", revision {r}"))
        .unwrap_or_default();
    format!("{src}: {title}{rev}")
}

fn graded_text(id: &str, v: &Value) -> Packed {
    let mut meta = v["meta"].clone();
    let year = meta.get("year").and_then(Value::as_i64).unwrap_or(0);
    let period = s_of(&meta, "period")
        .map(|p| period_slug(&p, year))
        .unwrap_or_else(|| "modern".into());
    let themes: Vec<Value> = meta
        .get("themes")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|t| json!(theme_slug(t)))
                .collect()
        })
        .unwrap_or_default();
    meta["period"] = json!(period);
    meta["themes"] = Value::Array(themes);
    let card = v["card"].clone();
    let provenance = json!({"source": "original", "url": null, "page_title": null, "revision_id": null, "revision_timestamp": null,
        "edition": card.get("edition_en").cloned().unwrap_or(Value::Null), "licence": meta.get("pd_basis").cloned().unwrap_or(Value::Null), "fetched_at": null});
    Packed {
        id: id.to_string(),
        shelf: "graded".into(),
        period,
        year,
        script: s_of(&meta, "script").unwrap_or_else(|| "hangul".into()),
        level: s_of(&card, "level"),
        meta,
        card,
        notes: v["notes"].clone(),
        provenance,
        labels: v["labels"].clone(),
        review: v["review"].clone(),
        vocab: v.get("vocab").filter(|x| !x.is_null()).cloned(),
        questions: v.get("questions").filter(|x| !x.is_null()).cloned(),
        paragraphs: paras_from_enriched(v),
    }
}

/// Ids whose raw text a reviewer approved for original-only shipping (`[approved] ids = [...]`).
fn raw_reviewed(path: &Path) -> Result<HashSet<String>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let v: toml::Value = toml::from_str(&fs::read_to_string(path)?).with_context(|| format!("{}", path.display()))?;
    Ok(v.get("approved")
        .and_then(|a| a.get("ids"))
        .and_then(|ids| ids.as_array())
        .map(|ids| ids.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default())
}

/// Why a raw (unenriched) text must not be packed: wiki markup or licence boilerplate left in
/// the text, or nothing but section headings (e.g. a page that only transcludes scans).
pub fn raw_problem(paras: &[String]) -> Option<String> {
    const JUNK: &[&str] = &["{{", "}}", "[[", "]]", "<pages", "<ref", "__TOC__", "PD-old"];
    const BOILER: &[&str] = &["## 라이선스", "## 저작권", "## License", "## Copyright"];
    for p in paras {
        if let Some(j) = JUNK.iter().find(|j| p.contains(*j)) {
            return Some(format!("wiki markup {j:?} in: {}", clip(p)));
        }
        if BOILER.iter().any(|b| p.trim() == *b) {
            return Some(format!("licence section kept: {}", p.trim()));
        }
    }
    if !paras.iter().any(|p| !p.trim_start().starts_with("## ") && !p.trim().is_empty()) {
        return Some("only section headings, no text".into());
    }
    None
}

fn catalog_text(
    e: &Entry,
    raw: Option<&Value>,
    enriched: Option<&Value>,
    raw_paras: &[String],
) -> Packed {
    let provenance = match raw {
        Some(r) => provenance_from_raw(r, enriched, e),
        None => {
            json!({"source": e.source, "url": e.url, "page_title": e.source_title, "revision_id": null, "revision_timestamp": null, "edition": null, "licence": null, "fetched_at": null})
        }
    };
    let base = |card: Value,
                notes: Value,
                labels: Value,
                review: Value,
                level: Option<String>,
                paragraphs: Vec<Para>,
                vocab,
                questions| Packed {
        id: e.id.clone(),
        shelf: e.shelf.clone(),
        period: e.period.clone(),
        year: e.year as i64,
        script: e.script.clone(),
        level,
        meta: catalog_meta(e),
        card,
        notes,
        provenance: provenance.clone(),
        labels,
        review,
        vocab,
        questions,
        paragraphs,
    };
    match enriched {
        Some(v) => base(
            v["card"].clone(),
            v["notes"].clone(),
            v["labels"].clone(),
            v["review"].clone(),
            s_of(&v["card"], "level"),
            paras_from_enriched(v),
            v.get("vocab").filter(|x| !x.is_null()).cloned(),
            v.get("questions").filter(|x| !x.is_null()).cloned(),
        ),
        None => {
            let ed = raw
                .map(edition_line)
                .unwrap_or_else(|| e.source_title.clone());
            let notes_en = [Some(e.pd_basis.as_str()), e.note.as_deref()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
            base(
                json!({"summary_ko": "", "summary_en": "", "level": derived_level(e), "edition_ko": ed, "edition_en": ed}),
                json!({"ko": "", "en": notes_en}),
                json!({"text": "original"}),
                json!({"status": "original-only", "by": "pipeline", "notes": "Source text only; no approved enrichment yet"}),
                Some(derived_level(e).to_string()),
                raw_paras
                    .iter()
                    .map(|p| Para {
                        orig: p.clone(),
                        modern: None,
                        reading: None,
                        en: None,
                    })
                    .collect(),
                None,
                None,
            )
        }
    }
}

fn news_text(raw: &Value) -> Option<Packed> {
    let id = s_of(raw, "id")?;
    let title = s_of(raw, "title")?;
    let text = s_of(raw, "text")?;
    let mut paras = split_paragraphs(&text, false, false);
    if paras.len() == 1 && text.lines().filter(|l| !l.trim().is_empty()).count() > 1 {
        paras = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
    }
    if paras.is_empty() {
        return None;
    }
    let date = s_of(raw, "published").unwrap_or_default();
    let year = date
        .get(..4)
        .and_then(|y| y.parse::<i64>().ok())
        .unwrap_or(2026);
    let licence = s_of(raw, "licence").unwrap_or_else(|| "KOGL Type 1".into());
    let attribution = s_of(raw, "attribution")
        .unwrap_or_else(|| "출처: 정책브리핑 (korea.kr), 대한민국 정부".into());
    let meta = json!({
        "id": id, "title_ko": title, "title_en": "", "author_ko": "정책브리핑 (korea.kr)", "author_en": "Korea Policy Briefing",
        "author_dates": "", "date": date, "year": year, "period": "modern", "themes": ["modern-korea"], "shelf": "news", "script": "hangul",
        "excerpt": false, "source": "korea-kr", "source_title": title, "url": raw.get("url").cloned().unwrap_or(Value::Null),
        "pd_basis": "KOGL Type 1 (공공누리 제1유형): attribution required, commercial use and modification allowed", "note": attribution,
    });
    Some(Packed {
        id,
        shelf: "news".into(),
        period: "modern".into(),
        year,
        script: "hangul".into(),
        level: None,
        meta,
        card: json!({"summary_ko": "", "summary_en": "", "level": null, "edition_ko": attribution, "edition_en": "Korea Policy Briefing (korea.kr), KOGL Type 1"}),
        notes: json!({"ko": "", "en": ""}),
        provenance: json!({"source": "korea-kr", "url": raw.get("url").cloned().unwrap_or(Value::Null), "page_title": title, "revision_id": null,
            "revision_timestamp": date, "edition": null, "licence": licence, "fetched_at": raw.get("fetched_at").cloned().unwrap_or(Value::Null)}),
        labels: json!({"text": "original"}),
        review: json!({"status": "approved", "by": "source", "notes": "KOGL Type 1 original"}),
        vocab: None,
        questions: None,
        paragraphs: paras
            .into_iter()
            .map(|p| Para {
                orig: p,
                modern: None,
                reading: None,
                en: None,
            })
            .collect(),
    })
}

/// Collect every text that can be packed; `problems` collects build failures.
fn collect(
    o: &TextsOpts,
    res: &Resolver,
    problems: &mut Vec<String>,
) -> Result<(Vec<Packed>, Value)> {
    let cat = texts_catalog::load(&o.dir.join("catalog.toml"))?;
    let (raw_dir, enr_dir) = (o.dir.join("raw"), o.dir.join("enriched"));
    let mut out: Vec<Packed> = Vec::new();
    let (mut n_enriched, mut n_original, mut n_graded, mut n_news, mut n_unapproved) =
        (0, 0, 0, 0, 0);
    let mut rejected: Vec<Value> = Vec::new();
    let mut n_unreviewed = 0;
    let raw_ok = raw_reviewed(&o.dir.join("raw_review.toml"))?;
    let note = |partial_ok: bool, msg: String, problems: &mut Vec<String>| {
        if partial_ok && o.allow_partial {
            log::warn!("texts: {msg}");
        } else {
            problems.push(msg);
        }
    };

    for e in &cat.texts {
        let raw = match fs::read_to_string(raw_dir.join(format!("{}.json", e.id))) {
            Ok(s) => Some(
                serde_json::from_str::<Value>(&s).with_context(|| format!("raw/{}.json", e.id))?,
            ),
            Err(_) => None,
        };
        let mut enriched = match fs::read_to_string(enr_dir.join(format!("{}.json", e.id))) {
            Ok(s) => Some(
                serde_json::from_str::<Value>(&s)
                    .with_context(|| format!("enriched/{}.json", e.id))?,
            ),
            Err(_) => None,
        };
        if let Some(v) = &enriched {
            if s_of(v, "id").as_deref() != Some(e.id.as_str()) {
                problems.push(format!("enriched/{}.json: id is {:?}", e.id, s_of(v, "id")));
            }
            if v.pointer("/review/status").and_then(Value::as_str) != Some("approved") {
                log::info!("texts: {}: enrichment not approved, ignored", e.id);
                n_unapproved += 1;
                enriched = None;
            }
        }
        let hanmun = e.script == "hanmun";
        let raw_paras = raw
            .as_ref()
            .and_then(|r| r.get("text").and_then(Value::as_str))
            .map(|t| split_paragraphs(t, VERSE_SHELVES.contains(&e.shelf.as_str()), hanmun))
            .unwrap_or_default();
        if let Some(v) = &enriched {
            let bad = validate_enriched(v, false);
            if !bad.is_empty() {
                note(
                    false,
                    format!("{}: enriched file invalid: {}", e.id, bad.join("; ")),
                    problems,
                );
                continue;
            }
            if raw.is_some() {
                let enr_orig: Vec<String> =
                    paras_from_enriched(v).into_iter().map(|p| p.orig).collect();
                if let Some(m) = check_integrity(&enr_orig, &raw_paras, e.excerpt, hanmun) {
                    note(
                        true,
                        format!("{}: enriched text does not match the raw source: {m}", e.id),
                        problems,
                    );
                    // partial build: fall back to the original text
                    enriched = None;
                }
            }
        }
        if raw.is_none() && enriched.is_none() {
            continue;
        }
        if raw.is_some() && enriched.is_none() && e.required {
            note(
                true,
                format!(
                    "{}: required text has raw source but no approved enrichment",
                    e.id
                ),
                problems,
            );
        }
        if raw.is_some() && raw_paras.is_empty() && enriched.is_none() {
            problems.push(format!("{}: raw source has no text", e.id));
            continue;
        }
        if enriched.is_none() && !raw_ok.contains(&e.id) {
            // original-only texts ship only after a reviewer checked the raw text (raw_review.toml)
            n_unreviewed += 1;
            continue;
        }
        if enriched.is_none() {
            if let Some(why) = raw_problem(&raw_paras) {
                // never ship markup leftovers or a heading-only page as a Reader text
                log::warn!("texts: {}: raw text left out of the pack: {why}", e.id);
                rejected.push(json!({"id": e.id, "why": why}));
                continue;
            }
        }
        let mut p = catalog_text(e, raw.as_ref(), enriched.as_ref(), &raw_paras);
        if enriched.is_some() {
            n_enriched += 1;
        } else {
            n_original += 1;
        }
        if let Some(v) = p.vocab.as_mut() {
            let bad = fix_vocab(v, res)?;
            if !bad.is_empty() {
                note(
                    true,
                    format!(
                        "{}: vocab words not found in the dictionary: {}",
                        e.id,
                        bad.join(", ")
                    ),
                    problems,
                );
            }
        }
        out.push(p);
    }

    // graded readers (own meta, no raw file)
    for path in json_files(&enr_dir) {
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        if !stem.starts_with("graded-") {
            if !cat.texts.iter().any(|e| e.id == stem) {
                problems.push(format!(
                    "enriched/{stem}.json: not in the catalogue and not a graded reader"
                ));
            }
            continue;
        }
        let v = read_json(&path)?;
        if s_of(&v, "id").as_deref() != Some(stem.as_str()) {
            problems.push(format!("enriched/{stem}.json: id is {:?}", s_of(&v, "id")));
            continue;
        }
        if v.pointer("/review/status").and_then(Value::as_str) != Some("approved") {
            n_unapproved += 1;
            log::info!("texts: {stem}: not approved, skipped");
            continue;
        }
        let bad = validate_enriched(&v, true);
        if !bad.is_empty() {
            problems.push(format!("{stem}: {}", bad.join("; ")));
            continue;
        }
        let mut p = graded_text(&stem, &v);
        if let Some(vv) = p.vocab.as_mut() {
            let bad = fix_vocab(vv, res)?;
            if !bad.is_empty() {
                note(
                    true,
                    format!(
                        "{stem}: vocab words not found in the dictionary: {}",
                        bad.join(", ")
                    ),
                    problems,
                );
            }
        }
        n_graded += 1;
        out.push(p);
    }

    // news
    for path in json_files(&raw_dir.join("news")) {
        let v = read_json(&path)?;
        match news_text(&v) {
            Some(p) => {
                n_news += 1;
                out.push(p);
            }
            None => log::warn!(
                "texts: {} is not a usable news article, skipped",
                path.display()
            ),
        }
    }

    let mut ids = std::collections::HashSet::new();
    for p in &out {
        if !ids.insert(p.id.clone()) {
            problems.push(format!("{}: duplicate text id", p.id));
        }
    }
    let stats = json!({"enriched": n_enriched, "original_only": n_original, "graded": n_graded, "news": n_news, "unapproved_ignored": n_unapproved, "rejected_raw": rejected, "raw_unreviewed": n_unreviewed});
    Ok((out, stats))
}

/// Build `texts.sqlite`. `Ok(None)` when no text is available at all (no catalogue, or nothing
/// fetched / approved yet).
pub fn build_texts(o: &TextsOpts) -> Result<Option<TextsResult>> {
    if !o.dir.join("catalog.toml").exists() {
        return Ok(None);
    }
    let res = Resolver::open(o.core, o.extra)?;
    let mut problems = Vec::new();
    let (mut texts, stats) = collect(o, &res, &mut problems)?;
    if !problems.is_empty() {
        anyhow::bail!(
            "texts pack: {} problem(s):\n  - {}",
            problems.len(),
            problems.join("\n  - ")
        );
    }
    if texts.is_empty() {
        return Ok(None);
    }
    texts.sort_by(|a, b| (a.year, &a.id).cmp(&(b.year, &b.id)));

    let path = o.out.join("texts.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(SCHEMA)?;
    conn.execute_batch("BEGIN")?;
    let mut shelves: BTreeMap<String, i64> = BTreeMap::new();
    let (mut n_par, mut n_chars) = (0i64, 0i64);
    for (i, t) in texts.iter().enumerate() {
        let chars: i64 = t
            .paragraphs
            .iter()
            .map(|p| p.orig.chars().count() as i64)
            .sum();
        conn.execute(
            "INSERT INTO texts(id,sort,shelf,period,year,script,level,chars,meta,card,notes,provenance,labels,review,vocab,questions) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                t.id,
                i as i64,
                t.shelf,
                t.period,
                t.year,
                t.script,
                t.level,
                chars,
                t.meta.to_string(),
                t.card.to_string(),
                t.notes.to_string(),
                t.provenance.to_string(),
                t.labels.to_string(),
                t.review.to_string(),
                t.vocab.as_ref().map(Value::to_string),
                t.questions.as_ref().map(Value::to_string),
            ],
        )?;
        for (n, p) in t.paragraphs.iter().enumerate() {
            conn.execute(
                "INSERT INTO paragraphs(text_id,n,orig,modern,reading,en) VALUES(?,?,?,?,?,?)",
                params![t.id, n as i64, p.orig, p.modern, p.reading, p.en],
            )?;
        }
        *shelves.entry(t.shelf.clone()).or_default() += 1;
        n_par += t.paragraphs.len() as i64;
        n_chars += chars;
    }
    let mut counts =
        json!({"texts": texts.len(), "paragraphs": n_par, "chars": n_chars, "shelves": shelves});
    for (k, v) in stats.as_object().unwrap() {
        counts[k] = v.clone();
    }
    set_meta(&conn, "pack", "texts")?;
    set_meta(&conn, "schema", "1")?;
    set_meta(&conn, "version", o.version)?;
    set_meta(&conn, "built_at", o.built_at)?;
    set_meta(&conn, "counts", &counts.to_string())?;
    conn.execute_batch("COMMIT")?;
    finish_db(conn)?;
    Ok(Some(TextsResult { path, counts }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraphs_split_on_blank_lines() {
        let t = "첫째 줄\n둘째 줄\n\n\n셋째\n";
        assert_eq!(
            split_paragraphs(t, false, false),
            vec!["첫째 줄 둘째 줄", "셋째"]
        );
        assert_eq!(
            split_paragraphs(t, true, false),
            vec!["첫째 줄\n둘째 줄", "셋째"]
        );
        assert_eq!(
            split_paragraphs("天地\n玄黃", false, true),
            vec!["天地玄黃"]
        );
    }

    #[test]
    fn integrity_exact_and_excerpt() {
        let raw: Vec<String> = ["가  나", "다", "라", "마"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let same: Vec<String> = ["가 나", "다", "라", "마"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(check_integrity(&same, &raw, false, false).is_none());
        assert!(check_integrity(&same[..2], &raw, false, false).is_some());
        assert!(check_integrity(&[same[0].clone(), same[3].clone()], &raw, true, false).is_none());
        assert!(check_integrity(&[same[3].clone(), same[0].clone()], &raw, true, false).is_some());
        let changed: Vec<String> = ["가 나", "다!", "라", "마"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(check_integrity(&changed, &raw, false, false)
            .unwrap()
            .contains("paragraph 1"));
    }

    #[test]
    fn graded_labels_are_slugged() {
        assert_eq!(period_slug("Joseon", 1592), "joseon-early");
        assert_eq!(period_slug("Joseon", 1790), "joseon-late");
        assert_eq!(
            theme_slug("Colonial era & independence"),
            "colonial-independence"
        );
    }
}

#[cfg(test)]
mod raw_problem_tests {
    use super::raw_problem;

    #[test]
    fn rejects_leftovers_and_heading_only_pages() {
        let v = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(raw_problem(&v(&["가시리 가시리잇고", "## 라이선스"])).unwrap().contains("licence"));
        assert!(raw_problem(&v(&["## 청구영언", "## 가곡원류"])).unwrap().contains("headings"));
        assert!(raw_problem(&v(&["본문 {{틀}}"])).unwrap().contains("markup"));
        assert_eq!(raw_problem(&v(&["## 1장", "본문입니다."])), None);
    }
}
