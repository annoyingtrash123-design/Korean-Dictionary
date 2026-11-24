//! "Chinese & English" block of a 성어 page (core pack table `seongeo_zh`).
//!
//! Two independent halves, one row per 성어 hanja:
//!  * the Chinese chengyu: the CC-CEDICT line whose traditional headword equals the 성어's four hanja EXACTLY
//!    (deterministic dictionary data, no AI; CC BY-SA 4.0, attributed in `ATTRIBUTION.md`). Only filled when the
//!    CEDICT source is present at build time (CI fetches it); without it there are no Chinese columns and no error.
//!  * an established English idiom with the same meaning (`data/seongeo_en_equiv.tsv`, AI-proposed). Only rows
//!    whose `review` column says `approved...` ship; the app labels the text as AI-written (`en_ai = 1`).
//!
//! The roster of 성어 is the equivalents file itself (hanja + headword for every 성어 of the study lists
//! `seongeo300` and `seongeo_learner`, an empty equivalent where none exists); a test keeps it in step with
//! `data/study_seongeo.tsv`. The entry page looks the row up by the entry's hanja (marks like ▽ removed).

use crate::cedict;
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Most CEDICT definitions kept per chengyu, and the longest one.
const MAX_DEFS: usize = 4;
const MAX_DEF_CHARS: usize = 240;

#[derive(Debug, Clone, PartialEq)]
pub struct Equiv {
    pub hanja: String,
    pub headword: String,
    /// Empty = no real English equivalent.
    pub equivalent: String,
    pub note: String,
    pub review: String,
}

pub fn parse(tsv: &str) -> Result<Vec<Equiv>> {
    let mut lines = tsv.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty());
    let header: Vec<&str> = lines.next().context("seongeo_en_equiv.tsv has no header")?.split('\t').collect();
    if header != ["hanja", "headword", "equivalent", "note", "review"] {
        bail!("seongeo_en_equiv.tsv header: {header:?}");
    }
    let mut out = Vec::new();
    for (i, line) in lines.enumerate() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 5 {
            bail!("seongeo_en_equiv.tsv row {}: {} columns, expected 5", i + 1, f.len());
        }
        out.push(Equiv {
            hanja: f[0].trim().to_string(),
            headword: f[1].trim().to_string(),
            equivalent: f[2].trim().to_string(),
            note: f[3].trim().to_string(),
            review: f[4].trim().to_string(),
        });
    }
    Ok(out)
}

pub fn builtin() -> Vec<Equiv> {
    parse(include_str!("../data/seongeo_en_equiv.tsv")).expect("seongeo_en_equiv.tsv parses")
}

impl Equiv {
    /// Ships only when the reviewer approved it and there is an equivalent to show.
    pub fn ships(&self) -> bool {
        self.review.starts_with("approved") && !self.equivalent.is_empty()
    }
}

/// A CC-CEDICT entry matched to a 성어.
#[derive(Debug, Clone, PartialEq)]
pub struct Chengyu {
    pub trad: String,
    pub simp: String,
    /// Tone-marked ("guò yóu bù jí").
    pub pinyin: String,
    pub defs: Vec<String>,
}

fn is_variant_only(g: &str) -> bool {
    let g = g.to_ascii_lowercase();
    g.starts_with("variant of") || g.starts_with("old variant of") || g.starts_with("also written") || g.starts_with("see ")
}

fn chengyu_of(l: &cedict::Line) -> Chengyu {
    let defs: Vec<String> = l
        .glosses
        .iter()
        .filter(|g| !g.starts_with("CL:"))
        .take(MAX_DEFS)
        .map(|g| crate::common::truncate(g, MAX_DEF_CHARS))
        .collect();
    Chengyu { trad: l.trad.clone(), simp: l.simp.clone(), pinyin: cedict::pinyin_marked(&l.pinyin), defs }
}

