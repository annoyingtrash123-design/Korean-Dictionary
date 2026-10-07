//! Build core.sqlite / stdict.sqlite and the site-data bundle.

use crate::common::*;
use crate::{freq, kaikki, kengdic, krdict, opendict, pack, schema, stdict, tatoeba, unihan};
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

fn fts_text(e: &Entry) -> String {
    let mut parts: Vec<&str> = sense_strs(e, "gloss").collect();
    if parts.is_empty() {
        parts = sense_strs(e, "def").collect();
    }
    let mut seen = HashSet::new();
    parts.retain(|p| seen.insert(*p));
    parts.join(" ; ")
}

#[derive(Default)]
pub struct Counters {
    pub by_source: BTreeMap<&'static str, i64>,
    pub entries: i64,
    pub forms: i64,
}

fn insert_entry(conn: &Connection, e: &Entry, rank: i64, fts: bool, c: &mut Counters) -> Result<i64> {
    let hwn = hw_norm(&e.headword);
    let gloss = entry_gloss(e);
    let data = serde_json::to_string(&e.data)?;
    conn.prepare_cached(
        "INSERT INTO entries(headword,hw_norm,homonym,hanja,pos,pron,source,lang,level,rank,kind,gloss,data,ext_id) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )?
    .execute(params![e.headword, hwn, e.homonym, e.hanja, e.pos, e.pron, e.source, e.lang, e.level, rank, e.kind, gloss, data, e.ext_id])?;
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
        let t = fts_text(e);
        if !t.is_empty() {
            conn.prepare_cached("INSERT INTO entries_fts(rowid, en) VALUES(?,?)")?.execute(params![id, t])?;
        }
    }
    c.entries += 1;
    *c.by_source.entry(e.source).or_default() += 1;
    Ok(id)
}

