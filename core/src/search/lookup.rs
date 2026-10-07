//! `lookup_in_text`: dictionary lookup for the Reader. Given a paragraph and a character offset
//! (UTF-16 code units, i.e. a JS string index) return the best dictionary match around it.
//!
//! * Hangul: the eojeol (maximal run of Hangul letters, old jamo and 방점 tone marks) goes through
//!   the strict Hangul search (exact headword, listed forms, deconjugation). If that finds nothing:
//!   (1) 옛말 entries (`entries.hist = 1`) for the word and its old-spelling variants
//!   ([`crate::oldhangul`]), (2) the strict search for each variant, (3) the longest leading part
//!   of the eojeol (>= 2 syllables) that is a headword. Rows found by (1) have `via: 'hist'`,
//!   by (2) `via: 'spelling'`.
//! * Han: the longest dictionary word that covers the offset inside the run of hanja (ties: the
//!   one that starts earlier) in the Korean packs (`entries.hanja`) and in `cedict` / `zhwikt`
//!   (headword or simplified form). Falls back to the single character (`hanja` card with 훈음).

use super::*;
use crate::hangul::norm_headword;
use crate::oldhangul;

/// Most rows returned.
pub const LOOKUP_ROWS: usize = 30;
/// Longest hanja word tried (characters).
const MAX_HAN_WORD: usize = 8;
/// Old-spelling variants looked up in the strict (per-variant) pass.
const MAX_SPELLING_PASS: usize = 12;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TextMatch {
    /// The matched text (a slice of the input).
    #[serde(rename = "match")]
    pub matched: String,
    /// UTF-16 code unit offsets of the match in the input, `end` exclusive.
    pub start: usize,
    pub end: usize,
    pub rows: Vec<ResultRow>,
    /// Hanja card (with 훈음) when the match is a single Han character.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanja: Option<HanjaChar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deconj: Option<Vec<DeconjOut>>,
}

impl TextMatch {
    fn none(at: usize) -> TextMatch {
        TextMatch { matched: String::new(), start: at, end: at, rows: vec![], hanja: None, deconj: None }
    }
}

fn is_old_jamo(c: char) -> bool {
    let u = c as u32;
    (0x1100..=0x11FF).contains(&u) || (0xA960..=0xA97F).contains(&u) || (0xD7B0..=0xD7FF).contains(&u)
}

fn is_hangul_word_char(c: char) -> bool {
    is_hangul_char(c) || is_old_jamo(c) || oldhangul::is_tone_mark(c)
}

fn is_hangul_char(c: char) -> bool {
    crate::hangul::is_hangul_char(c)
}

#[derive(PartialEq, Clone, Copy)]
enum Kind {
    Hangul,
    Han,
    Other,
}

fn kind(c: char) -> Kind {
    if is_han(c) {
        Kind::Han
    } else if is_hangul_word_char(c) {
        Kind::Hangul
    } else {
        Kind::Other
    }
}

fn tag(rows: &mut [ResultRow], via: &'static str) {
    for r in rows {
        r.via = Some(via);
    }
}