/// CEDICT lines whose traditional headword is one of `hanja`, EXACTLY (no partial or simplified-form match).
/// When several lines share a headword the first that is not only a "variant of ..." pointer wins.
pub fn match_lines(hanja: &HashSet<String>, lines: impl IntoIterator<Item = cedict::Line>) -> HashMap<String, Chengyu> {
    let mut out: HashMap<String, (Chengyu, bool)> = HashMap::new();
    for l in lines {
        if !hanja.contains(&l.trad) {
            continue;
        }
        let pointer = l.glosses.iter().all(|g| is_variant_only(g));
        if out.get(&l.trad).is_some_and(|(_, p)| !*p || pointer) {
            continue;
        }
        out.insert(l.trad.clone(), (chengyu_of(&l), pointer));
    }
    out.into_iter().map(|(k, (c, _))| (k, c)).collect()
}

pub fn match_cedict_text(hanja: &HashSet<String>, text: &str) -> HashMap<String, Chengyu> {
    match_lines(hanja, text.lines().filter_map(cedict::parse_line))
}

/// The matches from the CEDICT source file (zip or plain text).
pub fn match_cedict_file(hanja: &HashSet<String>, path: &Path) -> Result<HashMap<String, Chengyu>> {
    let mut lines = Vec::new();
    cedict::read_lines(path, |l| {
        if hanja.contains(&l.trad) {
            lines.push(l);
        }
        Ok(())
    })?;
    Ok(match_lines(hanja, lines))
}

#[derive(Debug, Default, PartialEq)]
pub struct Stats {
    /// Rows written.
    pub rows: usize,
    /// Rows with a CC-CEDICT chengyu.
    pub zh: usize,
    /// Rows with a shipped English equivalent.
    pub en: usize,
}

/// Create `seongeo_zh` in the (finished) core pack. `cedict`: the CEDICT source when it is present.
pub fn add_table(core: &Path, cedict: Option<&Path>) -> Result<Stats> {
    add_table_with(core, &builtin(), |h| match cedict {
        Some(p) => match_cedict_file(h, p),
        None => Ok(HashMap::new()),
    })
}

pub fn add_table_with(core: &Path, equivs: &[Equiv], zh: impl FnOnce(&HashSet<String>) -> Result<HashMap<String, Chengyu>>) -> Result<Stats> {
    let hanja: HashSet<String> = equivs.iter().map(|e| e.hanja.clone()).collect();
    let zh = zh(&hanja)?;
    let conn = Connection::open(core)?;
    conn.execute_batch(
        "CREATE TABLE seongeo_zh (
           hanja TEXT PRIMARY KEY, headword TEXT NOT NULL,
           zh_trad TEXT, zh_simp TEXT, zh_pinyin TEXT, zh_en TEXT,
           en_equiv TEXT, en_note TEXT, en_ai INTEGER NOT NULL DEFAULT 0
         ) WITHOUT ROWID;",
    )?;
    let mut stats = Stats::default();
    conn.execute_batch("BEGIN")?;
    for e in equivs {
        let c = zh.get(&e.hanja);
        let en = e.ships();
        if c.is_none() && !en {
            continue;
        }
        conn.prepare_cached("INSERT INTO seongeo_zh VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)")?.execute(params![
            e.hanja,
            e.headword,
            c.map(|c| c.trad.clone()),
            c.map(|c| c.simp.clone()),
            c.map(|c| c.pinyin.clone()),
            c.map(|c| json!(c.defs).to_string()),
            en.then(|| e.equivalent.clone()),
            (en && !e.note.is_empty()).then(|| e.note.clone()),
            en as i64,
        ])?;
        stats.rows += 1;
        stats.zh += c.is_some() as usize;
        stats.en += en as usize;
    }
    conn.execute_batch("COMMIT")?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
