//! English -> Korean from Wiktionary translation tables (`en_ko` / `en_ko_words`, core pack,
//! optional; see docs/SCOPE.md). For an English query: the query term's own translations,
//! grouped by part of speech and sense, and multi-word phrases that start with or contain it.

use super::{PackDb, SearchResult};
use crate::sql::{Result, Val};
use serde::Serialize;
use std::collections::HashMap;

/// Phrases returned per query.
pub const PHRASE_LIMIT: usize = 30;
/// Candidate phrase terms read per source (prefix range / word index) before ranking.
const PHRASE_SCAN: usize = 2000;
/// Korean words shown per phrase.
const PHRASE_KO: usize = 6;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EnKoWord {
    pub ko: String,
    pub roman: Option<String>,
}

/// Korean translations of one (part of speech, sense) of the query term.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EnKoSense {
    /// English term as written in Wiktionary (capitals kept, e.g. "Korea").
    pub term: String,
    pub pos: String,
    pub sense: Option<String>,
    pub words: Vec<EnKoWord>,
}

/// A multi-word English expression containing the query, with its Korean.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EnKoPhrase {
    pub term: String,
    pub pos: String,
    /// First sense label, if any.
    pub sense: Option<String>,
    pub words: Vec<EnKoWord>,
}

/// Lookup form of an English query: lower-cased, whitespace collapsed, leading "to " dropped.
pub fn en_term(q: &str) -> String {
    let ql = q.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    match ql.strip_prefix("to ") {
        Some(rest) if !rest.trim().is_empty() => rest.trim().to_string(),
        _ => ql,
    }
}

fn trim_tok(t: &str) -> &str {
    t.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
}

/// `term` holds the words of `q` (already split) as a run that does not start the term.
fn contains_later(term: &str, q: &[&str]) -> bool {
    let toks: Vec<&str> = term.split(' ').map(trim_tok).collect();
    q.len() < toks.len() && (1..=toks.len() - q.len()).any(|i| toks[i..i + q.len()] == *q)
}

/// The query term's translations: rows in rank order, grouped by (pos, sense); groups of one
/// part of speech stay together, in order of first appearance.
pub fn translations(p: &PackDb, t: &str) -> Result<Vec<EnKoSense>> {
    let rows = p.conn.query(
        "SELECT coalesce(term, term_norm), pos, sense, ko, roman FROM en_ko WHERE term_norm = ?1 ORDER BY rank",
        &[t.into()],
    )?;
    let mut out: Vec<EnKoSense> = Vec::new();
    for r in rows {
        let (term, pos, sense) = (r.string(0), r.string(1), r.text(2));
        let w = EnKoWord { ko: r.string(3), roman: r.text(4) };
        match out.iter_mut().find(|g| g.pos == pos && g.sense == sense) {
            Some(g) => {
                if !g.words.iter().any(|x| x.ko == w.ko) {
                    g.words.push(w)
                }
            }
            None => out.push(EnKoSense { term, pos, sense, words: vec![w] }),
        }
    }
    let first: HashMap<String, usize> =
        out.iter().enumerate().rev().map(|(i, g)| (g.pos.clone(), i)).collect();
    out.sort_by_key(|g| first[&g.pos]); // stable: senses keep their order within a pos
    Ok(out)
}

