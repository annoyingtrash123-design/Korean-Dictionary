//! Build core.sqlite / stdict.sqlite and the site-data bundle.

use crate::common::*;
use crate::{cedict, freq, kaikki, kengdic, krdict, opendict, pack, schema, stdict, tatoeba, unihan, zhwikt};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const ATTRIBUTION: &str = include_str!("../ATTRIBUTION.md");

/// Everything the build can read. Each optional source may be missing.
#[derive(Default, Clone)]
pub struct Sources {
    pub krdict: Vec<PathBuf>,
    pub stdict: Vec<PathBuf>,
    pub opendict: Vec<PathBuf>,
    pub kaikki: Option<PathBuf>,
    pub kengdic: Option<PathBuf>,
    pub freq: Option<PathBuf>,
    pub tatoeba: Option<(PathBuf, PathBuf, PathBuf)>,
    pub unihan: Option<PathBuf>,
    /// `cedict_1_0_ts_utf-8_mdbg.zip` (or a plain `.u8`): optional `cedict` pack.
    pub cedict: Option<PathBuf>,
    /// kaikki Chinese JSONL (multi-GB, streamed): optional `zhwikt` pack.
    pub zhwikt: Option<PathBuf>,
}

fn xml_files(dir: &Path, limit: Option<usize>) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "xml")).collect())
        .unwrap_or_default();
    v.sort();
    if let Some(n) = limit {
        v.truncate(n);
    }
    v
}

impl Sources {
    pub fn discover(work: &Path, limit_files: Option<usize>) -> Sources {
        let nikl = work.join("nikl");
        let opt = |p: PathBuf, what: &str| -> Option<PathBuf> {
            if p.exists() {
                Some(p)
            } else {
                log::warn!("{what} not found at {} - building without it", p.display());
                None
            }
        };
        let krdict = xml_files(&nikl.join("krdict"), limit_files);
        let stdict = xml_files(&nikl.join("stdict"), limit_files);
        if krdict.is_empty() {
            log::warn!("no krdict XML files in {}", nikl.join("krdict").display());
        }
        if stdict.is_empty() {
            log::warn!("no stdict XML files in {}", nikl.join("stdict").display());
        }
        let opendict = xml_files(&nikl.join("opendict"), limit_files);
        if opendict.is_empty() {
            log::warn!("no opendict XML files in {} - skipping the opendict pack", nikl.join("opendict").display());
        }
        let tat = work.join("tatoeba");
        let tatoeba = match (
            opt(tat.join("kor_sentences.tsv.bz2"), "Tatoeba kor sentences"),
            opt(tat.join("eng_sentences.tsv.bz2"), "Tatoeba eng sentences"),
            opt(tat.join("links.tar.bz2"), "Tatoeba links"),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
            _ => None,
        };
        Sources {
            krdict,
            stdict,
            opendict,
            kaikki: opt(work.join("kaikki-ko.jsonl"), "kaikki Wiktionary JSONL"),
            kengdic: opt(work.join("kengdic.tsv"), "kengdic"),
            freq: opt(work.join("ko_50k.txt"), "FrequencyWords"),
            tatoeba,
            unihan: opt(work.join("Unihan.zip"), "Unihan"),
            cedict: opt(work.join("cedict.zip"), "CC-CEDICT (skipping the cedict pack)"),
            zhwikt: opt(work.join("kaikki-zh.jsonl"), "kaikki Chinese JSONL (skipping the zhwikt pack)"),
        }
    }
}

// --- time ---------------------------------------------------------------

/// (version "YYYYMMDD-HHMM", built_at ISO-8601 UTC) for a unix timestamp.
pub fn stamp(secs: i64) -> (String, String) {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (h, m, s) = (rem / 3600, rem % 3600 / 60, rem % 60);
    // civil_from_days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (format!("{y:04}{mo:02}{d:02}-{h:02}{m:02}"), format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z"))
}

// --- ranking ----------------------------------------------------------------

pub struct Ranker {
    pub ranks: HashMap<String, u32>,
    pub stems: HashMap<String, u32>,
}

const TIER_DEFAULT: [i64; 8] = [15_000, 25_000, 40_000, 55_000, 60_000, 70_000, 80_000, 90_000];

impl Ranker {
    pub fn new(ranks: HashMap<String, u32>) -> Ranker {
        let stems = freq::stem_ranks(&ranks);
        Ranker { ranks, stems }
    }

    pub fn tier(e: &Entry) -> usize {
        match e.source {
            "krdict" => match e.level {
                Some(l @ 1..=3) => (l - 1) as usize,
                _ => 3,
            },
            "wikt" => 4,
            "kengdic" => 5,
            "stdict" => 6,
            _ => 7,
        }
    }

    /// rank = freq_rank*8 + tier; lower = more important.
    pub fn rank(&self, e: &Entry) -> i64 {
        let tier = Self::tier(e);
        let hwn = hw_norm(&e.headword);
        let mut best: Option<u32> = self.ranks.get(&hwn).copied();
        if hwn.ends_with('다') && (is_verbal(&e.pos) || e.pos == "other" || e.kind != "word" || e.pos == "noun") {
            let stem = &hwn[..hwn.len() - '다'.len_utf8()];
            if let Some(&r) = self.stems.get(stem) {
                best = Some(best.map_or(r, |b| b.min(r)));
            }
        }
        let f = match best {
            Some(r) => r as i64,
            None => {
                let mut d = TIER_DEFAULT[tier];
                if matches!(e.kind.as_str(), "phrase" | "idiom" | "proverb") {
                    d += 20_000;
                }
                d
            }
        };
        f * 8 + tier as i64
    }
}

// --- entry helpers -----------------------------------------------------------

fn senses(e: &Entry) -> &[Value] {
    e.data.get("senses").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn sense_strs<'a>(e: &'a Entry, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    senses(e).iter().filter_map(move |s| s.get(key).and_then(Value::as_str)).filter(|s| !s.is_empty())
}

pub fn entry_gloss(e: &Entry) -> Option<String> {
    let mut seen = HashSet::new();
    let mut gl: Vec<&str> = Vec::new();
    for g in sense_strs(e, "gloss") {
        for piece in g.split("; ") {
            if seen.insert(piece.to_lowercase()) {
                gl.push(piece);
            }
        }
    }
    if !gl.is_empty() {
        return Some(make_gloss(&gl, 120));
    }
    for key in ["def", "ko_def"] {
        if let Some(d) = sense_strs(e, key).next() {
            return Some(truncate(d, 120));
        }
    }
    None
}

/// (head, en): `head` = the short English glosses (first 6 distinct items); `en` = the longer
/// English definitions, indexed only for entries that have no gloss. bm25 normalises by the
/// total document length over all columns, so definitions on glossed entries would push
/// good entries (학교 "school") below one-word kengdic matches.
fn fts_text(e: &Entry) -> (String, String) {
    // `head`: the short glosses (weighted 10x by the engine). `en`: any further glosses plus the
    // English definitions, so words that only appear in a definition stay searchable. Ranking
    // of gloss matches over definition matches is done by the engine, not by bm25 alone.
    let mut seen = HashSet::new();
    let pieces: Vec<&str> = sense_strs(e, "gloss")
        .flat_map(|g| g.split("; "))
        .map(str::trim)
        .filter(|p| !p.is_empty() && seen.insert(p.to_lowercase()))
        .collect();
    let head = pieces.iter().take(12).copied().collect::<Vec<_>>().join(" ; ");
    let mut en = pieces.iter().skip(12).copied().collect::<Vec<_>>().join(" ; ");
    for d in sense_strs(e, "def").take(8) {
        if !en.is_empty() {
            en.push_str(" ; ");
        }
        en.push_str(&truncate(d, 300));
    }
    (head, en)
}