# CC-CEDICT fixture (a few real lines)
過猶不及 过犹不及 [guo4 you2 bu4 ji2] /to go too far is as bad as not going far enough (idiom, from Analects)/
塞翁失馬 塞翁失马 [sai4 weng1 shi1 ma3] /lit. when the old man lost his horse, it turned out to be a blessing (idiom)/fig. a blessing in disguise/
他山之石 他山之石 [ta1 shan1 zhi1 shi2] /lit. stones from other hills (may serve to polish jade) (idiom)/fig. advice or criticism from others can help one to correct one's own shortcomings/
一石二鳥 一石二鸟 [yi1 shi2 er4 niao3] /to kill two birds with one stone (idiom)/
一石二鳥 一石二鸟 [yi1 shi2 er4 niao3] /variant of 一箭雙鵰|一箭双雕[yi1 jian4 shuang1 diao1]/
過猶 过犹 [guo4 you2] /not a chengyu/
";

    fn set(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn exact_traditional_match_gives_simplified_pinyin_and_english() {
        let m = match_cedict_text(&set(&["過猶不及", "他山之石"]), FIXTURE);
        assert_eq!(m.len(), 2);
        let c = &m["過猶不及"];
        assert_eq!((c.trad.as_str(), c.simp.as_str(), c.pinyin.as_str()), ("過猶不及", "过犹不及", "guò yóu bù jí"));
        assert_eq!(c.defs, vec!["to go too far is as bad as not going far enough (idiom, from Analects)"]);
        let t = &m["他山之石"];
        assert_eq!(t.simp, "他山之石");
        assert_eq!(t.pinyin, "tā shān zhī shí");
        assert_eq!(t.defs.len(), 2);
    }

    #[test]
    fn only_exact_four_hanja_matches_count() {
        // 塞翁之馬 (the Korean 새옹지마 spelling) differs from CEDICT's 塞翁失馬: no match, nothing guessed
        let m = match_cedict_text(&set(&["塞翁之馬", "過猶"]), FIXTURE);
        assert!(!m.contains_key("塞翁之馬"));
        assert_eq!(m["過猶"].defs, vec!["not a chengyu"]);
        // a simplified spelling does not match the traditional headword
        assert!(match_cedict_text(&set(&["过犹不及"]), FIXTURE).is_empty());
        assert_eq!(match_cedict_text(&set(&["塞翁失馬"]), FIXTURE)["塞翁失馬"].defs[1], "fig. a blessing in disguise");
    }

    #[test]
    fn a_pointer_line_does_not_beat_a_real_definition() {
        let m = match_cedict_text(&set(&["一石二鳥"]), FIXTURE);
        assert_eq!(m["一石二鳥"].defs, vec!["to kill two birds with one stone (idiom)"]);
        let rev = "一石二鳥 一石二鸟 [yi1 shi2 er4 niao3] /variant of 一箭雙鵰/\n一石二鳥 一石二鸟 [yi1 shi2 er4 niao3] /to kill two birds with one stone/\n";
        assert_eq!(match_cedict_text(&set(&["一石二鳥"]), rev)["一石二鳥"].defs, vec!["to kill two birds with one stone"]);
    }

    fn eq(h: &str, e: &str, review: &str) -> Equiv {
        Equiv { hanja: h.into(), headword: "x".into(), equivalent: e.into(), note: "n".into(), review: review.into() }
    }

    fn rows(dir: &Path, equivs: &[Equiv], with_zh: bool) -> (Stats, Vec<(String, Option<String>, Option<String>, i64)>) {
        let db = dir.join("core.sqlite");
        let _ = std::fs::remove_file(&db);
        Connection::open(&db).unwrap().execute_batch("CREATE TABLE entries (id INTEGER)").unwrap();
        let stats = add_table_with(&db, equivs, |h| Ok(if with_zh { match_cedict_text(h, FIXTURE) } else { HashMap::new() })).unwrap();
        let c = Connection::open(&db).unwrap();
        let mut q = c.prepare("SELECT hanja, zh_pinyin, en_equiv, en_ai FROM seongeo_zh ORDER BY hanja").unwrap();
        let v = q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap().map(|r| r.unwrap()).collect();
        (stats, v)
    }

    #[test]
    fn table_has_chinese_and_only_approved_english_with_the_ai_flag() {
        let dir = tempfile::tempdir().unwrap();
        let equivs = [eq("過猶不及", "too much of a good thing", "approved"), eq("他山之石", "learn from others' mistakes", "draft"), eq("無此詞語", "x", "approved")];
        let (st, v) = rows(dir.path(), &equivs, true);
        assert_eq!(st, Stats { rows: 3, zh: 2, en: 2 });
        assert_eq!(v.iter().find(|r| r.0 == "過猶不及").unwrap().2.as_deref(), Some("too much of a good thing"));
        assert_eq!(v.iter().find(|r| r.0 == "過猶不及").unwrap().3, 1);
        // a draft's English never ships, but the Chinese half still does
        let t = v.iter().find(|r| r.0 == "他山之石").unwrap();
        assert_eq!((t.1.as_deref(), t.2.as_deref(), t.3), (Some("tā shān zhī shí"), None, 0));
    }

    #[test]
    fn absent_cedict_gives_no_chinese_and_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let equivs = [eq("過猶不及", "too much of a good thing", "approved"), eq("他山之石", "x", "draft"), eq("塞翁失馬", "", "approved: no equivalent")];
        let (st, v) = rows(dir.path(), &equivs, false);
        assert_eq!(st, Stats { rows: 1, zh: 0, en: 1 });
        assert!(v[0].1.is_none());
        // and the file-based entry point with no source
        let db = dir.path().join("c2.sqlite");
        Connection::open(&db).unwrap().execute_batch("CREATE TABLE entries (id INTEGER)").unwrap();
        assert_eq!(add_table(&db, None).unwrap().zh, 0);
    }

    #[test]
    fn the_file_reader_matches_a_plain_u8_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("cedict_ts.u8");
        std::fs::write(&p, FIXTURE).unwrap();
        let m = match_cedict_file(&set(&["過猶不及"]), &p).unwrap();
        assert_eq!(m["過猶不及"].simp, "过犹不及");
    }

    // --- the real equivalents file ---

    #[test]
    fn roster_is_the_study_lists_and_rows_are_well_formed() {
        let eqs = builtin();
        let study: HashSet<String> = crate::study_idioms::builtin().into_iter().filter(|i| i.list == "seongeo300" || i.list == "seongeo_learner").map(|i| i.text).collect();
        let heads: HashSet<String> = eqs.iter().map(|e| e.headword.clone()).collect();
        assert_eq!(heads, study, "seongeo_en_equiv.tsv must list exactly the 성어 of seongeo300 + seongeo_learner");
        assert_eq!(eqs.len(), study.len(), "one row per 성어");
        let mut seen = HashSet::new();
        for e in &eqs {
            assert!(seen.insert(&e.hanja), "duplicate hanja {}", e.hanja);
            assert_eq!(e.hanja.chars().count(), 4, "{}: hanja must be four characters", e.headword);
            assert!(e.hanja.chars().all(crate::common::is_cjk), "{}: hanja {:?}", e.headword, e.hanja);
            assert!(e.review == "draft" || e.review.starts_with("approved") || e.review.starts_with("rejected: "), "{}: review {:?}", e.headword, e.review);
            assert!(!crate::idiom_extra::has_hangul(&e.equivalent), "{}: the equivalent is English", e.headword);
            assert!(e.equivalent.chars().count() <= 70, "{}: equivalent too long", e.headword);
            assert!(!e.equivalent.contains('\t') && !e.note.contains('\t'));
        }
        // not forced: a good share of the 성어 have no real equivalent
        let with = eqs.iter().filter(|e| !e.equivalent.is_empty()).count();
        assert!(with >= 50 && with * 100 <= eqs.len() * 60, "{with} of {} have an equivalent", eqs.len());
    }
}
