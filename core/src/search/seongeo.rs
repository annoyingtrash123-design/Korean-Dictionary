//! The "Chinese & English" block of a 성어 page (pack table `seongeo_zh`, built by the pipeline from CC-CEDICT and
//! the reviewed English equivalents; see `pipeline/src/seongeo_zh.rs`). Keyed by the entry's hanja.
//!
//! A pack without the table (older packs) gives nothing.

use super::PackDb;
use crate::sql::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SeongeoZh {
    pub hanja: String,
    /// CC-CEDICT chengyu (all four `zh_*` fields are set together, or none).
    pub zh_trad: Option<String>,
    pub zh_simp: Option<String>,
    /// Tone-marked pinyin.
    pub zh_pinyin: Option<String>,
    /// CC-CEDICT English definitions.
    pub zh_en: Vec<String>,
    /// The English idiom with the same meaning (AI-written, reviewed), and how its nuance differs.
    pub en_equiv: Option<String>,
    pub en_note: Option<String>,
    pub en_ai: bool,
}

/// The block of the 성어 whose hanja is `hanja` (▽ / ▼ marks are ignored).
pub fn seongeo_zh(packs: &[&PackDb], hanja: &str) -> Result<Option<SeongeoZh>> {
    let key: String = hanja.chars().filter(|c| *c != '▽' && *c != '▼').collect();
    for p in packs.iter().filter(|p| p.caps.seongeo_zh) {
        let rows = p.conn.query(
            "SELECT hanja, zh_trad, zh_simp, zh_pinyin, zh_en, en_equiv, en_note, en_ai FROM seongeo_zh WHERE hanja = ?1",
            &[key.as_str().into()],
        )?;
        if let Some(r) = rows.first() {
            let zh_en: Vec<String> = r.text(4).and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default();
            return Ok(Some(SeongeoZh {
                hanja: r.string(0),
                zh_trad: r.text(1),
                zh_simp: r.text(2),
                zh_pinyin: r.text(3),
                zh_en,
                en_equiv: r.text(5),
                en_note: r.text(6),
                en_ai: r.int(7).unwrap_or(0) != 0,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::Conn;

    fn pack(with_table: bool) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec("CREATE TABLE entries (id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, source TEXT NOT NULL, lang TEXT NOT NULL, rank INTEGER NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL);").unwrap();
        if with_table {
            c.exec(
                "CREATE TABLE seongeo_zh (hanja TEXT PRIMARY KEY, headword TEXT NOT NULL, zh_trad TEXT, zh_simp TEXT, zh_pinyin TEXT, zh_en TEXT, en_equiv TEXT, en_note TEXT, en_ai INTEGER NOT NULL DEFAULT 0);
                 INSERT INTO seongeo_zh VALUES ('過猶不及', '과유불급', '過猶不及', '过犹不及', 'guò yóu bù jí', '[\"to go too far is as bad as not going far enough\"]', 'too much of a good thing', NULL, 1),
                                               ('他山之石', '타산지석', '他山之石', '他山之石', 'tā shān zhī shí', '[\"a\",\"b\"]', NULL, NULL, 0),
                                               ('一石二鳥', '일석이조', NULL, NULL, NULL, NULL, 'kill two birds with one stone', 'n', 1);",
            )
            .unwrap();
        }
        PackDb::new("core", c).unwrap()
    }

    #[test]
    fn both_halves_one_half_or_none() {
        let p = pack(true);
        let a = seongeo_zh(&[&p], "過猶不及").unwrap().unwrap();
        assert_eq!((a.zh_simp.as_deref(), a.zh_pinyin.as_deref(), a.en_equiv.as_deref(), a.en_ai), (Some("过犹不及"), Some("guò yóu bù jí"), Some("too much of a good thing"), true));
        assert_eq!(a.zh_en.len(), 1);
        let b = seongeo_zh(&[&p], "他山之石").unwrap().unwrap();
        assert_eq!((b.zh_en.len(), b.en_equiv, b.en_ai), (2, None, false));
        let c = seongeo_zh(&[&p], "一石二鳥").unwrap().unwrap();
        assert_eq!((c.zh_trad, c.zh_en.len(), c.en_note.as_deref()), (None, 0, Some("n")));
        assert!(seongeo_zh(&[&p], "不在其中").unwrap().is_none());
    }

    #[test]
    fn dictionary_marks_are_ignored_and_an_older_pack_gives_nothing() {
        let p = pack(true);
        assert!(seongeo_zh(&[&p], "過▽猶不及").unwrap().is_some());
        let old = pack(false);
        assert!(!old.caps.seongeo_zh);
        assert!(seongeo_zh(&[&old], "過猶不及").unwrap().is_none());
    }
}