/// Result-quality hint for the engine (0 = best): source tier + 1 for phrases/idioms/proverbs.
pub fn quality(e: &Entry) -> i64 {
    let base = match e.source {
        "krdict" => 0,
        "wikt" => 1,
        "stdict" => 2,
        "kengdic" => 3,
        _ => 4,
    };
    base + i64::from(matches!(e.kind.as_str(), "phrase" | "proverb" | "idiom"))
}

#[derive(Default)]
pub struct Counters {
    pub by_source: BTreeMap<&'static str, i64>,
    pub entries: i64,
    pub forms: i64,
    /// (term, tier, score, entry_id) rows for `gloss_terms` (core pack only).
    pub terms: Vec<(String, i64, i64, i64)>,
}

/// Normalised English gloss items of an entry in gloss order: lower-cased, trimmed, leading
/// "to " stripped, whitespace collapsed, items over 40 chars skipped, duplicates dropped.
/// The flag is true for the entry's first item (tier 0).
pub fn gloss_items(e: &Entry) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut first = true;
    for g in sense_strs(e, "gloss") {
        for piece in g.split([';', ',']) {
            let t = piece.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
            let t = t.strip_prefix("to ").map(str::trim).unwrap_or(&t).to_string();
            if t.is_empty() {
                continue;
            }
            let is_first = std::mem::take(&mut first);
            if t.chars().count() > 40 || out.iter().any(|(o, _)| *o == t) {
                continue;
            }
            out.push((t, is_first));
        }
    }
    out
}

/// True for 옛말 (old word) entries of the Korean dictionaries: every sense carries the sense
/// type/tag "옛말", or its Korean definition ends with "옛말" ("'아무'의 옛말.").
pub fn is_historical(e: &Entry) -> bool {
    if !matches!(e.source, "stdict" | "opendict") {
        return false;
    }
    let ss = senses(e);
    !ss.is_empty()
        && ss.iter().all(|s| {
            let tagged = s.get("tags").and_then(Value::as_array).is_some_and(|t| t.iter().any(|x| x.as_str() == Some("옛말")));
            let defd = s.get("ko_def").and_then(Value::as_str).is_some_and(|d| d.trim().trim_end_matches('.').ends_with("옛말"));
            tagged || defd
        })
}

fn insert_entry(conn: &Connection, e: &Entry, rank: i64, fts: bool, c: &mut Counters) -> Result<i64> {
    let hwn = hw_norm(&e.headword);
    let gloss = entry_gloss(e);
    let data = serde_json::to_string(&e.data)?;
    conn.prepare_cached(
        "INSERT INTO entries(headword,hw_norm,homonym,hanja,pos,pron,source,lang,level,rank,kind,gloss,data,ext_id,quality,hist) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )?
    .execute(params![e.headword, hwn, e.homonym, e.hanja, e.pos, e.pron, e.source, e.lang, e.level, rank, e.kind, gloss, data, e.ext_id, quality(e), i64::from(is_historical(e))])?;
    let id = conn.last_insert_rowid();
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(hwn.clone());
    for f in &e.forms {
        let n = hw_norm(f);
        if n.is_empty() || !seen.insert(n.clone()) {
            continue;
        }
        conn.prepare_cached("INSERT INTO forms(form, entry_id) VALUES(?,?)")?.execute(params![n, id])?;
        c.forms += 1;
    }
    if let Some(h) = &e.hanja {
        for ch in cjk_chars(h) {
            conn.prepare_cached("INSERT INTO hanja_words(ch, entry_id) VALUES(?,?)")?.execute(params![ch.to_string(), id])?;
        }
    }
    if fts {
        let q = quality(e);
        for (term, first) in gloss_items(e) {
            let tier = i64::from(!first);
            c.terms.push((term, tier, (tier + q) * 10_000_000 + rank.min(9_999_999), id));
        }
        let (head, en) = fts_text(e);
        if !head.is_empty() || !en.is_empty() {
            conn.prepare_cached("INSERT INTO entries_fts(rowid, head, en) VALUES(?,?,?)")?.execute(params![id, head, en])?;
        }
    }
    c.entries += 1;
    *c.by_source.entry(e.source).or_default() += 1;
    Ok(id)
}

pub(crate) struct GrammarRow {
    pub(crate) entry_id: i64,
    pub(crate) pattern: String,
    pub(crate) category: &'static str,
    pub(crate) level: Option<i64>,
    pub(crate) summary: Option<String>,
    pub(crate) key: String,
    pub(crate) rank: i64,
    pub(crate) jamo_last: bool,
}

/// Particle allomorph pairs that krdict lists as two entries with near-identical summaries.
const PARTICLE_PAIRS: [(&str, &str); 8] =
    [("이", "가"), ("은", "는"), ("을", "를"), ("와", "과"), ("으로", "로"), ("이랑", "랑"), ("아", "야"), ("이나", "나")];

fn word_set(s: &str) -> HashSet<String> {
    s.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(|w| w.to_lowercase()).collect()
}

/// Jaccard similarity of the word sets of two summaries (1.0 = identical wording).
fn summary_similarity(a: &str, b: &str) -> f64 {
    let (x, y) = (word_set(a), word_set(b));
    if x.is_empty() || y.is_empty() {
        return 0.0;
    }
    x.intersection(&y).count() as f64 / x.union(&y).count() as f64
}

/// Merge the two entries of each allomorph pair (이 + 가 -> "이/가") into one row: the first
/// entry's id is kept, the twin is dropped. The twin is the same-category entry of the second
/// form whose summary is most similar (>= 0.4), so homonyms with a different sense (야 "emphasis"
/// vs 아 "address") stay separate.
pub(crate) fn merge_particle_pairs(rows: &mut Vec<GrammarRow>) {
    for (a, b) in PARTICLE_PAIRS {
        let a_idx: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].category == "Particles" && rows[i].pattern == a).collect();
        let mut dropped: Vec<usize> = Vec::new();
        for ia in a_idx {
            let sa = rows[ia].summary.clone().unwrap_or_default();
            let best = (0..rows.len())
                .filter(|&j| rows[j].category == "Particles" && rows[j].pattern == b && !dropped.contains(&j))
                .map(|j| (summary_similarity(&sa, rows[j].summary.as_deref().unwrap_or("")), j))
                .filter(|(sim, _)| *sim >= 0.4)
                .max_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((_, jb)) = best {
                let (lvl, rank) = (rows[jb].level, rows[jb].rank);
                let r = &mut rows[ia];
                r.pattern = format!("{a}/{b}");
                r.key = format!("{a}/{b}");
                r.level = match (r.level, lvl) {
                    (Some(x), Some(y)) => Some(x.min(y)),
                    (x, y) => x.or(y),
                };
                r.rank = r.rank.min(rank);
                dropped.push(jb);
            }
        }
        dropped.sort_unstable();
        for j in dropped.into_iter().rev() {
            rows.remove(j);
        }
    }
}

