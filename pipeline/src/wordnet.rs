//! Open English WordNet (CC BY 4.0) -> `en_rel`: Korean words for English words that the
//! Korean dictionaries never use as a gloss in that part of speech ("articulate" adj.), reached
//! through the word's WordNet synonyms ("fluent" -> 유창하다). Precomputed at build time against
//! `gloss_terms` + `entries` of the core pack, so the engine only reads rows.
//!
//! Precision rules (the matches are automatic, and the app labels them so):
//! * only single-word synset members and, for adjectives, "similar" synsets (hypernyms and
//!   multi-word synonyms add too much noise);
//! * a Korean word must have the same part of speech, and either have the synonym as its first
//!   gloss item (tier 0) or be reached through two different synonyms.

use crate::xml::{for_each, Node};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::io::Read;

/// Senses kept per (term, part of speech) and Korean words per sense.
pub const MAX_SENSES: usize = 3;
pub const MAX_WORDS: usize = 6;
/// Words already given as the first gloss of this many Korean entries (any pos) are taken as
/// well covered and get no rows: the WordNet route would only add noise above good results.
pub const WELL_COVERED: usize = 3;
/// Function words: their rare WordNet senses ("in" = fashionable, "I" = iodine) are not what a
/// learner is looking for.
const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "i", "me", "my", "we", "us", "our", "you", "your", "he", "him", "his", "she", "her", "it", "its",
    "they", "them", "their", "this", "that", "these", "those", "who", "whom", "whose", "which", "what", "in", "on", "at",
    "by", "for", "of", "to", "from", "with", "about", "as", "into", "onto", "over", "under", "up", "down", "out", "off",
    "or", "and", "but", "nor", "so", "if", "than", "be", "am", "is", "are", "was", "were", "do", "does", "did", "have",
    "has", "had", "will", "would", "shall", "should", "can", "could", "may", "might", "must", "not", "no", "yes", "any",
    "some", "all", "one", "s", "m", "t", "d", "mr", "mrs", "ms",
];
/// Korean gloss hits read per synonym.
const HITS_PER_SYNONYM: usize = 4;
/// Definitions are cut to this many characters.
const SENSE_CHARS: usize = 80;

pub struct WordNet {
    /// (lower-cased lemma, pos letter n/v/a/r) -> synset ids in sense order
    pub entries: HashMap<(String, char), Vec<String>>,
    /// synset id -> (member lemmas, definition, "similar" synset ids)
    pub synsets: HashMap<String, (Vec<String>, String, Vec<String>)>,
}

/// WordNet pos letter -> (English label stored in `en_rel.pos`, Korean `entries.pos`).
pub fn pos_names(p: char) -> (&'static str, &'static str) {
    match p {
        'n' => ("noun", "noun"),
        'v' => ("verb", "verb"),
        'a' => ("adjective", "adjective"),
        _ => ("adverb", "adverb"),
    }
}

fn pos_letter(p: &str) -> Option<char> {
    match p {
        "n" => Some('n'),
        "v" => Some('v'),
        "a" | "s" => Some('a'),
        "r" => Some('r'),
        _ => None,
    }
}

/// Parse the WN-LMF XML (`english-wordnet-*.xml`, possibly gzipped by the caller).
pub fn parse<R: Read, F: Fn() -> Result<R>>(open: F) -> Result<WordNet> {
    let mut lemma_of: HashMap<String, String> = HashMap::new();
    let mut entries: HashMap<(String, char), Vec<String>> = HashMap::new();
    for_each(open()?, "LexicalEntry", |n: &Node| {
        let Some(lem) = n.child("Lemma") else { return Ok(()) };
        let (Some(w), Some(p)) = (lem.attr("writtenForm"), lem.attr("partOfSpeech").and_then(pos_letter)) else { return Ok(()) };
        if let Some(id) = n.attr("id") {
            lemma_of.insert(id.to_string(), w.to_string());
        }
        let v = entries.entry((w.to_lowercase(), p)).or_default();
        v.extend(n.kids("Sense").filter_map(|s| s.attr("synset")).map(str::to_string));
        Ok(())
    })?;
    let mut synsets = HashMap::new();
    for_each(open()?, "Synset", |n: &Node| {
        let Some(id) = n.attr("id") else { return Ok(()) };
        let members = n.attr("members").unwrap_or("").split_whitespace().filter_map(|m| lemma_of.get(m)).cloned().collect();
        let def = n.child("Definition").map(|d| d.text.trim().to_string()).unwrap_or_default();
        let similar = n
            .kids("SynsetRelation")
            .filter(|r| r.attr("relType") == Some("similar"))
            .filter_map(|r| r.attr("target"))
            .map(str::to_string)
            .collect();
        synsets.insert(id.to_string(), (members, def, similar));
        Ok(())
    })?;
    Ok(WordNet { entries, synsets })
}