/// Up to [`PHRASE_LIMIT`] phrases that start with "`t` " or contain `t` as a later word run:
/// those starting with the query first, then shortest first, then alphabetical.
pub fn phrases(p: &PackDb, t: &str) -> Result<Vec<EnKoPhrase>> {
    let mut cands: Vec<(bool, usize, String)> = Vec::new();
    let lo = format!("{t} ");
    let hi = format!("{t} {}", char::MAX);
    for r in p.conn.query(
        &format!("SELECT DISTINCT term_norm FROM en_ko WHERE term_norm > ?1 AND term_norm < ?2 LIMIT {PHRASE_SCAN}"),
        &[lo.as_str().into(), hi.as_str().into()],
    )? {
        let term = r.string(0);
        cands.push((false, term.chars().count(), term));
    }
    let qtoks: Vec<&str> = t.split(' ').map(trim_tok).collect();
    if p.caps.en_ko_words && !qtoks.is_empty() && !qtoks[0].is_empty() {
        for r in p.conn.query(
            &format!("SELECT term_norm FROM en_ko_words WHERE word = ?1 LIMIT {PHRASE_SCAN}"),
            &[qtoks[0].into()],
        )? {
            let term = r.string(0);
            if !term.starts_with(&lo) && contains_later(&term, &qtoks) {
                cands.push((true, term.chars().count(), term));
            }
        }
    }
    cands.sort();
    cands.dedup_by(|a, b| a.2 == b.2);
    cands.truncate(PHRASE_LIMIT);
    if cands.is_empty() {
        return Ok(vec![]);
    }
    let ph = (1..=cands.len()).map(|i| format!("?{i}")).collect::<Vec<_>>().join(",");
    let params: Vec<Val> = cands.iter().map(|c| c.2.as_str().into()).collect();
    let rows = p.conn.query(
        &format!(
            "SELECT term_norm, coalesce(term, term_norm), pos, sense, ko, roman FROM en_ko \
             WHERE term_norm IN ({ph}) ORDER BY term_norm, rank"
        ),
        &params,
    )?;
    let mut by: HashMap<String, EnKoPhrase> = HashMap::new();
    for r in rows {
        let ph = by.entry(r.string(0)).or_insert_with(|| EnKoPhrase { term: r.string(1), pos: r.string(2), sense: r.text(3), words: vec![] });
        let w = EnKoWord { ko: r.string(4), roman: r.text(5) };
        if ph.words.len() < PHRASE_KO && !ph.words.iter().any(|x| x.ko == w.ko) {
            ph.words.push(w);
        }
    }
    Ok(cands.iter().filter_map(|c| by.remove(&c.2)).collect())
}

