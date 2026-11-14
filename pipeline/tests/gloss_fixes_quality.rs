//! Guards for `pipeline/data/gloss_fixes.tsv` (reviewed corrections to krdict English glosses).

const TSV: &str = include_str!("../data/gloss_fixes.tsv");

fn has_hangul(s: &str) -> bool {
    s.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}

#[test]
fn rows_are_well_formed_and_unique() {
    let mut seen = std::collections::HashSet::new();
    let mut bad = Vec::new();
    for (n, l) in TSV.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() != 5 {
            bad.push(format!("line {}: expected 5 tab-separated fields, got {}", n + 1, f.len()));
            continue;
        }
        let (id, sense, ko, old, new) = (f[0], f[1], f[2], f[3], f[4]);
        if id.trim().is_empty() || sense.parse::<usize>().is_err() {
            bad.push(format!("line {}: bad ext_id/sense", n + 1));
        }
        if !seen.insert((id, sense)) {
            bad.push(format!("line {}: duplicate key ({id}, {sense})", n + 1));
        }
        if [ko, old, new].iter().any(|s| s.trim().is_empty()) {
            bad.push(format!("line {}: empty field", n + 1));
        }
        if has_hangul(new) {
            bad.push(format!("line {}: Hangul left in new_gloss: {new}", n + 1));
        }
        if new == old {
            bad.push(format!("line {}: new_gloss equals old_gloss: {new}", n + 1));
        }
    }
    assert!(bad.is_empty(), "{} problems\n{}", bad.len(), bad.join("\n"));
}