pub struct RelRow {
    pub term_norm: String,
    pub pos: &'static str,
    pub sense: String,
    pub via: String,
    pub ko: String,
    pub rank: i64,
}

fn clip(s: &str) -> String {
    if s.chars().count() <= SENSE_CHARS {
        return s.to_string();
    }
    let cut: String = s.chars().take(SENSE_CHARS).collect();
    match cut.rfind([' ', ';', ',']) {
        Some(i) if i > SENSE_CHARS / 2 => format!("{}…", cut[..i].trim_end_matches([';', ','])),
        _ => format!("{cut}…"),
    }
}

/// Korean (headword, tier) hits of an English gloss item, restricted to a Korean pos ("" = any).
type Hits<'a> = dyn FnMut(&str, &str) -> Result<Vec<(String, i64)>> + 'a;

/// Short Wiktionary definitions: (word, pos letter) -> items ("poo" noun -> feces, poop, …).
pub type Defs = HashMap<(String, char), Vec<String>>;

/// Read `kaikki-en-defs.tsv` (`word \t kaikki pos \t item|item…`); other parts of speech are skipped.
pub fn parse_defs<R: std::io::BufRead>(rd: R) -> Result<Defs> {
    let mut out: Defs = HashMap::new();
    for line in rd.lines() {
        let line = line?;
        let mut it = line.split('\t');
        let (Some(w), Some(pos), Some(items)) = (it.next(), it.next(), it.next()) else { continue };
        let p = match pos {
            "noun" => 'n',
            "verb" => 'v',
            "adj" => 'a',
            "adv" => 'r',
            _ => continue,
        };
        let v = out.entry((w.to_string(), p)).or_default();
        for i in items.split('|') {
            if !v.iter().any(|x| x == i) {
                v.push(i.to_string());
            }
        }
    }
    Ok(out)
}

/// Memoised gloss lookups.
struct Lookup<'a, 'b> {
    hits: &'a mut Hits<'b>,
    cache: HashMap<(String, &'static str), Vec<(String, i64)>>,
}

impl Lookup<'_, '_> {
    /// `kpos` "" = any part of speech
    fn get(&mut self, w: &str, kpos: &'static str) -> Result<Vec<(String, i64)>> {
        if let Some(v) = self.cache.get(&(w.to_string(), kpos)) {
            return Ok(v.clone());
        }
        let v = (self.hits)(w, kpos)?;
        self.cache.insert((w.to_string(), kpos), v.clone());
        Ok(v)
    }

    /// Korean words reached through `cands` (single words other than `lemma`): kept when a
    /// candidate is their first gloss (2 points) or two candidates reach them; (ko, via).
    fn through(&mut self, lemma: &str, p: char, cands: &[String]) -> Result<Vec<(String, String)>> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut score: HashMap<String, (i64, usize, String)> = HashMap::new(); // ko -> (score, first seen, via)
        for c in cands {
            let cl = c.to_lowercase();
            // multi-word synonyms ("sound out") match Korean words too loosely
            if cl == lemma || cl.contains(' ') || !seen.insert(cl.clone()) {
                continue;
            }
            let mut this: HashSet<String> = HashSet::new();
            for (ko, tier) in self.get(&cl, pos_names(p).1)?.into_iter().take(HITS_PER_SYNONYM) {
                if !this.insert(ko.clone()) {
                    continue;
                }
                let n = score.len();
                let e = score.entry(ko).or_insert((0, n, cl.clone()));
                e.0 += if tier == 0 { 2 } else { 1 };
            }
        }
        let mut words: Vec<(usize, String, String)> =
            score.into_iter().filter(|(_, (s, _, _))| *s >= 2).map(|(ko, (_, i, via))| (i, ko, via)).collect();
        words.sort();
        words.truncate(MAX_WORDS);
        Ok(words.into_iter().map(|(_, ko, via)| (ko, via)).collect())
    }
}