/// 옛말 rows for the keys, best (earliest) candidate first. `cands[i]` owns `keys[i]`.
fn hist_rows(packs: &[&PackDb], cands: &[String]) -> Result<Vec<ResultRow>> {
    let with: Vec<&PackDb> = packs.iter().copied().filter(|p| p.caps.hist).collect();
    if with.is_empty() {
        return Ok(vec![]);
    }
    // key -> index of the first candidate that produced it (the candidate itself, then a few lemmas)
    let mut owner: HashMap<String, usize> = HashMap::new();
    let mut keys: Vec<String> = Vec::new();
    for (i, c) in cands.iter().enumerate() {
        let mut ks = vec![c.clone()];
        if i < 8 {
            ks.extend(deconjugate(c).into_iter().map(|d| d.lemma));
        }
        for k in ks {
            if !owner.contains_key(&k) {
                owner.insert(k.clone(), i);
                keys.push(k);
            }
        }
    }
    let mut found: Vec<(usize, ResultRow)> = Vec::new();
    for p in &with {
        for chunk in keys.chunks(40) {
            let ph = (1..=chunk.len()).map(|i| format!("?{i}")).collect::<Vec<_>>().join(",");
            let sql = format!("SELECT {COLS} FROM entries e WHERE e.hist = 1 AND e.hw_norm IN ({ph}) ORDER BY e.rank");
            let params: Vec<Val> = chunk.iter().map(|k| Val::Text(k.clone())).collect();
            for r in p.conn.query(&sql, &params)? {
                let row = result_row(&p.id, &r, Some("hist"));
                let i = owner.get(&row.hw_norm).copied().unwrap_or(usize::MAX);
                found.push((i, row));
            }
        }
    }
    let Some(best) = found.iter().map(|f| f.0).min() else { return Ok(vec![]) };
    let mut rows: Vec<ResultRow> = found.into_iter().filter(|f| f.0 == best).map(|f| f.1).collect();
    sort_by_rank(&mut rows);
    Ok(rows)
}

fn lookup_hangul(korean: &[&PackDb], chars: &[char], u16_at: &[usize], s: usize, e: usize, limit: usize) -> Result<TextMatch> {
    let slice: String = chars[s..e].iter().collect();
    let whole = |rows: Vec<ResultRow>, deconj: Vec<DeconjOut>| TextMatch {
        matched: slice.clone(),
        start: u16_at[s],
        end: u16_at[e],
        rows,
        hanja: None,
        deconj: (!deconj.is_empty()).then_some(deconj),
    };
    let cleaned = oldhangul::clean(&slice);
    let q = norm_headword(&cleaned);
    if q.is_empty() {
        return Ok(TextMatch::none(u16_at[s]));
    }
    // 1. the ordinary (strict) Hangul search
    let (mut rows, deconj) = hangul_strict(korean, &q, limit)?;
    if !rows.is_empty() {
        rows.truncate(limit);
        return Ok(whole(rows, deconj));
    }
    // 2. 옛말 entries for the word and its old-spelling variants
    let mut cands = vec![q.clone()];
    cands.extend(oldhangul::variants(&q).into_iter().map(|v| norm_headword(&v)));
    cands.dedup();
    let mut rows = hist_rows(korean, &cands)?;
    if !rows.is_empty() {
        rows.truncate(limit);
        return Ok(whole(rows, vec![]));
    }
    // 3. modern entries for the spelling variants
    for c in cands.iter().skip(1).take(MAX_SPELLING_PASS) {
        let (mut rows, deconj) = hangul_strict(korean, c, limit)?;
        if !rows.is_empty() {
            tag(&mut rows, "spelling");
            rows.truncate(limit);
            return Ok(whole(rows, deconj));
        }
    }
    // 4. longest leading part of the eojeol that is a headword (particles / endings we cannot strip)
    let n = chars[s..e].len();
    if n >= 3 && cleaned == slice {
        let prefixes: Vec<String> = (2..n).rev().map(|k| chars[s..s + k].iter().collect()).collect();
        let hits = rows_by_keys(korean, KeyCol::Headword, &prefixes, Some("prefix"))?;
        if let Some(best) = hits.iter().map(|(k, _)| k.chars().count()).max() {
            let mut rows: Vec<ResultRow> = hits.into_iter().filter(|(k, _)| k.chars().count() == best).map(|(_, r)| r).collect();
            sort_by_rank(&mut rows);
            rows.truncate(limit);
            return Ok(TextMatch {
                matched: chars[s..s + best].iter().collect(),
                start: u16_at[s],
                end: u16_at[s + best],
                rows,
                hanja: None,
                deconj: None,
            });
        }
    }
    Ok(whole(vec![], vec![]))
}

