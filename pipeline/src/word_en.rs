//! English for 표준국어대사전 meanings that have no English anywhere in the app (`data/word_en.tsv`), so English search
//! can find them (선물 先物 "futures", 기사 技師 "engineer", 수지 樹脂 "resin" …). The rows came from a shortlist of
//! modern meanings (`qa/english_gap_shortlist.tsv`); the English is AI-written and a row ships ONLY once an independent
//! review set its `review` column to `approved…`. Approved rows become core pack entries of source `hanppuri`: the
//! English gloss carries `gloss_ai` (labelled AI in the app), the Korean definitions are stdict's, copied verbatim
//! (`ko_def_source` names the stdict entry).

use crate::common::{hw_norm, Entry};
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde_json::json;

pub use crate::grammar_extra::SOURCE;

#[derive(Debug, Clone, PartialEq)]
pub struct WordEn {
    pub headword: String,
    pub hanja: String,
    pub pos: String,
    /// stdict ext_id of the Korean source entry.
    pub src_id: String,
    pub en_gloss: String,
    /// stdict's definitions of the entry, verbatim, senses joined by " ‖ ".
    pub ko_def: String,
    pub review: String,
}

const COLS: usize = 7;

pub fn parse(tsv: &str) -> Result<Vec<WordEn>> {
    let mut out = Vec::new();
    for (i, line) in tsv.lines().enumerate().filter(|(_, l)| !l.starts_with('#') && !l.trim().is_empty()) {
        let f: Vec<&str> = line.split('\t').collect();
        if f[0] == "headword" {
            continue;
        }
        if f.len() != COLS {
            bail!("word_en.tsv line {}: {} columns, expected {COLS}", i + 1, f.len());
        }
        let src_id = f[3].strip_prefix("stdict:").with_context(|| format!("word_en.tsv line {}: src {:?}", i + 1, f[3]))?;
        out.push(WordEn {
            headword: f[0].into(),
            hanja: f[1].into(),
            pos: f[2].into(),
            src_id: src_id.into(),
            en_gloss: f[4].into(),
            ko_def: f[5].into(),
            review: f[6].into(),
        });
    }
    Ok(out)
}

pub fn builtin() -> Vec<WordEn> {
    parse(include_str!("../data/word_en.tsv")).expect("word_en.tsv parses")
}

/// Only an independent review may set this: the review column then starts with `approved`.
pub fn is_approved(w: &WordEn) -> bool {
    w.review.starts_with("approved")
}

/// Structural problems of one row (empty when it is fine).
pub fn problems(w: &WordEn) -> Vec<String> {
    let mut v = Vec::new();
    let hangul = crate::idiom_extra::has_hangul;
    if !hangul(&w.headword) || w.headword.contains(['(', ')']) {
        v.push("headword must be plain Hangul".into());
    }
    if w.src_id.is_empty() || !w.src_id.chars().all(|c| c.is_ascii_digit()) {
        v.push(format!("src stdict:{}", w.src_id));
    }
    if w.en_gloss.trim().is_empty() || hangul(&w.en_gloss) || w.en_gloss.chars().count() > 90 || w.en_gloss != w.en_gloss.trim() {
        v.push("en_gloss empty, not English, too long or untrimmed".into());
    }
    if !hangul(&w.ko_def) || w.ko_def != w.ko_def.trim() {
        v.push("ko_def must be Korean, trimmed".into());
    }
    if !(w.review == "draft" || w.review.starts_with("approved") || w.review.starts_with("rejected: ")) {
        v.push(format!("review {:?} (draft | approved... | rejected: reason)", w.review));
    }
    v
}

/// The krdict / kengdic entry with English the row duplicates, if any: same headword and the same hanja (any of the
/// entry's `/` variants, or one that the stdict hanja string starts with: stdict runs variants together, 妊娠姙娠).
/// The overlay is only for meanings that have no English, so an approved row for which this is `Some` fails the build.
pub fn english_duplicate(core: &Connection, w: &WordEn) -> Result<Option<String>> {
    use unicode_normalization::UnicodeNormalization;
    let nfc = |s: &str| -> String { s.nfc().collect() };
    let mine = nfc(&w.hanja);
    if mine.is_empty() {
        return Ok(None);
    }
    let mut q = core.prepare_cached("SELECT source, coalesce(ext_id,''), hanja FROM entries WHERE hw_norm = ?1 AND source IN ('krdict','kengdic','wikt') AND gloss IS NOT NULL AND gloss != '' AND hanja IS NOT NULL")?;
    let rows = q.query_map([hw_norm(&w.headword)], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))?;
    for r in rows {
        let (src, ext, h) = r?;
        if h.split('/').map(nfc).filter(|v| !v.is_empty()).any(|v| v == mine || mine.starts_with(&v) || v.starts_with(&mine)) {
            return Ok(Some(format!("{src}:{ext}")));
        }
    }
    Ok(None)
}