/// Hand-picked beginner patterns, most common first: (category restriction, accepted `hw_norm`
/// keys of the krdict headword). Without a restriction any category except Particles / Affixes
/// matches. Patterns missing from the data are simply skipped.
const PRIORITY: &[(Option<&str>, &[&str])] = &[
    (Some("Particles"), &["이", "가"]),
    (Some("Particles"), &["은", "는"]),
    (Some("Particles"), &["을", "를"]),
    (Some("Particles"), &["에"]),
    (Some("Particles"), &["에서"]),
    (Some("Particles"), &["의"]),
    (Some("Particles"), &["도"]),
    (Some("Particles"), &["만"]),
    (Some("Particles"), &["와", "과"]),
    (Some("Particles"), &["하고"]),
    (Some("Particles"), &["으로", "로"]),
    (Some("Particles"), &["에게"]),
    (Some("Particles"), &["한테"]),
    (Some("Particles"), &["부터"]),
    (Some("Particles"), &["까지"]),
    (Some("Particles"), &["보다"]),
    (None, &["아요", "어요", "여요", "해요"]),
    (None, &["습니다", "ㅂ니다"]),
    (None, &["았", "었", "였", "았었"]),
    (None, &["겠"]),
    (None, &["고"]),
    (None, &["아서", "어서", "여서"]),
    (None, &["니까", "으니까", "(으)니까"]),
    (None, &["지만"]),
    (None, &["면", "으면", "(으)면"]),
    (None, &["는데", "ㄴ데", "은데", "(으)ㄴ데"]),
    (None, &["ㄹ거예요", "을거예요", "(으)ㄹ거예요", "ㄹ것이다", "을것이다", "(으)ㄹ것이다"]),
    (None, &["고싶다"]),
    (None, &["고있다"]),
    (None, &["아야하다", "어야하다", "여야하다"]),
    (None, &["ㄹ수있다", "을수있다", "(으)ㄹ수있다"]),
    (None, &["지않다"]),
    (None, &["세요", "으세요", "(으)세요"]),
    (None, &["려고", "으려고", "(으)려고"]),
    (None, &["기때문에", "기때문"]),
    (Some("Nominal/adnominal endings"), &["는", "ㄴ", "은", "(으)ㄴ"]),
    (Some("Nominal/adnominal endings"), &["ㄹ", "을", "(으)ㄹ"]),
    (None, &["기"]),
    (None, &["게"]),
    (None, &["도록"]),
];

/// Index into [`PRIORITY`] of the first beginner pattern this row is, or `PRIORITY.len()`.
pub(crate) fn grammar_priority(g: &GrammarRow) -> usize {
    if g.category == "Affixes" {
        return PRIORITY.len();
    }
    let parts: Vec<String> = g.pattern.split('/').map(hw_norm).collect();
    PRIORITY
        .iter()
        .position(|(cat, keys)| {
            let cat_ok = match cat {
                Some(c) => g.category == *c,
                None => g.category != "Particles",
            };
            cat_ok && parts.iter().any(|p| keys.contains(&p.as_str()))
        })
        .unwrap_or(PRIORITY.len())
}

/// Final order of the grammar table: category, beginner priority list, bare-jamo contractions
/// last, then level and frequency rank.
pub(crate) fn sort_grammar(rows: &mut Vec<GrammarRow>) {
    merge_particle_pairs(rows);
    rows.sort_by_cached_key(|g| (cat_order(g.category), grammar_priority(g), g.jamo_last, g.level.unwrap_or(4), g.rank, g.key.clone()));
}

const CATEGORIES: [&str; 6] =
    ["Particles", "Connective endings", "Final endings", "Pre-final endings", "Nominal/adnominal endings", "Expressions"];
const CAT_AFFIX: &str = "Affixes";

fn grammar_category(e: &Entry) -> Option<&'static str> {
    if e.ko_pos == "조사" || e.pos == "particle" {
        return Some("Particles");
    }
    if e.ko_pos == "어미" || e.pos == "ending" {
        let d = e.ko_defs.join(" ");
        return Some(if d.contains("선어말") {
            "Pre-final endings"
        } else if d.contains("연결") {
            "Connective endings"
        } else if d.contains("종결") {
            "Final endings"
        } else if d.contains("전성") || d.contains("관형사형") || d.contains("명사형") || d.contains("관형") {
            "Nominal/adnominal endings"
        } else {
            "Final endings"
        });
    }
    if e.kind == "grammar" {
        return Some("Expressions");
    }
    if e.pos == "affix" {
        return Some(CAT_AFFIX);
    }
    None
}

fn cat_order(c: &str) -> usize {
    CATEGORIES.iter().position(|x| *x == c).unwrap_or(CATEGORIES.len())
}