fn lookup_han(packs: &[&PackDb], chars: &[char], u16_at: &[usize], idx: usize, limit: usize) -> Result<TextMatch> {
    let (korean, han) = split_han(packs);
    let n = chars.len();
    let mut rs = idx;
    while rs > 0 && is_han(chars[rs - 1]) && idx - rs < MAX_HAN_WORD {
        rs -= 1;
    }
    let mut re = idx + 1;
    while re < n && is_han(chars[re]) && re - idx < MAX_HAN_WORD {
        re += 1;
    }
    // candidate spans covering idx, longest first, then the earlier start
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for s in rs..=idx {
        for e in (idx + 1)..=re.min(s + MAX_HAN_WORD) {
            spans.push((s, e));
        }
    }
    spans.sort_by_key(|&(s, e)| (std::cmp::Reverse(e - s), s));
    let text_of = |&(s, e): &(usize, usize)| -> String { chars[s..e].iter().collect() };
    let mut keys: Vec<String> = spans.iter().map(text_of).collect();
    keys.dedup();
    keys.sort();
    keys.dedup();

    let mut by_key: HashMap<String, Vec<ResultRow>> = HashMap::new();
    for (k, r) in rows_by_keys(&korean, KeyCol::Hanja, &keys, Some("hanja"))? {
        by_key.entry(k).or_default().push(r);
    }
    let mut hk = rows_by_keys(&han, KeyCol::Headword, &keys, Some("hanja"))?;
    hk.extend(rows_by_keys(&han, KeyCol::Form, &keys, Some("hanja"))?);
    for (k, r) in hk {
        by_key.entry(k).or_default().push(r);
    }
    for sp in &spans {
        let key = text_of(sp);
        if let Some(rows) = by_key.remove(&key) {
            // Korean packs first (they come first in `by_key` order), duplicates dropped
            let mut seen = HashSet::new();
            let mut out: Vec<ResultRow> = rows.into_iter().filter(|r| seen.insert(super::key(r))).collect();
            let split = out.iter().position(|r| HAN_PACKS.contains(&r.pack.as_str())).unwrap_or(out.len());
            sort_by_rank(&mut out[..split]);
            sort_by_rank(&mut out[split..]);
            out.truncate(limit);
            let single = sp.1 - sp.0 == 1;
            return Ok(TextMatch {
                hanja: if single { hanja_char(&korean, &key)? } else { None },
                matched: key,
                start: u16_at[sp.0],
                end: u16_at[sp.1],
                rows: out,
                deconj: None,
            });
        }
    }
    // nothing in the word lists: the character itself with its 훈음 if known
    let ch = chars[idx].to_string();
    match hanja_char(&korean, &ch)? {
        Some(card) => Ok(TextMatch {
            matched: ch,
            start: u16_at[idx],
            end: u16_at[idx + 1],
            rows: vec![],
            hanja: Some(card),
            deconj: None,
        }),
        None => Ok(TextMatch::none(u16_at[idx])),
    }
}

