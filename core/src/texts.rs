//! Reader library (`texts` pack): the text list and one text with its paragraphs.
//! Contract: docs/READER.md, "`texts` pack + engine API". The pack has no `entries` table, so it
//! is never part of dictionary searches; the engine hands it to these functions separately.

use crate::search::PackDb;
use crate::sql::{Result, Row};
use serde::Serialize;
use serde_json::Value;

/// One row of the library list (`listTexts()`), sorted by the pack's `sort` column.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TextSummary {
    pub id: String,
    pub shelf: String,
    pub period: Option<String>,
    pub year: Option<i64>,
    pub script: String,
    pub level: Option<String>,
    pub chars: i64,
    pub title_ko: String,
    pub title_en: String,
    pub author_ko: String,
    pub author_en: String,
    pub date: String,
    pub themes: Vec<String>,
    /// True when the text is an excerpt of a longer work.
    pub excerpt: bool,
    /// Blurb from the card (empty for texts without enrichment).
    pub summary_ko: String,
    pub summary_en: String,
    pub labels: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TextParagraph {
    pub n: i64,
    pub orig: String,
    pub modern: Option<String>,
    pub reading: Option<String>,
    pub en: Option<String>,
}

/// `getText(id)`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TextDoc {
    pub id: String,
    pub meta: Value,
    pub card: Value,
    pub notes: Value,
    pub provenance: Value,
    pub labels: Value,
    pub review: Value,
    pub vocab: Value,
    pub questions: Value,
    pub paragraphs: Vec<TextParagraph>,
}

fn json(r: &Row, i: usize) -> Value {
    r.text(i).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null)
}

fn str_of(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// The library list; empty when the pack is not installed.
pub fn list_texts(pack: Option<&PackDb>) -> Result<Vec<TextSummary>> {
    let Some(p) = pack.filter(|p| p.caps.texts) else { return Ok(vec![]) };
    let rows = p.conn.query("SELECT id, shelf, period, year, script, level, chars, meta, card, labels FROM texts ORDER BY sort, id", &[])?;
    Ok(rows
        .iter()
        .map(|r| {
            let (meta, card) = (json(r, 7), json(r, 8));
            TextSummary {
                id: r.string(0),
                shelf: r.string(1),
                period: r.text(2),
                year: r.int(3),
                script: r.string(4),
                level: r.text(5),
                chars: r.int(6).unwrap_or(0),
                title_ko: str_of(&meta, "title_ko"),
                title_en: str_of(&meta, "title_en"),
                author_ko: str_of(&meta, "author_ko"),
                author_en: str_of(&meta, "author_en"),
                date: str_of(&meta, "date"),
                themes: meta.get("themes").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect()).unwrap_or_default(),
                excerpt: meta.get("excerpt").and_then(Value::as_bool).unwrap_or(false),
                summary_ko: str_of(&card, "summary_ko"),
                summary_en: str_of(&card, "summary_en"),
                labels: json(r, 9),
            }
        })
        .collect())
}