/// Fill `translations` / `phrases` of an English search result from the first pack that has
/// `en_ko` (packs without it are skipped; nothing is set when nothing matches).
pub fn attach(packs: &[&PackDb], q: &str, r: &mut SearchResult) -> Result<()> {
    let t = en_term(q);
    if t.is_empty() || !t.chars().any(|c| c.is_alphabetic()) {
        return Ok(());
    }
    for p in packs.iter().filter(|p| p.caps.en_ko) {
        let tr = translations(p, &t)?;
        let ph = phrases(p, &t)?;
        if !tr.is_empty() && r.translations.is_none() {
            r.translations = Some(tr);
        }
        if !ph.is_empty() && r.phrases.is_none() {
            r.phrases = Some(ph);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{search, PackDb};
    use crate::sql::Conn;

    const MINI: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER,
  hanja TEXT, pos TEXT, pron TEXT, source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER,
  rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);
CREATE TABLE gloss_terms (term TEXT NOT NULL, tier INTEGER NOT NULL, score INTEGER NOT NULL, entry_id INTEGER NOT NULL,
  PRIMARY KEY (term, score, entry_id)) WITHOUT ROWID;
INSERT INTO entries VALUES (1, '보고', '보고', NULL, '報告', 'noun', NULL, 'krdict', 'en', 1, 10, 'word', 'report', '{"senses":[]}');
INSERT INTO gloss_terms VALUES ('report', 0, 10, 1);
"#;

    const EN_KO: &str = r#"
CREATE TABLE en_ko (term_norm TEXT NOT NULL, term TEXT, pos TEXT NOT NULL, sense TEXT, ko TEXT NOT NULL, roman TEXT, rank INTEGER NOT NULL);
CREATE TABLE en_ko_words (word TEXT NOT NULL, term_norm TEXT NOT NULL, PRIMARY KEY (word, term_norm)) WITHOUT ROWID;
CREATE INDEX en_ko_term ON en_ko(term_norm, rank);
INSERT INTO en_ko VALUES
  ('report', NULL, 'noun', 'account of events', '보고', 'bogo', 0),
  ('report', NULL, 'noun', 'account of events', '보고서', 'bogoseo', 1),
  ('report', NULL, 'verb', 'to relay information', '보고하다', 'bogohada', 2),
  ('report', NULL, 'verb', 'to relay information', '알리다', 'allida', 3),
  ('report', NULL, 'noun', 'loud noise', '폭음', NULL, 4),
  ('report card', NULL, 'noun', NULL, '성적표', NULL, 0),
  ('report card', NULL, 'noun', NULL, '통지표', NULL, 1),
  ('report sick', NULL, 'verb', 'call in sick', '병가를 내다', NULL, 0),
  ('file a report', NULL, 'verb', NULL, '신고하다', NULL, 0),
  ('weather report', NULL, 'noun', NULL, '일기 예보', NULL, 0),
  ('reporter', NULL, 'noun', NULL, '기자', NULL, 0),
  ('korea', 'Korea', 'proper noun', 'peninsula', '한국', 'han''guk', 0);
INSERT INTO en_ko_words VALUES ('report', 'file a report'), ('report', 'weather report'), ('card', 'report card'), ('sick', 'report sick');
"#;

    fn pack(with_en_ko: bool) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(MINI).unwrap();
        if with_en_ko {
            c.exec(EN_KO).unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn english_search_has_translations_and_phrases() {
        let p = pack(true);
        assert!(p.caps.en_ko && p.caps.en_ko_words);
        let r = search(&[&p], "Report", Some(20)).unwrap();
        // existing rows unchanged
        assert_eq!(r.rows.iter().map(|x| x.headword.as_str()).collect::<Vec<_>>(), ["보고"]);
        let tr = r.translations.unwrap();
        let got: Vec<(&str, Option<&str>, Vec<&str>)> =
            tr.iter().map(|g| (g.pos.as_str(), g.sense.as_deref(), g.words.iter().map(|w| w.ko.as_str()).collect())).collect();
        assert_eq!(
            got,
            vec![
                ("noun", Some("account of events"), vec!["보고", "보고서"]),
                ("noun", Some("loud noise"), vec!["폭음"]),
                ("verb", Some("to relay information"), vec!["보고하다", "알리다"]),
            ]
        );
        assert_eq!(tr[0].words[0].roman.as_deref(), Some("bogo"));
        // phrases: starting with the query first (shortest first), then containing it; not "reporter"
        let ph = r.phrases.unwrap();
        let terms: Vec<&str> = ph.iter().map(|x| x.term.as_str()).collect();
        assert_eq!(terms, ["report card", "report sick", "file a report", "weather report"]);
        assert_eq!(ph[0].words.iter().map(|w| w.ko.as_str()).collect::<Vec<_>>(), ["성적표", "통지표"]);
        assert_eq!(ph[1].sense.as_deref(), Some("call in sick"));
    }

    #[test]
    fn verb_query_and_display_form() {
        let p = pack(true);
        let r = search(&[&p], "to report", None).unwrap();
        assert_eq!(r.translations.as_ref().map(|t| t.len()), Some(3));
        let r = search(&[&p], "korea", None).unwrap();
        assert_eq!(r.translations.unwrap()[0].term, "Korea");
        // multi-word query: its own translations, and longer phrases containing it
        let r = search(&[&p], "report card", None).unwrap();
        assert_eq!(r.translations.unwrap()[0].words.len(), 2);
        assert!(r.phrases.is_none());
        // nothing found: fields absent (not empty arrays)
        let r = search(&[&p], "zebra", None).unwrap();
        assert!(r.translations.is_none() && r.phrases.is_none());
        let j = serde_json::to_value(&r).unwrap();
        assert!(j.get("translations").is_none() && j.get("phrases").is_none());
    }

    #[test]
    fn hangul_search_and_packs_without_table() {
        let p = pack(true);
        let r = search(&[&p], "보고", None).unwrap();
        assert!(r.translations.is_none() && r.phrases.is_none());
        let old = pack(false);
        assert!(!old.caps.en_ko);
        let r = search(&[&old], "report", None).unwrap();
        assert_eq!(r.rows.len(), 1);
        assert!(r.translations.is_none() && r.phrases.is_none());
    }

    #[test]
    fn helpers() {
        assert_eq!(en_term("  To   Report "), "report");
        assert_eq!(en_term("to"), "to");
        assert!(contains_later("file a report", &["report"]));
        assert!(contains_later("make a report, quickly", &["report"]));
        assert!(!contains_later("report card", &["report"]));
        assert!(contains_later("hand in a report card", &["report", "card"]));
    }
}
