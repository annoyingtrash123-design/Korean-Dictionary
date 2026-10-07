//! Which Chinese dictionary entries are relevant to Korean: the optional `cedict` / `zhwikt`
//! packs keep only words whose characters are attested as a hanja word in the Korean
//! dictionaries (core, 표준국어대사전, 우리말샘), plus single characters with a Korean reading.
//! Pure Classical/Modern Chinese vocabulary is left to a Chinese dictionary app.

use crate::common::is_cjk;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Default)]
pub struct KoreanHanja {
    /// Character -> variant-class representative (`unihan::variant_classes`).
    variants: HashMap<char, char>,
    /// Canonical runs of hanja found in Korean entries' `hanja` field (陋醜하다 -> 陋醜).
    words: HashSet<String>,
    /// Canonical single characters that have a Korean reading or occur in a Korean word.
    chars: HashSet<char>,
}

impl KoreanHanja {
    pub fn new(variants: HashMap<char, char>) -> Self {
        Self { variants, ..Default::default() }
    }

    fn canon_char(&self, c: char) -> char {
        self.variants.get(&c).copied().unwrap_or(c)
    }

    pub fn canon(&self, s: &str) -> String {
        s.chars().map(|c| self.canon_char(c)).collect()
    }

    /// Record a Korean `hanja` value: every maximal run of ideographs is a word.
    pub fn add_hanja(&mut self, hanja: &str) {
        let mut run = String::new();
        for c in hanja.chars().chain(std::iter::once(' ')) {
            if is_cjk(c) {
                let k = self.canon_char(c);
                run.push(k);
                self.chars.insert(k);
            } else if !run.is_empty() {
                self.words.insert(std::mem::take(&mut run));
            }
        }
    }

    pub fn add_char(&mut self, c: char) {
        let k = self.canon_char(c);
        self.chars.insert(k);
    }

    /// Read `entries.hanja` from a built pack (missing file or table: nothing added).
    pub fn add_pack(&mut self, path: &Path) -> usize {
        let Ok(conn) = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { return 0 };
        let Ok(mut st) = conn.prepare("SELECT DISTINCT hanja FROM entries WHERE hanja IS NOT NULL AND hanja != ''") else { return 0 };
        let Ok(rows) = st.query_map([], |r| r.get::<_, String>(0)) else { return 0 };
        let mut n = 0;
        for h in rows.flatten() {
            self.add_hanja(&h);
            n += 1;
        }
        n
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Is a Chinese headword (any of its written forms) Korean-relevant?
    pub fn keeps<'a>(&self, forms: impl IntoIterator<Item = &'a str>) -> bool {
        forms.into_iter().any(|f| {
            if f.is_empty() || !f.chars().all(is_cjk) {
                return false;
            }
            let k = self.canon(f);
            let mut cs = k.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => self.chars.contains(&c),
                _ => self.words.contains(&k),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_korean_attested_words_across_variants() {
        let mut var = HashMap::new();
        var.insert('敎', '教'); // representative = smaller code point (U+6559)
        let mut k = KoreanHanja::new(var);
        for h in ["敎育", "陋醜하다", "겹顯微鏡", "學校"] {
            k.add_hanja(h);
        }
        k.add_char('吾');
        assert!(k.keeps(["教育"])); // Chinese form of Korean 敎育
        assert!(k.keeps(["陋醜"]));
        assert!(k.keeps(["顯微鏡"]));
        assert!(k.keeps(["学校", "學校"])); // any written form
        assert!(k.keeps(["吾"])); // single character with a Korean reading
        assert!(k.keeps(["學"])); // character occurring in a Korean word
        assert!(!k.keeps(["電腦"])); // Chinese-only word
        assert!(!k.keeps(["顯微"])); // a fragment of a Korean word is not a Korean word
        assert!(!k.keeps(["卡拉OK"]));
        assert!(!k.keeps(["咖"]));
    }
}