struct GrammarRow {
    entry_id: i64,
    pattern: String,
    category: &'static str,
    level: Option<i64>,
    summary: Option<String>,
    key: String,
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

fn open_db(path: &Path) -> Result<Connection> {
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

fn finish_db(conn: Connection) -> Result<()> {
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

fn set_meta(conn: &Connection, k: &str, v: &str) -> Result<()> {
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
        "unihan" => ("Unicode License", "https://www.unicode.org/charts/unihan.html"),
        "freq" => ("CC BY-SA 4.0", "https://github.com/hermitdave/FrequencyWords"),
        _ => ("", ""),
    };
    json!({"license": lic, "url": url, "count": n})
}

// --- core pack -------------------------------------------------------------

pub fn build_core(src: &Sources, ranker: &Ranker, out: &Path, version: &str, built_at: &str) -> Result<PackResult> {
    let path = out.join("core.sqlite");
    let conn = open_db(&path)?;
    conn.execute_batch(schema::COMMON)?;
    conn.execute_batch(schema::CORE_ONLY)?;
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
    for p in &src.krdict {
        let t = Instant::now();
        let before = c.entries;
        krdict::parse_file(p, |e| {
            let rank = ranker.rank(&e);
            let id = insert_entry(&conn, &e, rank, true, &mut c)?;
            if let Some(cat) = grammar_category(&e) {
                let summary = senses(&e).iter().find_map(|s| s.get("def").or_else(|| s.get("ko_def")).and_then(Value::as_str)).map(|d| truncate(d, 200));
                grammar.push(GrammarRow { entry_id: id, pattern: e.headword.clone(), category: cat, level: e.level, summary, key: hw_norm(&e.headword) });
            }
            Ok(())
        })
        .with_context(|| format!("krdict {}", p.display()))?;
        log::info!("krdict {}: {} entries ({:.1}s)", p.file_name().unwrap().to_string_lossy(), c.entries - before, t.elapsed().as_secs_f32());
    }

    // wiktionary
    let mut wikt_sentences: Vec<(String, String)> = Vec::new();
    if let Some(p) = &src.kaikki {
        let t = Instant::now();
        let parsed = fs::File::open(p).map_err(anyhow::Error::from).and_then(|f| kaikki::parse(BufReader::with_capacity(1 << 20, f)));
        match parsed {
            Ok(k) => {
                for e in &k.entries {
                    let rank = ranker.rank(e);
                    insert_entry(&conn, e, rank, true, &mut c)?;
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
                log::info!("wikt: {} entries, {} redirects -> {} forms, {} example pairs ({:.1}s)", k.entries.len(), k.redirects.len(), added, wikt_sentences.len(), t.elapsed().as_secs_f32());
            }
            Err(e) => log::warn!("kaikki failed: {e:#}"),
        }
    }

    // kengdic entries
    if let Some(k) = &kd {
        for e in &k.entries {
            let rank = ranker.rank(e);
            insert_entry(&conn, e, rank, true, &mut c)?;
        }
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
    grammar.sort_by(|a, b| {
        (cat_order(a.category), a.level.unwrap_or(4), &a.key).cmp(&(cat_order(b.category), b.level.unwrap_or(4), &b.key))
    });
    for (i, g) in grammar.iter().enumerate() {
        conn.prepare_cached("INSERT INTO grammar(entry_id,pattern,category,level,summary_en,sort) VALUES(?,?,?,?,?,?)")?
            .execute(params![g.entry_id, g.pattern, g.category, g.level, g.summary, i as i64])?;
    }

    conn.execute_batch(schema::COMMON_INDEXES)?;

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
    chars.sort();
    for ch in &chars {
        let s = ch.to_string();
        let info = uni.get(ch);
        let readings = info.map(|i| i.readings.join(",")).filter(|r| !r.is_empty());
        conn.execute(
            "INSERT INTO hanja_chars(ch,readings,meaning_en,strokes,radical,word_count,radical_num) VALUES(?,?,?,?,?,?,?)",
            params![s, readings, info.and_then(|i| i.meaning_en.clone()), info.and_then(|i| i.strokes), info.and_then(|i| i.radical.clone()), wc.get(&s).copied().unwrap_or(0), info.and_then(|i| i.radical_num)],
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
    counts.insert("sentences".into(), q("SELECT COUNT(*) FROM sentences")?.into());
    counts.insert("sentences_tatoeba".into(), n_tat.into());
    counts.insert("sentences_wikt".into(), n_wikt_s.into());
    counts.insert("grammar".into(), q("SELECT COUNT(*) FROM grammar")?.into());
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
    conn.execute_batch(schema::COMMON_INDEXES)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
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
    conn.execute_batch(schema::COMMON_INDEXES)?;
    let q = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut counts = Map::new();
    counts.insert("entries".into(), q("SELECT COUNT(*) FROM entries")?.into());
    counts.insert("forms".into(), q("SELECT COUNT(*) FROM forms")?.into());
    counts.insert("hanja_words".into(), q("SELECT COUNT(*) FROM hanja_words")?.into());
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

// --- driver ------------------------------------------------------------------------

pub struct BuildOpts {
    pub out: PathBuf,
    pub sources: Sources,
    pub chunk_bytes: u64,
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
    if o.sources.stdict.is_empty() {
        log::warn!("no stdict files: skipping the stdict pack");
        remove_stale(&site, "stdict.sqlite.gz.")?;
    } else {
        let st = build_stdict(&o.sources, &ranker, &o.out, &version, &built_at)?;
        log::info!("stdict.sqlite built: {}", st.counts);
        packs.push(pack::compress_pack("stdict", false, &st.path, &site, &st.counts, o.chunk_bytes)?);
        dedupe = st.keys;
    }
    if o.sources.opendict.is_empty() {
        remove_stale(&site, "opendict.sqlite.gz.")?;
    } else {
        let od = build_opendict(&o.sources, &ranker, &o.out, &version, &built_at, &dedupe)?;
        log::info!("opendict.sqlite built: {}", od.counts);
        packs.push(pack::compress_pack("opendict", false, &od.path, &site, &od.counts, o.chunk_bytes)?);
    }

    let manifest = json!({"version": version, "packs": packs});
    fs::write(site.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    fs::write(site.join("ATTRIBUTION.md"), ATTRIBUTION)?;
    log::info!("build finished in {:.1}s", t0.elapsed().as_secs_f32());
    Ok(manifest)
}
