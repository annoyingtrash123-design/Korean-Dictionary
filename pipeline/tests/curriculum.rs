//! Guards for the NIKL curriculum level lists (`data/curriculum_*.tsv`) and the matcher that links
//! them to krdict entries and grammar rows. The matcher cases are real rows of the lists, checked
//! by hand against krdict (ids/homonyms as in the built core pack).

use kdict_pipeline::curriculum::*;
use std::collections::HashSet;

#[test]
fn tsvs_parse_with_expected_shape() {
    let words = builtin_words();
    let grammar = builtin_grammar();
    assert_eq!(words.len(), 10_635);
    assert_eq!(grammar.len(), 336);
    for w in &words {
        assert!((1..=6).contains(&w.level), "{w:?}");
        assert!(!w.form_raw.is_empty() && !w.headword.is_empty() && !w.pos.is_empty(), "{w:?}");
        assert!(!w.headword.chars().any(|c| c.is_ascii_digit() || c == '/' || c == '∙'), "{w:?}");
    }
    for g in &grammar {
        assert!((1..=6).contains(&g.level), "{g:?}");
        assert!(!g.form_raw.is_empty() && !g.form.is_empty() && !g.category.is_empty(), "{g:?}");
        assert!(!g.form.chars().any(|c| c.is_ascii_digit() || c == '\u{ad}'), "{g:?}");
    }
    // ranks are the workbook's 전체 번호: unique and 1..=N
    let ranks: HashSet<i64> = words.iter().map(|w| w.rank).collect();
    assert_eq!(ranks.len(), words.len());
    assert_eq!(words.iter().map(|w| w.level).filter(|l| *l == 6).count(), 2580);
}

fn cand(id: i64, hom: i64, pos: &str, level: i64, text: &str) -> WordCand {
    WordCand { id, homonym: Some(hom), pos: pos.into(), level: Some(level), text: text.into() }
}

fn word(form_raw: &str) -> CurWord {
    builtin_words().into_iter().find(|w| w.form_raw == form_raw).unwrap_or_else(|| panic!("{form_raw} not in list"))
}

#[test]
fn words_match_by_pos_hint_and_level() {
    let none = HashSet::new();
    // 가다01 (동사) vs the auxiliary verb 가다2
    let go = vec![cand(99, 1, "verb", 1, "장소를 이동하다 학교에 가다"), cand(107, 2, "auxiliary verb", 2, "고기가 다 익어 가니까")];
    assert_eq!(match_word(&word("가다01"), &go, &none), Some(99));
    // 가격02 "가격이 비싸다": krdict numbers the price sense 2, the blow 1; level and hint decide
    let price = vec![cand(20, 1, "noun", 3, "손이나 주먹으로 치거나 때림 급소 가격"), cand(21, 2, "noun", 1, "물건의 가치를 돈으로 나타낸 것 가격이 비싸다")];
    assert_eq!(match_word(&word("가격02"), &price, &none), Some(21));
    // homonyms with the same POS: the hint picks the boat (배02 "배를 타다"), not the stomach or the pear
    let bae = vec![
        cand(1, 1, "noun", 1, "배가 고프다 사람의 배"),
        cand(2, 2, "noun", 1, "사람이나 짐을 싣고 물 위를 다니는 것 배를 타다"),
        cand(3, 3, "noun", 1, "배나무의 열매 배를 먹다"),
    ];
    assert_eq!(match_word(&word("배02"), &bae, &none), Some(2));
    assert_eq!(match_word(&word("배03"), &bae, &none), Some(3));
    // a second list row of the same headword takes the entry the first one did not
    let claimed: HashSet<i64> = [8].into_iter().collect();
    let plain = vec![cand(7, 1, "noun", 1, ""), cand(8, 2, "noun", 1, "")];
    let mut w = word("배02");
    w.hint = String::new();
    assert_eq!(match_word(&w, &plain, &claimed), Some(7));
    // wrong POS with several candidates and no hint support: no match
    let adv = vec![cand(5, 1, "adverb", 1, ""), cand(6, 2, "interjection", 1, "")];
    assert_eq!(match_word(&word("가다01"), &adv, &none), None);
}

fn g(id: i64, pattern: &str, category: &str) -> GrammarCand {
    GrammarCand { id, entry_id: Some(1000 + id), pattern: pattern.into(), category: category.into() }
}

fn grammar(rank: i64) -> CurGrammar {
    builtin_grammar().into_iter().find(|g| g.rank == rank).unwrap()
}

#[test]
fn grammar_keys_normalise() {
    assert_eq!(grammar_key("-ㄴ 데다가"), "ㄴ데다가");
    assert_eq!(grammar_key("-다가1(2)"), "다가");
    assert_eq!(grammar_key("에서부터(서부터)"), "에서부터");
    assert_eq!(grammar_key("\u{ad}으려고2"), "으려고");
}

#[test]
fn grammar_matches_forms_and_related_forms() {
    let rows = vec![
        g(1, "다가", "Particles"),
        g(2, "-다가", "Connective endings"),
        g(3, "-ㄴ 데다가", "Expressions"),
        g(4, "-는 데다가", "Expressions"),
        g(5, "-ㄹ 만하다", "Expressions"),
        g(6, "-을 만하다", "Expressions"),
        g(7, "와/과", "Particles"),
        g(8, "-과", "Affixes"),
        g(9, "-으려고", "Connective endings"),
        g(10, "으로 인하다", "Expressions"),
        g(11, "-더라도", "Connective endings"),
    ];
    let idx = grammar_index(&rows);
    let m = |rank| match_grammar(&grammar(rank), &rows, &idx).map(|i| rows[i].id);
    // 5급 표현 "-는 데다가" (related -ㄴ데다가 …): the main form wins
    assert_eq!(m(258), Some(4));
    // 5급 표현 "-을 만하다" is listed as related "-ㄹ 만하다" of the main "-을 만하다"
    assert_eq!(m(267), Some(6));
    // 2급 연결어미 "-다가1(1)" must not land on the particle "다가"
    assert_eq!(m(61), Some(2));
    // the related form "-ㄹ 만하다" alone would also find it
    let mut only_related = grammar(267);
    only_related.form_raw = "-을 만해야".into();
    assert_eq!(match_grammar(&only_related, &rows, &idx).map(|i| rows[i].id), Some(5));
    // 6급 조사 "이라면" has no row: no match, and an affix "-과" is never a grammar match
    let mut gwa = grammar(1);
    gwa.form_raw = "과".into();
    gwa.category = "조사".into();
    assert_eq!(match_grammar(&gwa, &rows, &idx).map(|i| rows[i].id), Some(7));
    // 4급 "으로 인하여" -> "으로 인하다"
    assert_eq!(m(213), Some(10));
    // soft hyphen in the source ("\u{ad}으려고2")
    assert_eq!(m(238), Some(9));
}