/// Best dictionary match in `text` around `offset` (UTF-16 code units). See the module docs.
pub fn lookup_in_text(packs: &[&PackDb], text: &str, offset: usize, limit: Option<usize>) -> Result<TextMatch> {
    let limit = limit.unwrap_or(LOOKUP_ROWS).clamp(1, DEFAULT_LIMIT);
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    // u16_at[i] = UTF-16 offset of chars[i]; u16_at[n] = total length
    let mut u16_at: Vec<usize> = Vec::with_capacity(n + 1);
    let mut acc = 0;
    for c in &chars {
        u16_at.push(acc);
        acc += c.len_utf16();
    }
    u16_at.push(acc);
    if n == 0 {
        return Ok(TextMatch::none(0));
    }
    // character containing the offset; just past a word counts as the word's last character
    let mut idx = u16_at.partition_point(|&o| o <= offset).saturating_sub(1).min(n);
    if idx >= n || kind(chars[idx]) == Kind::Other {
        if idx > 0 && kind(chars[idx - 1]) != Kind::Other {
            idx -= 1;
        } else {
            return Ok(TextMatch::none(offset.min(acc)));
        }
    }
    match kind(chars[idx]) {
        Kind::Han => lookup_han(packs, &chars, &u16_at, idx, limit),
        Kind::Hangul => {
            let mut s = idx;
            while s > 0 && is_hangul_word_char(chars[s - 1]) {
                s -= 1;
            }
            let mut e = idx + 1;
            while e < n && is_hangul_word_char(chars[e]) {
                e += 1;
            }
            let (korean, _) = split_han(packs);
            lookup_hangul(&korean, &chars, &u16_at, s, e, limit)
        }
        Kind::Other => Ok(TextMatch::none(offset.min(acc))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = r#"
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER,
  hanja TEXT, pos TEXT, pron TEXT, source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER,
  rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL, hist INTEGER NOT NULL DEFAULT 0);
CREATE INDEX entries_hw_rank ON entries(hw_norm, rank);
CREATE INDEX entries_hanja ON entries(hanja) WHERE hanja IS NOT NULL;
CREATE INDEX entries_hist ON entries(hist, hw_norm) WHERE hist = 1;
CREATE TABLE forms (form TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX forms_form ON forms(form);
CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, rank INTEGER NOT NULL DEFAULT 0);
"#;
    const HANJA_CHARS: &str = "CREATE TABLE hanja_chars (ch TEXT PRIMARY KEY, readings TEXT, meaning_en TEXT, strokes INTEGER, radical TEXT, word_count INTEGER, hun TEXT, eumhun TEXT);";

    /// (id, headword, hanja, source, hist, rank, forms)
    type Row<'a> = (i64, &'a str, Option<&'a str>, &'a str, i64, i64, &'a [&'a str]);

    fn pack(id: &str, lang: &str, rows: &[Row]) -> PackDb {
        let c = Conn::open_memory().unwrap();
        c.exec(SCHEMA).unwrap();
        for (eid, hw, hanja, source, hist, rank, forms) in rows {
            c.query(
                "INSERT INTO entries (id, headword, hw_norm, hanja, pos, source, lang, rank, kind, gloss, data, hist) VALUES (?1,?2,?3,?4,'noun',?5,?6,?7,'word',?8,'{\"senses\":[]}',?9)",
                &[(*eid).into(), (*hw).into(), norm_headword(hw).into(), (*hanja).into(), (*source).into(), lang.into(), (*rank).into(), format!("gloss of {hw}").into(), (*hist).into()],
            )
            .unwrap();
            for f in *forms {
                c.query("INSERT INTO forms VALUES (?1, ?2)", &[(*f).into(), (*eid).into()]).unwrap();
            }
        }
        PackDb::new(id, c).unwrap()
    }

    fn core() -> PackDb {
        let p = pack(
            "core",
            "en",
            &[
                (1, "학교", Some("學校"), "krdict", 0, 100, &[]),
                (2, "가다", None, "krdict", 0, 50, &[]),
                (3, "서울", None, "krdict", 0, 60, &[]),
                (4, "생활", Some("生活"), "krdict", 0, 120, &[]),
                (5, "하다", None, "krdict", 0, 10, &[]),
                (6, "조타", None, "krdict", 0, 400, &[]),
                (7, "학", None, "krdict", 0, 900, &[]),
            ],
        );
        p.conn.exec(HANJA_CHARS).unwrap();
        p.conn.exec("INSERT INTO hanja_chars VALUES ('學','학','to study',16,'子',3,'배울','배울 학'), ('校','교','school',10,'木',2,NULL,NULL)").unwrap();
        PackDb::new("core", p.conn).unwrap()
    }

    fn oldwords() -> PackDb {
        pack(
            "opendict",
            "ko",
            &[
                (1, "아름", None, "opendict", 1, 9000, &[]),
                (2, "어떠", None, "opendict", 1, 9010, &[]),
                                (4, "한", None, "opendict", 0, 9030, &[]),
                (5, "뜻", None, "opendict", 1, 9040, &[]),
                (6, "하얏", None, "opendict", 1, 9050, &[]),
            ],
        )
    }

    fn cedict() -> PackDb {
        pack(
            "cedict",
            "en",
            &[
                (1, "學校", Some("學校"), "cedict", 0, 1_200_000, &["学校"]),
                (2, "學", Some("學"), "cedict", 0, 1_100_000, &["学"]),
                (3, "校長", Some("校長"), "cedict", 0, 1_200_001, &["校长"]),
                (4, "學校生活", Some("學校生活"), "cedict", 0, 1_400_000, &["学校生活"]),
            ],
        )
    }

    fn zhwikt() -> PackDb {
        pack("zhwikt", "en", &[(1, "學", Some("學"), "zhwikt", 0, 1_100_000, &["学"]), (2, "之乎者也", Some("之乎者也"), "zhwikt", 0, 1_500_000, &[])])
    }

    #[test]
    fn han_search_merges_chinese_packs_after_korean() {
        let (c, d, z) = (core(), cedict(), zhwikt());
        let r = search(&[&c, &d, &z], "學校", None).unwrap();
        assert_eq!(r.mode, "hanja");
        let packs: Vec<&str> = r.rows.iter().map(|x| x.pack.as_str()).collect();
        let first_han = packs.iter().position(|p| *p == "cedict").unwrap();
        assert!(packs[..first_han].iter().all(|p| *p == "core") && packs[first_han..].iter().all(|p| *p != "core"));
        assert!(r.rows.iter().filter(|x| x.pack == "cedict").all(|x| x.via == Some("hanja")));
        assert_eq!(r.rows.iter().find(|x| x.pack == "cedict").unwrap().headword, "學校");
        // longer words starting with the query follow
        assert!(r.rows.iter().any(|x| x.headword == "學校生活"));
        // hanja card with 훈음
        let card = search(&[&c, &d], "學", None).unwrap().hanja.unwrap();
        assert_eq!((card[0].hun.as_deref(), card[0].eumhun.as_deref()), (Some("배울"), Some("배울 학")));
    }

    #[test]
    fn han_search_simplified_and_longest_prefix() {
        let (c, d, z) = (core(), cedict(), zhwikt());
        let r = search(&[&c, &d, &z], "学校", None).unwrap();
        assert!(r.rows.iter().any(|x| x.pack == "cedict" && x.headword == "學校"), "simplified -> traditional");
        let r = search(&[&c, &d, &z], "学", None).unwrap();
        assert!(r.rows.iter().any(|x| x.pack == "zhwikt" && x.headword == "學"));
        // unknown run: the longest known prefix is shown
        let r = search(&[&d], "學校長輩", None).unwrap();
        let hw: Vec<&str> = r.rows.iter().map(|x| x.headword.as_str()).collect();
        assert_eq!(hw, vec!["學校"], "{hw:?}");
        assert_eq!(r.rows[0].via, Some("hanja"));
        // Chinese packs do not leak into Hangul / English searches
        let r = search(&[&c, &d], "학교", None).unwrap();
        assert!(r.rows.iter().all(|x| x.pack == "core"));
        assert!(search(&[&d], "school", None).unwrap().rows.is_empty());
    }

    #[test]
    fn lookup_hangul_eojeol_with_deconjugation() {
        let c = core();
        let text = "그는 학교에서 갔어요.";
        // offset inside 학교에서 (UTF-16 index of '교')
        let off = text.chars().position(|ch| ch == '교').unwrap();
        let m = lookup_in_text(&[&c], text, off, None).unwrap();
        assert_eq!(m.matched, "학교에서");
        assert_eq!((m.start, m.end), (3, 7));
        assert_eq!(m.rows[0].headword, "학교");
        assert_eq!(m.rows[0].via, Some("deconj"));
        // just past the word (on the space) still selects it; punctuation is not part of the eojeol
        let m = lookup_in_text(&[&c], text, 7, None).unwrap();
        assert_eq!(m.matched, "학교에서");
        let m = lookup_in_text(&[&c], text, text.chars().count() - 1, None).unwrap();
        assert_eq!(m.matched, "갔어요");
        assert_eq!(m.rows[0].headword, "가다");
        // spaces only / empty text: empty match
        assert_eq!(lookup_in_text(&[&c], "  ", 1, None).unwrap().rows.len(), 0);
        assert_eq!(lookup_in_text(&[&c], "", 0, None).unwrap().matched, "");
        // unknown word: match is still the eojeol, no rows
        let m = lookup_in_text(&[&c], "쀍쀍", 0, None).unwrap();
        assert_eq!((m.matched.as_str(), m.rows.len()), ("쀍쀍", 0));
    }

    #[test]
    fn lookup_falls_back_to_historical_entries() {
        let (c, o) = (core(), oldwords());
        assert!(o.caps.hist);
        // 옛말 headword, not in core
        let m = lookup_in_text(&[&c, &o], "아름 다운", 1, None).unwrap();
        assert_eq!(m.matched, "아름");
        assert_eq!((m.rows[0].headword.as_str(), m.rows[0].via), ("아름", Some("exact"))); // an 옛말 headword in modern spelling: normal search
        // old spelling: 어 + ᄯ ᅥ -> 어떠, a 옛말 entry
        let old = "어\u{112F}\u{1165}"; // 어 + ᄯ ᅥ -> 어떠
        let m = lookup_in_text(&[&c, &o], old, 0, None).unwrap();
        assert_eq!(m.rows[0].headword, "어떠");
        assert_eq!(m.rows[0].via, Some("hist"));
        assert_eq!((m.start, m.end), (0, old.encode_utf16().count()));
        // ᄠ + ᅳ + ᆺ -> 뜻 (pieup-tikeut), 방점 tone mark attached to the eojeol and ignored
        let m = lookup_in_text(&[&c, &o], "\u{1120}\u{1173}\u{11BA}\u{302F} 이", 0, None).unwrap();
        assert_eq!(m.rows[0].headword, "뜻");
        assert_eq!(m.rows[0].via, Some("hist"));
        assert_eq!((m.start, m.end), (0, 4));
    }

    #[test]
    fn lookup_old_spelling_variants_reach_modern_entries() {
        let (c, o) = (core(), oldwords());
        // 갓다 -> 갔다 -> (deconj) 가다
        let m = lookup_in_text(&[&c], "서울에 갓다", 5, None).unwrap();
        assert_eq!(m.matched, "갓다");
        assert_eq!(m.rows[0].headword, "가다");
        assert_eq!(m.rows[0].via, Some("spelling"));
        let m = lookup_in_text(&[&c], "셔울", 0, None).unwrap();
        assert_eq!((m.rows[0].headword.as_str(), m.rows[0].via), ("서울", Some("spelling")));
        let m = lookup_in_text(&[&c], "됴타", 0, None).unwrap();
        assert_eq!((m.rows[0].headword.as_str(), m.rows[0].via), ("조타", Some("spelling")));
        // arae-a: ᄒ + ᆞ + ᆫ -> 한 (hist pack has a non-hist 한, found as a modern spelling)
        let m = lookup_in_text(&[&c, &o], "\u{1112}\u{119E}\u{11AB}", 0, None).unwrap();
        assert_eq!(m.rows[0].headword, "한");
        // packs without the hist column simply skip that step
        let m = lookup_in_text(&[&c], "\u{1112}\u{119E}\u{11AB}", 0, None).unwrap();
        assert_eq!(m.rows[0].headword, "하다".replace("하다", "하다")); // 하 + ㄴ -> 한 ~ deconj of 한? either way modern
    }

    #[test]
    fn lookup_prefix_fallback_narrows_the_match() {
        let c = core();
        // 학교쀍쀍: no deconjugation can strip it; the longest leading headword is returned
        let m = lookup_in_text(&[&c], "학교쀍쀍 가", 0, None).unwrap();
        assert_eq!(m.matched, "학교");
        assert_eq!((m.start, m.end), (0, 2));
        assert_eq!(m.rows[0].via, Some("prefix"));
    }

    #[test]
    fn lookup_han_longest_match_and_single_char_fallback() {
        let (c, d, z) = (core(), cedict(), zhwikt());
        let packs = [&c, &d, &z];
        // tapping 校 inside 學校生活: the longest covering word wins (學校生活 in cedict)
        let t = "他在學校生活。";
        let m = lookup_in_text(&packs, t, 3, None).unwrap();
        assert_eq!(m.matched, "學校生活");
        assert_eq!((m.start, m.end), (2, 6));
        // Korean pack rows (by hanja) come before Chinese pack rows; both are present
        let m = lookup_in_text(&packs, "學校", 0, None).unwrap();
        assert_eq!(m.matched, "學校");
        let order: Vec<&str> = m.rows.iter().map(|r| r.pack.as_str()).collect();
        assert_eq!(order, vec!["core", "cedict"]);
        // simplified text
        let m = lookup_in_text(&[&c, &d], "学校长", 0, None).unwrap();
        assert_eq!(m.matched, "学校");
        assert_eq!(m.rows[0].headword, "學校");
        // 校長 vs 學校: tapping 校 of "學校長": equally long (2) -> the earlier start
        let m = lookup_in_text(&packs, "學校長", 1, None).unwrap();
        assert_eq!(m.matched, "學校");
        // unknown word of one character: hanja card with 훈음
        let m = lookup_in_text(&[&c], "學問", 0, None).unwrap();
        assert_eq!(m.matched, "學");
        assert!(m.rows.is_empty());
        assert_eq!(m.hanja.as_ref().unwrap().eumhun.as_deref(), Some("배울 학"));
        // a character with no record at all: empty match
        let m = lookup_in_text(&[&c], "鑫", 0, None).unwrap();
        assert_eq!((m.matched.as_str(), m.rows.len(), m.hanja.is_none()), ("", 0, true));
        // classical run
        let m = lookup_in_text(&packs, "子曰之乎者也", 2, None).unwrap();
        assert_eq!(m.matched, "之乎者也");
    }

    #[test]
    fn offsets_are_utf16_code_units() {
        let (c, d) = (core(), cedict());
        // 𠮷 is outside the BMP: 2 UTF-16 units, 1 char
        let t = "𠮷學校 가다";
        let m = lookup_in_text(&[&c, &d], t, 3, None).unwrap(); // UTF-16 index of 校
        assert_eq!(m.matched, "學校");
        assert_eq!((m.start, m.end), (2, 4));
        let m = lookup_in_text(&[&c, &d], t, 6, None).unwrap();
        assert_eq!(m.matched, "가다");
        assert_eq!((m.start, m.end), (5, 7));
        // an offset in the middle of a surrogate pair maps to that character
        let m = lookup_in_text(&[&c, &d], t, 1, None).unwrap();
        assert_eq!(m.start, 0);
        // offsets past the end clamp
        let m = lookup_in_text(&[&c, &d], "가다", 99, None).unwrap();
        assert!(m.matched.is_empty() || m.matched == "가다");
    }

    #[test]
    fn mixed_script_text_picks_the_script_under_the_offset() {
        let (c, d) = (core(), cedict());
        let t = "學校에 가다";
        assert_eq!(lookup_in_text(&[&c, &d], t, 1, None).unwrap().matched, "學校");
        let m = lookup_in_text(&[&c, &d], t, 3, None).unwrap();
        assert_eq!(m.matched, "에");
        assert_eq!(lookup_in_text(&[&c, &d], t, 5, None).unwrap().matched, "가다");
        // Latin / digits: nothing
        assert!(lookup_in_text(&[&c], "abc 123", 1, None).unwrap().matched.is_empty());
    }
}
