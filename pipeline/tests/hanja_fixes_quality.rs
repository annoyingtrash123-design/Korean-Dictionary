//! Guards for `pipeline/data/hanja_fixes.tsv` / `hanja_keep.tsv` (reviewed krdict hanja corrections and keeps).

use kdict_pipeline::hanja_fixes::HanjaFixes;

const FIXES: &str = include_str!("../data/hanja_fixes.tsv");
const KEEP: &str = include_str!("../data/hanja_keep.tsv");
const REVIEW: &str = include_str!("../qa/suspect_core_hanja_review.md");

fn rows(t: &str) -> Vec<Vec<&str>> {
    t.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(|l| l.split('\t').collect()).collect()
}
fn hangul(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c))
}
fn cjk(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c) || ('\u{3400}'..='\u{4DBF}').contains(&c) || ('\u{F900}'..='\u{FAFF}').contains(&c))
}

#[test]
fn rows_are_well_formed_unique_and_reviewed() {
    let mut seen = std::collections::HashSet::new();
    let mut bad = Vec::new();
    let fixes = rows(FIXES);
    for f in &fixes {
        let [id, hw, old, new, reason] = f[..] else {
            bad.push(format!("fixes: expected 5 fields: {f:?}"));
            continue;
        };
        if id.parse::<u64>().is_err() || !hangul(hw) || !cjk(old) || reason.trim().is_empty() {
            bad.push(format!("fixes: bad row {f:?}"));
        }
        if !new.is_empty() && (!cjk(new) || new == old || new.chars().count() != hw.chars().count()) {
            bad.push(format!("fixes: bad new_hanja {f:?}"));
        }
        if !seen.insert(id) {
            bad.push(format!("duplicate ext_id {id}"));
        }
        if !REVIEW.contains(&format!("### {hw} ({id}) {old} — ")) {
            bad.push(format!("fixes: {hw} ({id}) has no review section"));
        }
    }
    let keep = rows(KEEP);
    for f in &keep {
        let [id, hw, h, reason] = f[..] else {
            bad.push(format!("keep: expected 4 fields: {f:?}"));
            continue;
        };
        if id.parse::<u64>().is_err() || !hangul(hw) || !cjk(h) || reason.trim().is_empty() {
            bad.push(format!("keep: bad row {f:?}"));
        }
        if !seen.insert(id) {
            bad.push(format!("ext_id {id} is both fixed and kept (or kept twice)"));
        }
        if !REVIEW.contains(&format!("### {hw} ({id}) {h} — keep")) {
            bad.push(format!("keep: {hw} ({id}) has no keep review section"));
        }
    }
    assert!(bad.is_empty(), "{} problems\n{}", bad.len(), bad.join("\n"));
    // nothing silently dropped by the parser
    let b = HanjaFixes::builtin();
    assert_eq!((b.len(), b.keep.len()), (fixes.len(), keep.len()));
}