/// The `en_rel` rows: for every single-word English word (WordNet lemma or Wiktionary entry) and
/// part of speech without a same-pos Korean gloss match of its own. A short Wiktionary definition
/// ("poo": feces) is the first route; WordNet synsets are used only when it finds nothing.
pub fn rows(wn: &WordNet, defs: &Defs, hits: &mut Hits) -> Result<Vec<RelRow>> {
    let ok = |l: &str| l.chars().count() >= 3 && l.chars().all(|c| c.is_alphabetic() || c == '-');
    let mut keys: Vec<&(String, char)> = wn.entries.keys().chain(defs.keys()).filter(|(l, _)| ok(l)).collect();
    keys.sort();
    keys.dedup();
    let mut lk = Lookup { hits, cache: HashMap::new() };
    let mut out = Vec::new();
    // rank runs on across the parts of speech of one lemma (keys are sorted by lemma)
    let mut rank = 0;
    let mut prev: Option<&str> = None;
    for (lemma, p) in keys {
        if prev != Some(lemma.as_str()) {
            rank = 0;
            prev = Some(lemma);
        }
        if FUNCTION_WORDS.contains(&lemma.as_str()) || !lk.get(lemma, pos_names(*p).1)?.is_empty() {
            continue; // the dictionaries already answer this word in this pos
        }
        if lk.get(lemma, "")?.iter().filter(|(_, t)| *t == 0).count() >= WELL_COVERED {
            continue;
        }
        let key = (lemma.clone(), *p);
        if let Some(items) = defs.get(&key) {
            let words = lk.through(lemma, *p, items)?;
            if !words.is_empty() {
                let sense = clip(&format!("“{}”", items.iter().take(4).cloned().collect::<Vec<_>>().join(", ")));
                for (ko, via) in words {
                    out.push(RelRow { term_norm: lemma.clone(), pos: pos_names(*p).0, sense: sense.clone(), via, ko, rank });
                    rank += 1;
                }
                continue;
            }
        }
        let mut senses = 0;
        for sid in wn.entries.get(&key).into_iter().flatten() {
            let Some((members, def, similar)) = wn.synsets.get(sid) else { continue };
            let mut cands: Vec<String> = members.clone();
            if *p == 'a' {
                for t in similar {
                    if let Some(s) = wn.synsets.get(t) {
                        cands.extend(s.0.iter().cloned());
                    }
                }
            }
            let words = lk.through(lemma, *p, &cands)?;
            if words.is_empty() {
                continue;
            }
            let sense = clip(def);
            for (ko, via) in words {
                out.push(RelRow { term_norm: lemma.clone(), pos: pos_names(*p).0, sense: sense.clone(), via, ko, rank });
                rank += 1;
            }
            senses += 1;
            if senses >= MAX_SENSES {
                break;
            }
        }
    }
    Ok(out)
}

