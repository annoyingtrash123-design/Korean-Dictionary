//! Matcher fallbacks of the curriculum lists (`src/curriculum.rs`): inflected / spelling variants of
//! grammar patterns, grammar rows that are a sense of a krdict word entry, and words whose bare noun
//! krdict lacks (X하다 / X되다). Every case is a real list row, checked by hand against krdict.

use kdict_pipeline::curriculum::*;
use std::collections::{HashMap, HashSet};

fn g(id: i64, pattern: &str, category: &str) -> GrammarCand {
    GrammarCand { id, entry_id: Some(1000 + id), pattern: pattern.into(), category: category.into() }
}

fn gform(form: &str) -> CurGrammar {
    builtin_grammar().into_iter().find(|g| g.form_raw == form).unwrap_or_else(|| panic!("{form} not in list"))
}

#[test]
fn grammar_inflected_and_spelling_variants_resolve() {
    let rows = vec![
        g(1, "-기 때문", "Expressions"),
        g(2, "-는 탓", "Expressions"),
        g(3, "-ㄴ 탓", "Expressions"),
        g(4, "-은 탓", "Expressions"),
        g(5, "-으려야", "Expressions"),
        g(6, "-려야", "Expressions"),
        g(7, "-래야", "Expressions"),
        g(8, "그려", "Particles"),
        g(9, "-기에", "Connective endings"),
        g(10, "따라", "Particles"),
    ];
    let idx = grammar_index(&rows);
    let m = |form: &str| match_grammar(&gform(form), &rows, &idx).map(|i| rows[i].id);
    // -기 때문에 / -는 탓에: krdict's expression has no closing 에
    assert_eq!(m("-기 때문에"), Some(1));
    assert_eq!(m("-는 탓에"), Some(2), "the main form wins over -ㄴ/-은 탓에");
    // -을래야 is the spoken form of -으려야 (krdict's note says so); not of -래야, a different ending
    assert_eq!(m("-을래야"), Some(5));
    // 종결어미 -그려 is krdict's particle 그려 (-네그려), though a 종결어미 never lands on a particle otherwise
    assert_eq!(m("-그려"), Some(8));
    // no row of the same pattern: stay unmatched rather than land on something near
    assert_eq!(m("-기에 따라"), None);
    assert_eq!(m("-기에 앞서(서)"), None);
    assert_eq!(m("-노라면"), None);
    // 에 따라 must not become the particle 따라 or the ending -기에
    assert_eq!(m("에 따라"), None);
}

#[test]
fn grammar_word_links_are_hand_checked_list_rows() {
    assert_eq!(grammar_word_link(&gform("에 따라")), Some(("따르다", Some(1), "verb")));
    assert_eq!(grammar_word_link(&gform("에 의하여")), Some(("의하다", None, "verb")));
    assert_eq!(grammar_word_link(&gform("이 아니다")), Some(("아니다", None, "adjective")));
    assert_eq!(grammar_word_link(&gform("-어 치우다")), Some(("치우다", None, "auxiliary verb")));
    // other rows have none
    assert_eq!(grammar_word_link(&gform("-노라면")), None);
    // every table entry names a row of the list
    let keys: HashSet<String> = builtin_grammar().iter().map(|g| grammar_key(&g.form_raw)).collect();
    for l in GRAMMAR_WORD_LINKS {
        assert!(keys.contains(l.0), "{} is not a list form", l.0);
    }
    for (cat, from, _) in GRAMMAR_ALIASES {
        assert!(builtin_grammar().iter().any(|g| g.category == *cat && grammar_forms(g).iter().any(|f| grammar_key(f) == *from)), "{from} is not a list form");
    }
}

fn cand(id: i64, pos: &str, level: i64) -> WordCand {
    WordCand { id, homonym: Some(1), pos: pos.into(), level: Some(level), text: String::new() }
}

fn word(form_raw: &str) -> CurWord {
    builtin_words().into_iter().find(|w| w.form_raw == form_raw).unwrap_or_else(|| panic!("{form_raw} not in list"))
}

fn by_hw(c: Vec<(&str, WordCand)>) -> HashMap<String, Vec<WordCand>> {
    let mut m: HashMap<String, Vec<WordCand>> = HashMap::new();
    for (hw, cand) in c {
        m.entry(hw.into()).or_default().push(cand);
    }
    m
}

#[test]
fn words_fall_back_to_derived_verbs_and_adjectives() {
    let none = HashSet::new();
    // 다양 (명사) -> the adjective 다양하다
    let m = by_hw(vec![("다양하다", cand(11547, "adjective", 1))]);
    assert_eq!(match_word_derived(&word("다양"), &m, &none), Some(11547));
    // 간직 "사진을 간직하다": the hint's suffix names the entry among 간직되다 / 간직하다
    let m = by_hw(vec![("간직되다", cand(818, "verb", 2)), ("간직하다", cand(819, "verb", 2))]);
    assert_eq!(match_word_derived(&word("간직00"), &m, &none), Some(819));
    // 상반 "주장이 상반되다": 되다 although 하다 exists too
    let m = by_hw(vec![("상반하다", cand(25561, "verb", 3)), ("상반되다", cand(25558, "verb", 3))]);
    assert_eq!(match_word_derived(&word("상반00"), &m, &none), Some(25558));
    // 사인02 is a 동사 row: 사인하다, though the noun 사인 exists
    let m = by_hw(vec![("사인하다", cand(24924, "verb", 2))]);
    assert_eq!(match_word_derived(&word("사인02"), &m, &none), Some(24924));
    // no 하다/되다 entry, or an entry that is not a verb/adjective: nothing
    assert_eq!(match_word_derived(&word("멜로00"), &by_hw(vec![]), &none), None);
    let m = by_hw(vec![("멜로하다", cand(1, "noun", 3))]);
    assert_eq!(match_word_derived(&word("멜로00"), &m, &none), None);
    // an adverb row never takes a derived verb (마저 is not 마저하다)
    let m = by_hw(vec![("마저하다", cand(2, "verb", 3))]);
    assert_eq!(match_word_derived(&word("마저"), &m, &none), None);
    // an entry already given to an earlier row is skipped while another candidate is left
    let m = by_hw(vec![("종사하다", cand(10, "verb", 3)), ("종사하다", cand(11, "verb", 3))]);
    let claimed: HashSet<i64> = [10].into_iter().collect();
    assert_eq!(match_word_derived(&word("종사00"), &m, &claimed), Some(11));
}
