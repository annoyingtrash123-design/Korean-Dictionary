//! Link quality of the "Writing & usage" lists, where a pipeline-built core pack is unpacked locally (`KDICT_PACKS` or
//! `pipeline/out`): the grammar rows and headwords the AI-written data points at must exist. Skipped without the pack
//! (CI's verify job has only the gzip chunks; the build itself logs every unresolved link).

use kdict_pipeline::usage_lists::*;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

fn core() -> Option<Connection> {
    let dir = std::env::var("KDICT_PACKS").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("out"));
    let p = dir.join("core.sqlite");
    p.exists().then(|| Connection::open_with_flags(p, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap())
}

#[test]
fn writing_links_and_confusable_words_resolve_in_the_real_pack() {
    let Some(conn) = core() else {
        eprintln!("skipped: no unpacked core.sqlite");
        return;
    };
    let l = Linker::new(&conn).unwrap();
    let writing = builtin_writing();
    let asked: Vec<_> = writing.iter().filter(|r| !r.link.is_empty()).collect();
    let missing: Vec<String> = asked.iter().filter(|r| l.link(&r.link).is_none()).map(|r| format!("{} {}", r.id, r.link)).collect();
    assert!(missing.is_empty(), "{} of {} links of writing_expressions.tsv do not resolve: {missing:?}", missing.len(), asked.len());
    assert!(asked.len() * 100 >= writing.len() * 60, "only {} of {} rows link to an entry", asked.len(), writing.len());

    let mut words = 0;
    let mut missing = Vec::new();
    for r in builtin_confusables() {
        for w in r.words.iter().filter(|w| !w.wrong) {
            words += 1;
            if l.confusable_word(&w.text).is_none() {
                missing.push(format!("{} {}", r.id, w.text));
            }
        }
    }
    // a few look-alike forms (colloquial -데 / -대, spelling variants) are no headword: at most one in ten
    assert!(missing.len() * 10 <= words, "{} of {words} confusable words have no entry: {missing:?}", missing.len());
}
