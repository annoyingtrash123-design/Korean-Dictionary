//! Guards for `pipeline/data/example_en.tsv` (AI English for krdict example sentences).
//! Each check here is a mistake that actually happened in review and was fixed in bulk; the test
//! keeps it from coming back with the next batch (CLAUDE.md: every recurring mistake becomes a check).

const TSV: &str = include_str!("../data/example_en.tsv");

/// Recurring krdict character names -> accepted English romanisations (lower-case, no hyphen).
const NAMES: &[(&str, &[&str])] = &[
    ("지수", &["jisu", "jisoo"]),
    ("승규", &["seunggyu", "seungkyu"]),
    ("민준", &["minjun", "minjoon"]),
    ("유민", &["yumin"]),
    ("영수", &["yeongsu", "youngsu", "yeongsoo", "youngsoo"]),
    ("민수", &["minsu", "minsoo"]),
    ("지영", &["jiyeong", "jiyoung"]),
];
/// Particles after a name or a pronoun: "승규는", "그에게", "그 사람"(space) …
const PARTICLES: &[&str] = &["이는", "이가", "이의", "이를", "이와", "이도", "이에게", "는", "가", "와", "과", "에게", "의", "도", "를", "을", "랑", "하고", "야", "씨", "한테", " "];
const HE: &[&str] = &["he", "him", "his", "himself", "man", "men", "boy", "gentleman"];
const SHE: &[&str] = &["she", "her", "hers", "herself", "woman", "women", "girl", "lady", "female"];

/// Lower-case English words with hyphens and apostrophes dropped ("Seung-gyu's" -> "seunggyus", "seunggyu").
fn words(en: &str) -> Vec<String> {
    let mut out = Vec::new();
    for w in en.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '\'' || c == '’')) {
        let w: String = w.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '\'' || *c == '’').collect::<String>().to_ascii_lowercase();
        if w.is_empty() {
            continue;
        }
        if let Some((stem, _)) = w.split_once(['\'', '’']) {
            out.push(stem.to_string());
        }
        out.push(w.replace(['\'', '’'], ""));
    }
    out
}

/// `word` used as a word of its own in Korean: at the start or after a space, followed by a particle.
fn ko_word(ko: &str, word: &str) -> bool {
    ko.match_indices(word).any(|(i, _)| {
        let before_ok = i == 0 || ko[..i].ends_with([' ', '‘', '“', '"', '(']);
        let rest = &ko[i + word.len()..];
        before_ok && PARTICLES.iter().any(|p| rest.starts_with(p))
    })
}

/// Hangul syllables left untranslated (lone jamo such as ‘ㅎ’ are fine: sentences about the alphabet).
fn has_hangul(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

struct Row<'a> {
    line: usize,
    key: (&'a str, &'a str, &'a str),
    ko: &'a str,
    en: &'a str,
}

fn rows() -> Vec<Row<'static>> {
    TSV.lines()
        .enumerate()
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
        .map(|(n, l)| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 5, "line {}: expected 5 tab-separated fields", n + 1);
            Row { line: n + 1, key: (f[0], f[1], f[2]), ko: f[3], en: f[4] }
        })
        .collect()
}

/// Every problem in one message (at most 40 shown), so a bad batch is fixed in one pass.
fn report(what: &str, bad: Vec<String>) {
    assert!(bad.is_empty(), "{} rows: {what}\n{}", bad.len(), bad.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
}

#[test]
fn rows_are_well_formed_and_unique() {
    let rs = rows();
    assert!(rs.len() > 40_000, "example_en.tsv has only {} rows", rs.len());
    let mut seen = std::collections::HashSet::new();
    let mut bad = Vec::new();
    for r in &rs {
        if !seen.insert(r.key) {
            bad.push(format!("line {}: duplicate key {:?}", r.line, r.key));
        }
        if r.en.trim().is_empty() || r.ko.trim().is_empty() {
            bad.push(format!("line {}: empty field", r.line));
        }
        if has_hangul(r.en) {
            bad.push(format!("line {}: Hangul left in the English: {}", r.line, r.en));
        }
    }
    report("malformed", bad);
}

#[test]
fn names_match_the_korean() {
    let mut bad = Vec::new();
    for r in rows() {
        let ko = r.ko.replace("영수증", ""); // 영수증 "receipt" is not the name 영수
        let ws = words(r.en);
        for (name, en_forms) in NAMES {
            let in_en = ws.iter().any(|w| en_forms.contains(&w.as_str()));
            if in_en && !ko.contains(name) {
                bad.push(format!("line {}: English names {name}, the Korean doesn't: {} | {}", r.line, r.ko, r.en));
            }
            // 지수 is also "index"/"exponent", so only checked one way
            if *name != "지수" && ko_word(&ko, name) && !in_en {
                bad.push(format!("line {}: Korean has {name}, the English lacks it: {} | {}", r.line, r.ko, r.en));
            }
        }
    }
    report("name mismatch (invented, dropped or substituted names)", bad);
}

#[test]
fn he_and_she_pronouns_are_kept() {
    let mut bad = Vec::new();
    for r in rows() {
        let ws = words(r.en);
        let any = |set: &[&str]| ws.iter().any(|w| set.contains(&w.as_str()));
        if ["그는", "그가", "그의", "그를", "그에게", "그와", "그도", "그한테"].iter().any(|p| ko_word(r.ko, &p[..3]) && r.ko.contains(p)) && !any(HE) {
            bad.push(format!("line {}: 그 (he) without he/him/his: {} | {}", r.line, r.ko, r.en));
        }
        if r.ko.contains("그녀") && !any(SHE) {
            bad.push(format!("line {}: 그녀 (she) without she/her: {} | {}", r.line, r.ko, r.en));
        }
    }
    report("pronoun 그/그녀 not rendered as he/she", bad);
}

#[test]
fn the_checks_catch_known_mistakes() {
    // names
    assert!(words("Seung-gyu's bag").contains(&"seunggyu".to_string()));
    assert!(ko_word("승규는 웃었다.", "승규"));
    assert!(!ko_word("영수하였습니다", "영수"));
    // pronouns: the 'they' rendering that slipped through review must fail
    let ws = words("They counted the money in their wallet.");
    assert!(!ws.iter().any(|w| HE.contains(&w.as_str())));
    assert!(ko_word("그는 지갑 속에 있는 돈을 헤아려 보았다.", "그"));
    assert!(!ko_word("그때 비가 왔다.", "그"));
}