pub(crate) fn open_db(path: &Path) -> Result<Connection> {
    for suffix in ["", "-journal", "-wal", "-shm"] {
        let p = PathBuf::from(format!("{}{}", path.display(), suffix));
        if p.exists() {
            fs::remove_file(&p)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA page_size=4096; PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-262144;")?;
    Ok(conn)
}

pub(crate) fn finish_db(conn: Connection) -> Result<()> {
    conn.execute_batch("ANALYZE;")?;
    conn.execute_batch("PRAGMA journal_mode=DELETE; VACUUM;")?;
    // verify header-level settings
    let ps: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    anyhow::ensure!(ps == 4096, "page_size is {ps}");
    let jm: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    anyhow::ensure!(jm.eq_ignore_ascii_case("delete"), "journal_mode is {jm}");
    conn.close().map_err(|(_, e)| e)?;
    Ok(())
}

pub(crate) fn set_meta(conn: &Connection, k: &str, v: &str) -> Result<()> {
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES(?,?)", params![k, v])?;
    Ok(())
}

pub struct PackResult {
    pub path: PathBuf,
    pub counts: Value,
    /// "hw_norm\tpos" keys of every entry (used to dedupe opendict against stdict)
    pub keys: HashSet<String>,
}

fn source_info(name: &str, n: i64) -> Value {
    let (lic, url) = match name {
        "krdict" => ("CC BY-SA 2.0 KR", "https://krdict.korean.go.kr (via https://github.com/spellcheck-ko/korean-dict-nikl)"),
        "stdict" => ("CC BY-SA 2.0 KR", "https://stdict.korean.go.kr (via https://github.com/spellcheck-ko/korean-dict-nikl)"),
        "opendict" => ("CC BY-SA 2.0 KR", "https://opendict.korean.go.kr (via https://github.com/spellcheck-ko/korean-dict-nikl)"),
        "wikt" => ("CC BY-SA 4.0", "https://kaikki.org/dictionary/Korean/"),
        "kengdic" => ("MPL 2.0 / LGPL", "https://github.com/garfieldnate/kengdic"),
        "tatoeba" => ("CC BY 2.0 FR", "https://tatoeba.org"),
        "cedict" => ("CC BY-SA 4.0", "https://www.mdbg.net/chinese/dictionary?page=cc-cedict"),
        "zhwikt" => ("CC BY-SA 4.0", "https://kaikki.org/dictionary/Chinese/"),
        "unihan" => ("Unicode License", "https://www.unicode.org/charts/unihan.html"),
        "freq" => ("CC BY-SA 4.0", "https://github.com/hermitdave/FrequencyWords"),
        _ => ("", ""),
    };
    json!({"license": lic, "url": url, "count": n})
}

// --- core pack -------------------------------------------------------------

/// Fill `entries.cho` (initial consonants of `hw_norm`) and index it.
fn fill_cho(conn: &Connection) -> Result<()> {
    let rows: Vec<(i64, String)> = {
        let mut st = conn.prepare("SELECT id, hw_norm FROM entries")?;
        let it = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        it.collect::<std::result::Result<_, _>>()?
    };
    {
        let mut up = conn.prepare("UPDATE entries SET cho = ? WHERE id = ?")?;
        for (id, hw) in rows {
            if let Some(cho) = choseong(&hw) {
                up.execute(params![cho, id])?;
            }
        }
    }
    conn.execute_batch(schema::CHO_INDEX)?;
    Ok(())
}

pub fn build_core(src: &Sources, ranker: &Ranker, out: &Path, version: &str, built_at: &str) -> Result<PackResult> {
    let path = out.join("core.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch(schema::CORE_ONLY)?;
    conn.execute_batch(schema::CHO_COLUMN)?;
    conn.execute_batch("BEGIN")?;
    let mut c = Counters::default();
    let mut grammar: Vec<GrammarRow> = Vec::new();

    // kengdic first: needed for the hanja fallback
    let kd = match &src.kengdic {
        Some(p) => {
            let t = Instant::now();
            match fs::File::open(p).map_err(anyhow::Error::from).and_then(|f| kengdic::parse(BufReader::new(f))) {
                Ok(k) => {
                    log::info!("kengdic: {} grouped entries from {} surfaces ({:.1}s)", k.entries.len(), k.hanja_by_surface.len(), t.elapsed().as_secs_f32());
                    Some(k)
                }
                Err(e) => {
                    log::warn!("kengdic failed: {e:#}");
                    None
                }
            }
        }
        None => None,
    };

    // krdict
    let mut krdict_ids: HashMap<String, Vec<(Option<i64>, i64)>> = HashMap::new();
    let mut pointers: Vec<(String, Pointer)> = Vec::new();
    for p in &src.krdict {
        let t = Instant::now();
        let before = c.entries;
        krdict::parse_file(p, |mut e| {
            if let Some(ptr) = e.pointer.take() {
                pointers.push((e.headword.clone(), ptr));
                return Ok(());
            }
            let rank = ranker.rank(&e);
            let id = insert_entry(&conn, &e, rank, true, &mut c)?;
            krdict_ids.entry(hw_norm(&e.headword)).or_default().push((e.homonym, id));
            if let Some(cat) = grammar_category(&e) {
                let summary = senses(&e).iter().find_map(|s| s.get("def").or_else(|| s.get("ko_def")).and_then(Value::as_str)).map(|d| truncate(d, 200));
                // bare-jamo particles (ㄴ, ㄹ랑...) are contractions: list them after the full forms
                let jamo_last = cat == "Particles" && e.headword.chars().next().is_some_and(|c| ('\u{3131}'..='\u{318E}').contains(&c));
                grammar.push(GrammarRow { entry_id: id, pattern: e.headword.clone(), category: cat, level: e.level, summary, key: hw_norm(&e.headword), rank, jamo_last });
            }
            Ok(())
        })
        .with_context(|| format!("krdict {}", p.display()))?;
        log::info!("krdict {}: {} entries ({:.1}s)", p.file_name().unwrap().to_string_lossy(), c.entries - before, t.elapsed().as_secs_f32());
    }
    // conjugation-pointer entries -> forms of the entries they point to
    {
        let (mut n_forms, mut unresolved) = (0i64, 0i64);
        for (hw, ptr) in &pointers {
            let mut forms: Vec<String> = ptr.forms.iter().map(|f| hw_norm(f)).collect();
            forms.push(hw_norm(hw)); // the bare stem
            for (lemma, hom) in &ptr.targets {
                let ln = hw_norm(lemma);
                let ids: Vec<i64> = krdict_ids
                    .get(&ln)
                    .map(|v| v.iter().filter(|(h, _)| hom.is_none() || h == hom).map(|(_, id)| *id).collect())
                    .unwrap_or_default();
                if ids.is_empty() {
                    unresolved += 1;
                }
                for id in ids {
                    for f in forms.iter().filter(|f| !f.is_empty() && **f != ln) {
                        conn.prepare_cached("INSERT INTO forms(form, entry_id) VALUES(?,?)")?.execute(params![f, id])?;
                        n_forms += 1;
                    }
                }
            }
        }
        log::info!("krdict pointer entries: {} dropped, {} forms added, {} unresolved targets", pointers.len(), n_forms, unresolved);
    }

    // wiktionary
    let mut wikt_norms: HashSet<String> = HashSet::new();
    let mut wikt_sentences: Vec<(String, String)> = Vec::new();
    let mut wikt_hanja: HashMap<char, kaikki::HanjaHun> = HashMap::new();
    if let Some(p) = &src.kaikki {
        let t = Instant::now();
        let parsed = fs::File::open(p).map_err(anyhow::Error::from).and_then(|f| kaikki::parse(BufReader::with_capacity(1 << 20, f)));
        match parsed {
            Ok(k) => {
                for e in &k.entries {
                    let rank = ranker.rank(e);
                    insert_entry(&conn, e, rank, true, &mut c)?;
                    wikt_norms.insert(hw_norm(&e.headword));
                }
                // form-of / alt-of -> target
                let mut added = 0i64;
                let mut seen: HashSet<(String, String)> = HashSet::new();
                for (form, target) in &k.redirects {
                    let (fnorm, tnorm) = (hw_norm(form), hw_norm(target));
                    if fnorm == tnorm || !seen.insert((fnorm.clone(), tnorm.clone())) {
                        continue;
                    }
                    let ids: Vec<i64> = {
                        let mut st = conn.prepare_cached("SELECT id FROM entries WHERE hw_norm=? AND source IN ('wikt','krdict') ORDER BY source DESC, id LIMIT 3")?;
                        let rows = st.query_map([&tnorm], |r| r.get(0))?;
                        rows.collect::<std::result::Result<_, _>>()?
                    };
                    for id in ids {
                        conn.prepare_cached("INSERT INTO forms(form, entry_id) VALUES(?,?)")?.execute(params![fnorm, id])?;
                        added += 1;
                    }
                }
                c.forms += added;
                wikt_sentences = k.sentences;
                wikt_hanja = k.hanja;
                log::info!("wikt: {} entries, {} redirects -> {} forms, {} example pairs ({:.1}s)", k.entries.len(), k.redirects.len(), added, wikt_sentences.len(), t.elapsed().as_secs_f32());
            }
            Err(e) => log::warn!("kaikki failed: {e:#}"),
        }
    }

    // kengdic entries
    if let Some(k) = &kd {
        // kengdic-only words: skip surfaces that krdict or wikt already cover
        let mut skipped = 0;
        for e in &k.entries {
            let n = hw_norm(&e.headword);
            if krdict_ids.contains_key(&n) || wikt_norms.contains(&n) {
                skipped += 1;
                continue;
            }
            let rank = ranker.rank(e);
            insert_entry(&conn, e, rank, true, &mut c)?;
        }
        log::info!("kengdic: {} kept (kengdic-only), {} skipped as already in krdict/wikt", k.entries.len() - skipped, skipped);
        // hanja fallback for krdict / wikt entries lacking it
        let mut cnt: HashMap<(String, String), i64> = HashMap::new();
        {
            let mut st = conn.prepare("SELECT source, hw_norm FROM entries WHERE source IN ('krdict','wikt')")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
                *cnt.entry(r?).or_default() += 1;
            }
        }
        let cands: Vec<(i64, String, String)> = {
            let mut st = conn.prepare(
                "SELECT id, source, hw_norm FROM entries WHERE source IN ('krdict','wikt') AND hanja IS NULL AND pos='noun' AND kind='word' AND data NOT LIKE '%\"origin_note\"%'",
            )?;
            let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            rows.collect::<std::result::Result<_, _>>()?
        };
        let mut filled = 0;
        for (id, source, hwn) in cands {
            if cnt.get(&(source, hwn.clone())) != Some(&1) {
                continue;
            }
            let Some(set) = k.hanja_by_surface.get(&hwn) else { continue };
            if set.len() != 1 {
                continue;
            }
            let h = set.iter().next().unwrap();
            conn.execute("UPDATE entries SET hanja=? WHERE id=?", params![h, id])?;
            for ch in cjk_chars(h) {
                conn.execute("INSERT INTO hanja_words(ch, entry_id) VALUES(?,?)", params![ch.to_string(), id])?;
            }
            filled += 1;
        }
        log::info!("kengdic hanja fallback filled {filled} entries");
    }

    // sentences: Tatoeba + wikt
    let mut sent_seen: HashSet<String> = HashSet::new();
    let mut n_tat = 0i64;
    if let Some((kor, eng, links)) = &src.tatoeba {
        let t = Instant::now();
        let r = (|| -> Result<Vec<tatoeba::Pair>> {
            let kf = bzip2::read::MultiBzDecoder::new(BufReader::new(fs::File::open(kor)?));
            let ef = bzip2::read::MultiBzDecoder::new(BufReader::new(fs::File::open(eng)?));
            tatoeba::pairs(kf, ef, |cb| {
                let lf = bzip2::read::MultiBzDecoder::new(BufReader::new(fs::File::open(links)?));
                tatoeba::iter_links_tar(lf, |a, b| cb(a, b))
            })
        })();
        match r {
            Ok(pairs) => {
                for p in pairs {
                    if sent_seen.insert(p.ko.clone()) {
                        conn.prepare_cached("INSERT INTO sentences(ko,en,source) VALUES(?,?,'tatoeba')")?.execute(params![p.ko, p.en])?;
                        n_tat += 1;
                    }
                }
                log::info!("tatoeba: {n_tat} pairs ({:.1}s)", t.elapsed().as_secs_f32());
            }
            Err(e) => log::warn!("tatoeba failed: {e:#}"),
        }
    }
    let mut n_wikt_s = 0i64;
    for (ko, en) in wikt_sentences {
        if sent_seen.insert(ko.clone()) {
            conn.prepare_cached("INSERT INTO sentences(ko,en,source) VALUES(?,?,'wikt')")?.execute(params![ko, en])?;
            n_wikt_s += 1;
        }
    }
    conn.execute_batch("INSERT INTO sentences_fts(sentences_fts) VALUES('rebuild')")?;

    // grammar table
    sort_grammar(&mut grammar);
    for (i, g) in grammar.iter().enumerate() {
        conn.prepare_cached("INSERT INTO grammar(entry_id,pattern,category,level,summary_en,sort) VALUES(?,?,?,?,?,?)")?
            .execute(params![g.entry_id, g.pattern, g.category, g.level, g.summary, i as i64])?;
    }

    // gloss_terms: sorted so the WITHOUT ROWID table is built by appending
    let mut terms = std::mem::take(&mut c.terms);
    terms.sort_unstable();
    terms.dedup_by(|a, b| a.0 == b.0 && a.3 == b.3 && a.2 >= b.2);
    {
        let mut st = conn.prepare("INSERT OR IGNORE INTO gloss_terms(term,tier,score,entry_id) VALUES(?,?,?,?)")?;
        for (term, tier, score, id) in &terms {
            st.execute(params![term, tier, score, id])?;
        }
    }
    conn.execute_batch(
        "INSERT INTO wotd(n, entry_id) SELECT row_number() OVER (ORDER BY id) - 1, id FROM entries \
         WHERE source = 'krdict' AND level IN (1, 2) AND kind = 'word' AND gloss IS NOT NULL AND gloss != ''",
    )?;

    conn.execute_batch(schema::FILL_HANJA_RANK)?;
    conn.execute_batch(schema::COMMON_INDEXES)?;
    fill_cho(&conn)?;
    conn.execute_batch("DELETE FROM forms WHERE rowid NOT IN (SELECT MIN(rowid) FROM forms GROUP BY form, entry_id)")?;

    // hanja characters
    let uni = match &src.unihan {
        Some(p) => match unihan::parse(p) {
            Ok(m) => {
                log::info!("unihan: {} characters", m.len());
                m
            }
            Err(e) => {
                log::warn!("unihan failed: {e:#}");
                HashMap::new()
            }
        },
        None => HashMap::new(),
    };
    let wc: HashMap<String, i64> = {
        let mut st = conn.prepare("SELECT hw.ch, COUNT(DISTINCT e.hw_norm) FROM hanja_words hw JOIN entries e ON e.id=hw.entry_id GROUP BY hw.ch")?;
        let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        rows.collect::<std::result::Result<_, _>>()?
    };
    let mut chars: Vec<char> = wc.keys().filter_map(|k| k.chars().next()).collect();
    for (ch, info) in &uni {
        if info.has_hangul && !wc.contains_key(&ch.to_string()) {
            chars.push(*ch);
        }
    }
    // characters with a Wiktionary 훈음 but neither a word nor a Unihan reading
    for ch in wikt_hanja.keys() {
        if !chars.contains(ch) && (wikt_hanja[ch].eumhun.len() + wikt_hanja[ch].eum.len()) > 0 {
            chars.push(*ch);
        }
    }
    chars.sort();
    let mut n_hun = 0i64;
    for ch in &chars {
        let s = ch.to_string();
        let info = uni.get(ch);
        let hh = wikt_hanja.get(ch);
        // Unihan readings first, then any further Wiktionary readings
        let mut rd: Vec<String> = info.map(|i| i.readings.clone()).unwrap_or_default();
        for e in hh.map(|h| h.eum.as_slice()).unwrap_or(&[]) {
            if !rd.contains(e) {
                rd.push(e.clone());
            }
        }
        let readings = Some(rd.join(",")).filter(|r| !r.is_empty());
        let hun = hh.and_then(|h| h.hun.first().cloned());
        let eumhun = hh.map(|h| h.eumhun.join("; ")).filter(|x| !x.is_empty());
        if hun.is_some() {
            n_hun += 1;
        }
        conn.execute(
            "INSERT INTO hanja_chars(ch,readings,meaning_en,strokes,radical,word_count,radical_num,hun,eumhun) VALUES(?,?,?,?,?,?,?,?,?)",
            params![s, readings, info.and_then(|i| i.meaning_en.clone()), info.and_then(|i| i.strokes), info.and_then(|i| i.radical.clone()), wc.get(&s).copied().unwrap_or(0), info.and_then(|i| i.radical_num), hun, eumhun],
        )?;
    }

    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    for (s, n) in &c.by_source {
        counts.insert((*s).into(), (*n).into());
    }
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
    counts.insert("hanja_chars".into(), q("SELECT COUNT(*) FROM hanja_chars")?.into());
    counts.insert("hanja_hun".into(), n_hun.into());
    counts.insert("sentences".into(), q("SELECT COUNT(*) FROM sentences")?.into());
    counts.insert("sentences_tatoeba".into(), n_tat.into());
    counts.insert("sentences_wikt".into(), n_wikt_s.into());
    counts.insert("grammar".into(), q("SELECT COUNT(*) FROM grammar")?.into());
    counts.insert("gloss_terms".into(), q("SELECT COUNT(*) FROM gloss_terms")?.into());
    let counts = Value::Object(counts);

    let mut sources = Map::new();
    for s in ["krdict", "wikt", "kengdic"] {
        sources.insert(s.into(), source_info(s, c.by_source.get(s).copied().unwrap_or(0)));
    }
    sources.insert("tatoeba".into(), source_info("tatoeba", n_tat));
    sources.insert("unihan".into(), source_info("unihan", uni.len() as i64));
    sources.insert("freq".into(), source_info("freq", ranker.ranks.len() as i64));

    set_meta(&conn, "pack", "core")?;
    set_meta(&conn, "schema", "1")?;
    set_meta(&conn, "version", version)?;
    set_meta(&conn, "built_at", built_at)?;
    set_meta(&conn, "counts", &counts.to_string())?;
    set_meta(&conn, "sources", &Value::Object(sources).to_string())?;
    conn.execute_batch("COMMIT")?;
    finish_db(conn)?;
    Ok(PackResult { path, counts, keys: HashSet::new() })
}

// --- stdict pack ---------------------------------------------------------------

pub fn build_stdict(src: &Sources, ranker: &Ranker, out: &Path, version: &str, built_at: &str) -> Result<PackResult> {
    let path = out.join("stdict.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch("BEGIN")?;
    let mut c = Counters::default();
    let mut keys: HashSet<String> = HashSet::new();
    for p in &src.stdict {
        let t = Instant::now();
        let before = c.entries;
        stdict::parse_file(p, |e| {
            keys.insert(format!("{}\t{}", hw_norm(&e.headword), e.pos));
            let rank = ranker.rank(&e);
            insert_entry(&conn, &e, rank, false, &mut c)?;
            Ok(())
        })
        .with_context(|| format!("stdict {}", p.display()))?;
        log::info!("stdict {}: {} entries ({:.1}s)", p.file_name().unwrap().to_string_lossy(), c.entries - before, t.elapsed().as_secs_f32());
    }
    conn.execute_batch(schema::FILL_HANJA_RANK)?;
    conn.execute_batch(schema::COMMON_INDEXES)?;
    conn.execute_batch(schema::HIST_INDEX)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
    counts.insert("hist".into(), q("SELECT COUNT(*) FROM entries WHERE hist = 1")?.into());
    let counts = Value::Object(counts);
    let mut sources = Map::new();
    sources.insert("stdict".into(), source_info("stdict", c.entries));
    set_meta(&conn, "pack", "stdict")?;
    set_meta(&conn, "schema", "1")?;
    set_meta(&conn, "version", version)?;
    set_meta(&conn, "built_at", built_at)?;
    set_meta(&conn, "counts", &counts.to_string())?;
    set_meta(&conn, "sources", &Value::Object(sources).to_string())?;
    conn.execute_batch("COMMIT")?;
    finish_db(conn)?;
    Ok(PackResult { path, counts, keys })
}

pub fn build_opendict(src: &Sources, ranker: &Ranker, out: &Path, version: &str, built_at: &str, dedupe: &HashSet<String>) -> Result<PackResult> {
    let path = out.join("opendict.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch("BEGIN")?;
    let mut c = Counters::default();
    let mut skipped = 0i64;
    for p in &src.opendict {
        let t = Instant::now();
        let before = c.entries;
        let skipped_before = skipped;
        opendict::parse_file(p, |e| {
            if dedupe.contains(&format!("{}\t{}", hw_norm(&e.headword), e.pos)) {
                skipped += 1;
                return Ok(());
            }
            let rank = ranker.rank(&e);
            insert_entry(&conn, &e, rank, false, &mut c)?;
            Ok(())
        })
        .with_context(|| format!("opendict {}", p.display()))?;
        log::info!(
            "opendict {}: {} entries, {} skipped as stdict duplicates ({:.1}s)",
            p.file_name().unwrap().to_string_lossy(),
            c.entries - before,
            skipped - skipped_before,
            t.elapsed().as_secs_f32()
        );
    }
    conn.execute_batch(schema::FILL_HANJA_RANK)?;
    conn.execute_batch(schema::COMMON_INDEXES)?;
    conn.execute_batch(schema::HIST_INDEX)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
    counts.insert("hist".into(), q("SELECT COUNT(*) FROM entries WHERE hist = 1")?.into());
    counts.insert("skipped_stdict_duplicates".into(), skipped.into());
    let counts = Value::Object(counts);
    let mut sources = Map::new();
    sources.insert("opendict".into(), source_info("opendict", c.entries));
    set_meta(&conn, "pack", "opendict")?;
    set_meta(&conn, "schema", "1")?;
    set_meta(&conn, "version", version)?;
    set_meta(&conn, "built_at", built_at)?;
    set_meta(&conn, "counts", &counts.to_string())?;
    set_meta(&conn, "sources", &Value::Object(sources).to_string())?;
    conn.execute_batch("COMMIT")?;
    finish_db(conn)?;
    Ok(PackResult { path, counts, keys: HashSet::new() })
}

// --- cedict / zhwikt packs (optional, Chinese) --------------------------------------------

/// First Sino-Korean reading of every hanja in a built core pack (`hanja_chars.readings`).
pub fn load_sino_readings(core: &Path) -> HashMap<char, String> {
    let mut m = HashMap::new();
    let Ok(conn) = Connection::open_with_flags(core, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { return m };
    let Ok(mut st) = conn.prepare("SELECT ch, readings FROM hanja_chars WHERE readings IS NOT NULL AND readings != ''") else { return m };
    let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
    if let Ok(rows) = rows {
        for (ch, rd) in rows.flatten() {
            if let (Some(c), Some(first)) = (ch.chars().next(), rd.split(',').next()) {
                m.insert(c, first.to_string());
            }
        }
    }
    m
}

/// Insert an entry with an explicit rank (Chinese packs have no frequency data).
fn insert_ranked(conn: &Connection, e: &Entry, rank: i64, c: &mut Counters) -> Result<i64> {
    insert_entry(conn, e, rank, false, c)
}

fn finish_pack(conn: Connection, pack: &str, counts: Value, src: &str, n: i64, version: &str, built_at: &str) -> Result<()> {
    let mut sources = Map::new();
    sources.insert(src.into(), source_info(src, n));
    set_meta(&conn, "pack", pack)?;
    set_meta(&conn, "schema", "1")?;
    set_meta(&conn, "version", version)?;
    set_meta(&conn, "built_at", built_at)?;
    set_meta(&conn, "counts", &counts.to_string())?;
    set_meta(&conn, "sources", &Value::Object(sources).to_string())?;
    conn.execute_batch("COMMIT")?;
    finish_db(conn)
}

/// `cedict.sqlite`: CC-CEDICT. headword = hanja = traditional; `forms.form` = simplified (the
/// "extra index on simplified": `forms_form`); `pron` = Sino-Korean reading; `rank` = headword
/// length then file order.
pub fn build_cedict(src: &Sources, sino: &HashMap<char, String>, out: &Path, version: &str, built_at: &str) -> Result<Option<PackResult>> {
    let Some(p) = &src.cedict else { return Ok(None) };
    let path = out.join("cedict.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch("BEGIN")?;
    let mut c = Counters::default();
    let mut seq = 0usize;
    cedict::parse_file(p, sino, |e| {
        let rank = cedict::rank(&e.headword, seq);
        seq += 1;
        insert_ranked(&conn, &e, rank, &mut c)?;
        Ok(())
    })
    .with_context(|| format!("cedict {}", p.display()))?;
    conn.execute_batch(schema::FILL_HANJA_RANK)?;
    conn.execute_batch(schema::COMMON_INDEXES)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
    counts.insert("with_sino_korean".into(), q("SELECT COUNT(*) FROM entries WHERE pron IS NOT NULL")?.into());
    let counts = Value::Object(counts);
    finish_pack(conn, "cedict", counts.clone(), "cedict", c.entries, version, built_at)?;
    Ok(Some(PackResult { path, counts, keys: HashSet::new() }))
}

/// `zhwikt.sqlite`: Wiktionary Chinese (kaikki), streamed. Form-of redirects (simplified ->
/// traditional, variants) become `forms` rows of the target entries.
pub fn build_zhwikt(src: &Sources, sino: &HashMap<char, String>, out: &Path, version: &str, built_at: &str) -> Result<Option<PackResult>> {
    let Some(p) = &src.zhwikt else { return Ok(None) };
    let path = out.join("zhwikt.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch("BEGIN")?;
    let mut c = Counters::default();
    let rd = BufReader::with_capacity(1 << 20, fs::File::open(p)?);
    let redirects = zhwikt::parse(rd, sino, |e, rank| {
        insert_ranked(&conn, &e, rank, &mut c)?;
        Ok(())
    })
    .with_context(|| format!("zhwikt {}", p.display()))?;
    let mut added = 0i64;
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for (form, target) in &redirects {
        let (f, t) = (hw_norm(form), hw_norm(target));
        if f == t || !seen.insert((f.clone(), t.clone())) {
            continue;
        }
        let ids: Vec<i64> = {
            let mut st = conn.prepare_cached("SELECT id FROM entries WHERE hw_norm = ? ORDER BY rank LIMIT 3")?;
            let rows = st.query_map([&t], |r| r.get(0))?;
            rows.collect::<std::result::Result<_, _>>()?
        };
        for id in ids {
            conn.prepare_cached("INSERT INTO forms(form, entry_id) VALUES(?,?)")?.execute(params![f, id])?;
            added += 1;
        }
    }
    conn.execute_batch("DELETE FROM forms WHERE rowid NOT IN (SELECT MIN(rowid) FROM forms GROUP BY form, entry_id)")?;
    conn.execute_batch(schema::FILL_HANJA_RANK)?;
    conn.execute_batch(schema::COMMON_INDEXES)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
    counts.insert("redirect_forms".into(), added.into());
    counts.insert("with_sino_korean".into(), q("SELECT COUNT(*) FROM entries WHERE pron IS NOT NULL")?.into());
    let counts = Value::Object(counts);
    finish_pack(conn, "zhwikt", counts.clone(), "zhwikt", c.entries, version, built_at)?;
    Ok(Some(PackResult { path, counts, keys: HashSet::new() }))
}

// --- driver ------------------------------------------------------------------------

pub struct BuildOpts {
    pub out: PathBuf,
    pub sources: Sources,
    pub chunk_bytes: u64,
    /// Skip the data-quality gate (local partial builds, `--limit-files`, tests).
    pub allow_partial: bool,
    /// Reader library sources (`catalog.toml`, `raw/`, `enriched/`); `None` skips the `texts` pack.
    pub texts: Option<PathBuf>,
}

/// Minimum row counts of a healthy full build (core pack).
pub const MIN_CORE: [(&str, i64); 5] = [("entries", 150_000), ("krdict", 50_000), ("wikt", 20_000), ("sentences", 10_000), ("hanja_chars", 5_000)];
/// Minimum entries of a healthy stdict pack.
pub const MIN_STDICT: i64 = 400_000;

/// Problems that make a build look degraded (a source silently failed to download or parse);
/// empty when the counts are at full-build level. `stdict` is `None` when it was not built.
pub fn quality_gate(core: &Value, stdict: Option<&Value>) -> Vec<String> {
    let n = |v: &Value, k: &str| v.get(k).and_then(Value::as_i64).unwrap_or(0);
    let mut bad = Vec::new();
    for (k, min) in MIN_CORE {
        if n(core, k) < min {
            bad.push(format!("core.{k} = {} (< {min})", n(core, k)));
        }
    }
    if let Some(st) = stdict {
        if n(st, "entries") < MIN_STDICT {
            bad.push(format!("stdict.entries = {} (< {MIN_STDICT})", n(st, "entries")));
        }
    }
    bad
}

/// Minimum entries of a healthy cedict pack (CC-CEDICT has ~120k lines).
pub const MIN_CEDICT: i64 = 100_000;
/// Minimum entries of a healthy zhwikt pack (kaikki Chinese has several 100k non-form-of entries).
pub const MIN_ZHWIKT: i64 = 100_000;

/// Gate for the optional packs; `built` holds `(pack id, counts)` for the packs that were
/// built in this run only (a missing source is not a failure).
pub fn quality_gate_optional(built: &[(&str, &Value)]) -> Vec<String> {
    let mut bad = Vec::new();
    for (id, counts) in built {
        let min = match *id {
            "cedict" => MIN_CEDICT,
            "zhwikt" => MIN_ZHWIKT,
            "texts" => crate::texts_pack::MIN_TEXTS,
            _ => continue,
        };
        let key = if *id == "texts" { "texts" } else { "entries" };
        let n = counts.get(key).and_then(Value::as_i64).unwrap_or(0);
        if n < min {
            bad.push(format!("{id}.{key} = {n} (< {min})"));
        }
    }
    bad
}

fn remove_stale(site: &Path, prefix: &str) -> Result<()> {
    for e in fs::read_dir(site)? {
        let e = e?;
        if e.file_name().to_string_lossy().starts_with(prefix) {
            fs::remove_file(e.path())?;
        }
    }
    Ok(())
}

pub fn run(o: &BuildOpts) -> Result<Value> {
    let t0 = Instant::now();
    fs::create_dir_all(&o.out)?;
    let site = o.out.join("site-data");
    fs::create_dir_all(&site)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
    let (version, built_at) = stamp(now);

    let ranker = match &o.sources.freq {
        Some(p) => {
            let r = Ranker::new(freq::load_ranks(BufReader::new(fs::File::open(p)?)));
            log::info!("freq: {} words, {} stems", r.ranks.len(), r.stems.len());
            r
        }
        None => Ranker::new(HashMap::new()),
    };

    let mut packs = Vec::new();
    if o.sources.krdict.is_empty() && o.sources.kaikki.is_none() && o.sources.kengdic.is_none() {
        anyhow::bail!("no English dictionary source available (krdict / kaikki / kengdic): nothing to build for core");
    }
    let core = build_core(&o.sources, &ranker, &o.out, &version, &built_at)?;
    log::info!("core.sqlite built: {}", core.counts);
    packs.push(pack::compress_pack("core", true, &core.path, &site, &core.counts, o.chunk_bytes)?);

    let mut dedupe: HashSet<String> = HashSet::new();
    let mut stdict_counts: Option<Value> = None;
    if o.sources.stdict.is_empty() {
        log::warn!("no stdict files: skipping the stdict pack");
        remove_stale(&site, "stdict.sqlite.gz.")?;
    } else {
        let st = build_stdict(&o.sources, &ranker, &o.out, &version, &built_at)?;
        log::info!("stdict.sqlite built: {}", st.counts);
        packs.push(pack::compress_pack("stdict", false, &st.path, &site, &st.counts, o.chunk_bytes)?);
        stdict_counts = Some(st.counts.clone());
        dedupe = st.keys;
    }
    if o.sources.opendict.is_empty() {
        remove_stale(&site, "opendict.sqlite.gz.")?;
    } else {
        let od = build_opendict(&o.sources, &ranker, &o.out, &version, &built_at, &dedupe)?;
        log::info!("opendict.sqlite built: {}", od.counts);
        packs.push(pack::compress_pack("opendict", false, &od.path, &site, &od.counts, o.chunk_bytes)?);
    }

    // optional Chinese packs; a failing one only logs a warning
    let sino = if o.sources.cedict.is_some() || o.sources.zhwikt.is_some() { load_sino_readings(&core.path) } else { HashMap::new() };
    let mut zh_counts: Vec<(&str, Value)> = Vec::new();
    if o.sources.cedict.is_none() {
        remove_stale(&site, "cedict.sqlite.gz.")?;
    } else {
        match build_cedict(&o.sources, &sino, &o.out, &version, &built_at) {
            Ok(Some(r)) => {
                log::info!("cedict.sqlite built: {}", r.counts);
                packs.push(pack::compress_pack("cedict", false, &r.path, &site, &r.counts, o.chunk_bytes)?);
                zh_counts.push(("cedict", r.counts));
            }
            Ok(None) => {}
            Err(e) => log::warn!("cedict pack failed, skipping it: {e:#}"),
        }
    }
    if o.sources.zhwikt.is_none() {
        remove_stale(&site, "zhwikt.sqlite.gz.")?;
    } else {
        match build_zhwikt(&o.sources, &sino, &o.out, &version, &built_at) {
            Ok(Some(r)) => {
                log::info!("zhwikt.sqlite built: {}", r.counts);
                packs.push(pack::compress_pack("zhwikt", false, &r.path, &site, &r.counts, o.chunk_bytes)?);
                zh_counts.push(("zhwikt", r.counts));
            }
            Ok(None) => {}
            Err(e) => log::warn!("zhwikt pack failed, skipping it: {e:#}"),
        }
    }

    // optional Reader library; a source problem (paragraph mismatch, unresolved vocab) fails the build
    match &o.texts {
        Some(dir) => {
            let r = crate::texts_pack::build_texts(&crate::texts_pack::TextsOpts { dir, core: &core.path, extra: &[o.out.join("stdict.sqlite"), o.out.join("opendict.sqlite")], out: &o.out, version: &version, built_at: &built_at, allow_partial: o.allow_partial })?;
            match r {
                Some(r) => {
                    log::info!("texts.sqlite built: {}", r.counts);
                    let mut p = pack::compress_pack("texts", false, &r.path, &site, &r.counts, o.chunk_bytes)?;
                    p["label"] = json!("Reader library");
                    packs.push(p);
                    zh_counts.push(("texts", r.counts));
                }
                None => {
                    log::warn!("no texts to pack: skipping the texts pack");
                    remove_stale(&site, "texts.sqlite.gz.")?;
                }
            }
        }
        None => remove_stale(&site, "texts.sqlite.gz.")?,
    }

    if !o.allow_partial {
        let mut problems = quality_gate(&core.counts, stdict_counts.as_ref());
        let built: Vec<(&str, &Value)> = zh_counts.iter().map(|(i, v)| (*i, v)).collect();
        problems.extend(quality_gate_optional(&built));
        if !problems.is_empty() {
            anyhow::bail!("data quality gate failed, the build looks degraded (a source failed to download or parse?): {}. Pass --allow-partial for local partial builds.", problems.join("; "));
        }
    } else {
        log::warn!("--allow-partial: data quality gate skipped");
    }

    let manifest = json!({"version": version, "packs": packs});
    fs::write(site.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    fs::write(site.join("ATTRIBUTION.md"), ATTRIBUTION)?;
    log::info!("build finished in {:.1}s", t0.elapsed().as_secs_f32());
    Ok(manifest)
}

#[cfg(test)]
mod grammar_tests {
    use super::*;

    fn row(id: i64, pat: &str, cat: &'static str, level: Option<i64>, rank: i64, summary: &str) -> GrammarRow {
        GrammarRow { entry_id: id, pattern: pat.into(), category: cat, level, summary: Some(summary.into()), key: hw_norm(pat), rank, jamo_last: false }
    }

    #[test]
    fn allomorph_pairs_merge_and_priority_orders() {
        let subj = "A postpositional particle referring to a subject under a certain state or situation, or the agent of an action.";
        let subj2 = "A postpositional particle referring to a subject under a certain state or situation, or the subject of an act.";
        let mut rows = vec![
            row(1, "마저", "Particles", Some(3), 5, "A postpositional particle that indicates the addition"),
            row(2, "가", "Particles", None, 9, subj2),
            row(3, "-게", "Connective endings", None, 50, "so that"),
            row(4, "이", "Particles", None, 8, subj),
            row(5, "야", "Particles", None, 7, "A postpositional particle used to emphasize the preceding word."),
            row(6, "아", "Particles", None, 7, "A postpositional particle used to address a friend, younger person, animal, etc."),
            row(7, "야", "Particles", None, 7, "A postpositional word used to address a friend, younger person, animal, etc."),
            row(8, "에", "Particles", None, 3, "A postpositional particle to indicate"),
            row(9, "-습니다", "Final endings", None, 3, "formal polite"),
            row(10, "-고", "Connective endings", None, 1, "and"),
            row(11, "-도", "Affixes", None, 1, "also"),
            row(12, "도", "Particles", None, 99, "also"),
            row(13, "와", "Particles", Some(1), 4, "A postpositional particle used to indicate that something is the subject of a comparison or subject of a standard."),
            row(14, "과", "Particles", None, 6, "A postpositional word used to indicate the subject of comparison or the object that serves as a basis."),
        ];
        sort_grammar(&mut rows);
        let pats: Vec<&str> = rows.iter().map(|r| r.pattern.as_str()).collect();
        // twins merged (first entry's id kept), unrelated homonym 야 (emphasis) stays separate
        assert!(pats.contains(&"이/가") && !pats.contains(&"가") && !pats.contains(&"이"));
        assert_eq!(rows.iter().find(|r| r.pattern == "이/가").unwrap().entry_id, 4);
        assert!(pats.contains(&"아/야") && pats.contains(&"야") && pats.contains(&"와/과"));
        assert_eq!(rows.iter().find(|r| r.pattern == "아/야").unwrap().entry_id, 6);
        assert_eq!(rows.iter().find(|r| r.pattern == "와/과").unwrap().level, Some(1));
        let pos = |p: &str| pats.iter().position(|x| *x == p).unwrap();
        // priority list order in Particles, then the rest by level / rank
        assert!(pos("이/가") < pos("에") && pos("에") < pos("도") && pos("도") < pos("와/과"));
        assert!(pos("와/과") < pos("마저"));
        // endings: -습니다 and -고 are priority entries, -게 too (after them); order by list position
        assert!(pos("-습니다") < pos("-게") || rows[pos("-습니다")].category != rows[pos("-게")].category);
        assert!(pos("-고") < pos("-게") || rows[pos("-고")].category != rows[pos("-게")].category);
        // the affix -도 is not a priority pattern
        assert_eq!(grammar_priority(&rows[pos("-도")]), PRIORITY.len());
    }

    #[test]
    fn priority_list_order_is_stable() {
        let g = |p: &str, c: &'static str| grammar_priority(&row(0, p, c, None, 1, "x"));
        assert!(g("-아요", "Final endings") < g("-겠-", "Final endings"));
        assert!(g("-고 싶다", "Expressions") < g("-고 있다", "Expressions"));
        assert!(g("-ㄴ", "Nominal/adnominal endings") < g("-ㄹ", "Nominal/adnominal endings"));
        assert_eq!(g("-이", "Final endings"), PRIORITY.len());
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    #[test]
    fn quality_gate_flags_degraded_builds() {
        let full = json!({"entries": 190000, "krdict": 56000, "wikt": 30000, "sentences": 40000, "hanja_chars": 8000});
        assert!(quality_gate(&full, None).is_empty());
        assert!(quality_gate(&full, Some(&json!({"entries": 436000}))).is_empty());
        let bad = quality_gate(&full, Some(&json!({"entries": 25000})));
        assert_eq!(bad.len(), 1);
        assert!(bad[0].contains("stdict.entries"));
        let degraded = json!({"entries": 60000, "krdict": 56000, "wikt": 0, "sentences": 0, "hanja_chars": 4999});
        let bad = quality_gate(&degraded, None);
        assert_eq!(bad.len(), 4, "{bad:?}"); // entries, wikt, sentences, hanja_chars
    }
}