/// The row's Korean definitions must be the stdict entry's, verbatim and in order (the pack is the built stdict pack).
pub fn check_against_stdict(stdict: &Connection, w: &WordEn) -> Result<()> {
    let data: Option<String> = stdict.query_row("SELECT data FROM entries WHERE ext_id = ?1 LIMIT 1", [w.src_id.as_str()], |r| r.get(0)).ok();
    let Some(data) = data else { bail!("word_en.tsv: {} {}: stdict has no entry {}", w.headword, w.hanja, w.src_id) };
    let v: serde_json::Value = serde_json::from_str(&data)?;
    let pack: Vec<&str> = v["senses"].as_array().into_iter().flatten().filter_map(|s| s["ko_def"].as_str()).collect();
    let mine: Vec<&str> = w.ko_def.split(" ‖ ").collect();
    if pack.len() < mine.len() || pack[..mine.len()] != mine[..] {
        bail!("word_en.tsv: {} {} (stdict:{}): ko_def is not the stdict entry's definitions\n  file: {}\n  pack: {}", w.headword, w.hanja, w.src_id, w.ko_def, pack.join(" ‖ "));
    }
    Ok(())
}

pub fn ext_id(w: &WordEn) -> String {
    format!("word-stdict-{}", w.src_id)
}

/// The core pack entry of an approved row: the gloss on the first sense, every stdict definition as its own sense.
pub fn to_entry(w: &WordEn) -> Entry {
    let src = format!("stdict:{}", w.src_id);
    let defs: Vec<String> = w.ko_def.split(" ‖ ").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let senses: Vec<_> = defs
        .iter()
        .enumerate()
        .map(|(i, d)| if i == 0 { json!({"gloss": w.en_gloss, "gloss_ai": true, "ko_def": d, "ko_def_source": src}) } else { json!({"ko_def": d, "ko_def_source": src}) })
        .collect();
    Entry {
        source: SOURCE,
        lang: "en",
        ext_id: ext_id(w),
        headword: w.headword.clone(),
        homonym: None,
        hanja: (!w.hanja.is_empty()).then(|| w.hanja.clone()),
        pos: w.pos.clone(),
        ko_pos: String::new(),
        pron: None,
        level: None,
        kind: "word".into(),
        forms: Vec::new(),
        data: json!({"senses": senses, "en_overlay": "word_en"}),
        ko_defs: defs,
        pointer: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: &str = "headword\thanja\tpos\tsrc\ten_gloss\tko_def\treview\n선물\t先物\tnoun\tstdict:12345\tfutures (finance)\t장래의 일정한 시기에 현품을 넘겨준다는 조건으로 매매 계약을 하는 거래 종목.\tapproved (independent review)\n";

    #[test]
    fn parses_and_builds_an_ai_labelled_entry() {
        let w = &parse(ROW).unwrap()[0];
        assert!(problems(w).is_empty(), "{:?}", problems(w));
        assert!(is_approved(w));
        let e = to_entry(w);
        assert_eq!(e.source, "hanppuri");
        assert_eq!(e.hanja.as_deref(), Some("先物"));
        assert_eq!(e.data["senses"][0]["gloss"], "futures (finance)");
        assert_eq!(e.data["senses"][0]["gloss_ai"], true);
        assert_eq!(e.data["senses"][0]["ko_def_source"], "stdict:12345");
    }

    #[test]
    fn several_definitions_become_senses() {
        let w = WordEn { ko_def: "첫째 뜻. ‖ 둘째 뜻.".into(), ..parse(ROW).unwrap().remove(0) };
        let e = to_entry(&w);
        assert_eq!(e.data["senses"].as_array().unwrap().len(), 2);
        assert!(e.data["senses"][1].get("gloss").is_none());
    }

    #[test]
    fn shipped_rows_are_well_formed_and_unique() {
        let rows = builtin();
        let mut seen = std::collections::HashSet::new();
        for w in &rows {
            assert!(problems(w).is_empty(), "{} {}: {:?}", w.headword, w.hanja, problems(w));
            assert!(seen.insert(w.src_id.clone()), "stdict:{} twice", w.src_id);
        }
    }
}