/// Korean (headword, tier) gloss hits from the core pack being built (single words only).
pub fn sql_hits(conn: &Connection) -> impl FnMut(&str, &str) -> Result<Vec<(String, i64)>> + '_ {
    move |term: &str, kpos: &str| {
        let mut st = conn.prepare_cached(
            "SELECT e.headword, g.tier FROM gloss_terms g JOIN entries e ON e.id = g.entry_id \
             WHERE g.term = ?1 AND e.kind = 'word' AND (?2 = '' OR e.pos = ?2) ORDER BY g.score LIMIT 12",
        )?;
        let v = st.query_map(params![term, kpos], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0"?>
<LexicalResource><Lexicon id="oewn">
<LexicalEntry id="e-articulate-a"><Lemma writtenForm="articulate" partOfSpeech="a"/><Sense id="s1" synset="S-art"/></LexicalEntry>
<LexicalEntry id="e-eloquent-a"><Lemma writtenForm="eloquent" partOfSpeech="s"/><Sense id="s2" synset="S-art"/></LexicalEntry>
<LexicalEntry id="e-fluent-a"><Lemma writtenForm="fluent" partOfSpeech="s"/><Sense id="s3" synset="S-art"/></LexicalEntry>
<LexicalEntry id="e-silver-a"><Lemma writtenForm="silver" partOfSpeech="s"/><Sense id="s4" synset="S-art"/></LexicalEntry>
<LexicalEntry id="e-articulate-v"><Lemma writtenForm="articulate" partOfSpeech="v"/><Sense id="s6" synset="S-say"/></LexicalEntry>
<LexicalEntry id="e-pronounce-v"><Lemma writtenForm="pronounce" partOfSpeech="v"/><Sense id="s7" synset="S-say"/></LexicalEntry>
<LexicalEntry id="e-soundout-v"><Lemma writtenForm="sound out" partOfSpeech="v"/><Sense id="s8" synset="S-say"/></LexicalEntry>
<LexicalEntry id="e-clear-a"><Lemma writtenForm="clear" partOfSpeech="a"/><Sense id="s5" synset="S-clear"/></LexicalEntry>
<Synset id="S-art" partOfSpeech="a" members="e-articulate-a e-eloquent-a e-fluent-a e-silver-a"><Definition>expressing yourself easily or characterized by clear expressive language</Definition><SynsetRelation relType="similar" target="S-clear"/></Synset>
<Synset id="S-say" partOfSpeech="v" members="e-articulate-v e-pronounce-v e-soundout-v"><Definition>speak in a certain way</Definition></Synset>
<Synset id="S-clear" partOfSpeech="a" members="e-clear-a"><Definition>readily apparent</Definition></Synset>
</Lexicon></LexicalResource>"#;

    #[test]
    fn related_words_through_synonyms() {
        let wn = parse(|| Ok(XML.as_bytes())).unwrap();
        assert_eq!(wn.entries[&("eloquent".to_string(), 'a')], vec!["S-art".to_string()]);
        // fluent -> 유창하다 as its first gloss (kept); silver -> 은빛 only as a later gloss (dropped);
        // clear (similar synset) and eloquent both reach 명료하다 (kept: two synonyms)
        let mut hits = |w: &str, pos: &str| -> Result<Vec<(String, i64)>> {
            if pos.is_empty() {
                return Ok(vec![]);
            }
            if pos == "verb" {
                return Ok(match w {
                    "pronounce" => vec![("발음하다".into(), 0)],
                    "sound out" => vec![("떠보다".into(), 0)],
                    _ => vec![],
                });
            }
            Ok(match w {
                "fluent" => vec![("유창하다".into(), 0)],
                "silver" => vec![("은빛".into(), 1)],
                "clear" | "eloquent" => vec![("명료하다".into(), 1)],
                _ => vec![],
            })
        };
        let rows = rows(&wn, &Defs::new(), &mut hits).unwrap();
        let art: Vec<(&str, &str, &str)> =
            rows.iter().filter(|r| r.term_norm == "articulate").map(|r| (r.ko.as_str(), r.via.as_str(), r.pos)).collect();
        assert_eq!(art, vec![("명료하다", "eloquent", "adjective"), ("유창하다", "fluent", "adjective"), ("발음하다", "pronounce", "verb")]);
        let ranks: Vec<i64> = rows.iter().filter(|r| r.term_norm == "articulate").map(|r| r.rank).collect();
        assert_eq!(ranks, [0, 1, 2], "ranks run on across parts of speech");
        assert!(rows.iter().all(|r| r.term_norm != "fluent"), "words with their own gloss get no rows");
        // a word that is the first gloss of 3+ Korean entries gets no rows for its other senses
        let mut hits2 = |w: &str, pos: &str| -> Result<Vec<(String, i64)>> {
            Ok(match (w, pos) {
                ("articulate", "") => vec![("가".into(), 0), ("나".into(), 0), ("다".into(), 0)],
                ("fluent", "adjective") => vec![("유창하다".into(), 0)],
                _ => vec![],
            })
        };
        assert!(super::rows(&wn, &Defs::new(), &mut hits2).unwrap().iter().all(|r| r.term_norm != "articulate"));
        assert!(rows[0].sense.starts_with("expressing yourself easily"));
        assert!(rows[0].sense.chars().count() <= SENSE_CHARS + 1);
    }
}