/// One text with all paragraphs; `None` when the pack or the id is missing.
pub fn get_text(pack: Option<&PackDb>, id: &str) -> Result<Option<TextDoc>> {
    let Some(p) = pack.filter(|p| p.caps.texts) else { return Ok(None) };
    let rows = p.conn.query("SELECT id, meta, card, notes, provenance, labels, review, vocab, questions FROM texts WHERE id = ?1", &[id.into()])?;
    let Some(r) = rows.first() else { return Ok(None) };
    let paragraphs = p
        .conn
        .query("SELECT n, orig, modern, reading, en FROM paragraphs WHERE text_id = ?1 ORDER BY n", &[id.into()])?
        .iter()
        .map(|r| TextParagraph { n: r.int(0).unwrap_or(0), orig: r.string(1), modern: r.text(2), reading: r.text(3), en: r.text(4) })
        .collect();
    Ok(Some(TextDoc {
        id: r.string(0),
        meta: json(r, 1),
        card: json(r, 2),
        notes: json(r, 3),
        provenance: json(r, 4),
        labels: json(r, 5),
        review: json(r, 6),
        vocab: json(r, 7),
        questions: json(r, 8),
        paragraphs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;

    const SCHEMA: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE texts (id TEXT PRIMARY KEY, sort INTEGER NOT NULL, shelf TEXT NOT NULL, period TEXT, year INTEGER, script TEXT NOT NULL,
  level TEXT, chars INTEGER NOT NULL, meta TEXT NOT NULL, card TEXT NOT NULL, notes TEXT NOT NULL, provenance TEXT NOT NULL,
  labels TEXT NOT NULL, review TEXT NOT NULL, vocab TEXT, questions TEXT);
CREATE TABLE paragraphs (text_id TEXT NOT NULL, n INTEGER NOT NULL, orig TEXT NOT NULL, modern TEXT, reading TEXT, en TEXT,
  PRIMARY KEY (text_id, n)) WITHOUT ROWID;
INSERT INTO meta VALUES ('pack','texts'), ('version','2025.01.test');
INSERT INTO texts VALUES
 ('b-text', 2, 'verse', 'goryeo', 1200, 'hangul', NULL, 7,
  '{"title_ko":"청산별곡","title_en":"Song of the Green Mountain","author_ko":"작자 미상","author_en":"Anonymous","date":"Goryeo","themes":["ancient-goryeo"],"excerpt":false}',
  '{"summary_ko":"","summary_en":"","level":null}', '{"ko":"","en":"n"}', '{"source":"wikisource-ko","revision_id":7}', '{"text":"original"}',
  '{"status":"original-only"}', NULL, NULL),
 ('a-text', 1, 'graded', 'ancient', -2333, 'hangul', 'TOPIK 2', 12,
  '{"title_ko":"단군","title_en":"Dangun","author_ko":"AI","author_en":"AI","date":"2026","themes":["ancient-goryeo","joseon"],"excerpt":true}',
  '{"summary_ko":"요약","summary_en":"Summary","level":"TOPIK 2"}', '{"ko":"k","en":"e"}', '{"source":"original"}', '{"text":"ai","translation":"ai"}',
  '{"status":"approved"}', '[{"word":"곰","gloss_en":"bear","level":1}]', '[{"q_ko":"?","q_en":"?","answer_en":"!"}]');
INSERT INTO paragraphs VALUES ('a-text', 1, '둘째', '둘째m', NULL, 'Second'), ('a-text', 0, '첫째', NULL, NULL, 'First'), ('b-text', 0, '살어리', NULL, NULL, NULL);
"#;

    fn pack() -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(SCHEMA).unwrap();
        PackDb::new("texts", c).unwrap()
    }

    #[test]
    fn list_is_sorted_and_flattened() {
        let p = pack();
        assert!(p.caps.texts);
        let l = list_texts(Some(&p)).unwrap();
        assert_eq!(l.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["a-text", "b-text"]);
        let a = &l[0];
        assert_eq!((a.shelf.as_str(), a.period.as_deref(), a.year, a.level.as_deref(), a.chars), ("graded", Some("ancient"), Some(-2333), Some("TOPIK 2"), 12));
        assert_eq!((a.title_ko.as_str(), a.title_en.as_str(), a.author_en.as_str(), a.date.as_str()), ("단군", "Dangun", "AI", "2026"));
        assert_eq!(a.themes, ["ancient-goryeo", "joseon"]);
        assert!(a.excerpt && !l[1].excerpt);
        assert_eq!(a.summary_en, "Summary");
        assert_eq!(a.labels["translation"], "ai");
        assert_eq!(l[1].level, None);
        let v = serde_json::to_value(&l[1]).unwrap();
        assert!(v["level"].is_null() && v["summary_ko"] == "");
    }

    #[test]
    fn get_text_with_paragraphs_in_order() {
        let p = pack();
        let t = get_text(Some(&p), "a-text").unwrap().unwrap();
        assert_eq!(t.id, "a-text");
        assert_eq!(t.card["level"], "TOPIK 2");
        assert_eq!(t.notes["en"], "e");
        assert_eq!(t.provenance["source"], "original");
        assert_eq!(t.review["status"], "approved");
        assert_eq!(t.vocab[0]["word"], "곰");
        assert_eq!(t.questions[0]["answer_en"], "!");
        assert_eq!(t.paragraphs.iter().map(|p| (p.n, p.orig.as_str(), p.modern.as_deref(), p.en.as_deref())).collect::<Vec<_>>(), [(0, "첫째", None, Some("First")), (1, "둘째", Some("둘째m"), Some("Second"))]);
        let b = get_text(Some(&p), "b-text").unwrap().unwrap();
        assert!(b.vocab.is_null() && b.questions.is_null());
        assert_eq!(b.paragraphs[0].en, None);
        assert_eq!(get_text(Some(&p), "nope").unwrap(), None);
    }

    #[test]
    fn missing_pack_gives_empty_and_null() {
        assert!(list_texts(None).unwrap().is_empty());
        assert_eq!(get_text(None, "a-text").unwrap(), None);
    }

    #[test]
    fn texts_pack_is_not_a_dictionary_pack() {
        // a pack with neither entries nor texts is still rejected
        assert!(PackDb::new("x", Conn::open_memory().unwrap()).is_err());
        let c = Conn::open_memory().unwrap();
        c.exec("CREATE TABLE texts (id TEXT)").unwrap();
        assert!(PackDb::new("texts", c).is_err(), "needs paragraphs too");
    }
}
