//! Search-quality benchmark: runs `tests/golden_queries.tsv` (format and how to run: see the
//! comment at the top of that file) against the real, unpacked packs and prints one line per
//! check and an overall score. Fails only when a check that is not marked as a known failure
//! fails, so a regression is caught while known engine weaknesses are just reported.

use kdict_core::hangul::norm_headword;
use kdict_core::search::{search, PackDb, ResultRow};
use kdict_core::sql::Conn;
use std::path::{Path, PathBuf};

/// Packs searched unless `$KDICT_GOLDEN_PACKS` says otherwise: the app's default selection.
const DEFAULT_PACKS: &str = "core,stdict";

struct Check {
    line: usize,
    query: String,
    kind: Kind,
    n: usize,
    /// alternatives: (headword, bare hanja or "")
    want: Vec<(String, String)>,
    known: bool,
    note: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// one of the headwords is among the first `n` distinct headwords
    Top,
    /// a row among the first `n` rows has the headword with exactly this hanja
    Hanja,
    /// no row among the first `n` rows has the headword with this hanja
    NotHanja,
}

fn bare_han(s: &str) -> String {
    s.chars().filter(|&c| kdict_core::hangul::is_han(c)).collect()
}

fn parse(text: &str) -> Vec<Check> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        assert!(f.len() >= 3, "golden_queries.tsv:{}: want query<TAB>check<TAB>expected[<TAB>status<TAB>note]", i + 1);
        let check = f[1].trim();
        let (kind, n) = if check == "first" {
            (Kind::Top, 1)
        } else if let Some(n) = check.strip_prefix("top") {
            (Kind::Top, n.parse().unwrap_or_else(|_| panic!("line {}: bad check {check}", i + 1)))
        } else if let Some(n) = check.strip_prefix("nothanja") {
            (Kind::NotHanja, n.parse().unwrap_or_else(|_| panic!("line {}: bad check {check}", i + 1)))
        } else if let Some(n) = check.strip_prefix("hanja") {
            (Kind::Hanja, n.parse().unwrap_or_else(|_| panic!("line {}: bad check {check}", i + 1)))
        } else {
            panic!("golden_queries.tsv:{}: unknown check '{check}'", i + 1)
        };
        let want = f[2]
            .split('|')
            .map(|alt| match alt.split_once('=') {
                Some((hw, h)) => (norm_headword(hw.trim()), bare_han(h)),
                None => (norm_headword(alt.trim()), String::new()),
            })
            .collect();
        let status = f.get(3).map(|s| s.trim()).unwrap_or("");
        assert!(matches!(status, "" | "ok" | "known"), "line {}: status must be ok or known", i + 1);
        out.push(Check {
            line: i + 1,
            query: f[0].to_string(),
            kind,
            n,
            want,
            known: status == "known",
            note: f.get(4).map(|s| s.trim().to_string()).unwrap_or_default(),
        });
    }
    out
}

/// Distinct headwords in result order (the app groups rows of one word).
fn headwords(rows: &[ResultRow]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for r in rows {
        let h = norm_headword(&r.headword);
        if !v.contains(&h) {
            v.push(h);
        }
    }
    v
}

fn passes(c: &Check, rows: &[ResultRow]) -> bool {
    match c.kind {
        Kind::Top => {
            let hw = headwords(rows);
            hw.iter().take(c.n).any(|h| c.want.iter().any(|(w, _)| w == h))
        }
        Kind::Hanja | Kind::NotHanja => {
            let hit = rows.iter().take(c.n).any(|r| {
                // a row may give variants: 沙果/砂果
                let shown = r.hanja.as_deref().unwrap_or("");
                let hs: Vec<String> = shown.split('/').map(bare_han).chain([bare_han(shown)]).collect();
                c.want.iter().any(|(w, wh)| *w == norm_headword(&r.headword) && hs.contains(wh))
            });
            hit == (c.kind == Kind::Hanja)
        }
    }
}

fn packs_dir() -> PathBuf {
    std::env::var_os("KDICT_PACKS")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../pipeline/out"))
}

#[test]
#[ignore = "needs the unpacked real packs: KDICT_PACKS=<dir with core.sqlite, stdict.sqlite>"]
fn golden_queries() {
    let dir = packs_dir();
    let ids = std::env::var("KDICT_GOLDEN_PACKS").unwrap_or_else(|_| DEFAULT_PACKS.to_string());
    let mut packs = Vec::new();
    for id in ids.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let path = dir.join(format!("{id}.sqlite"));
        let conn = Conn::open_path(path.to_str().unwrap(), true).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        packs.push(PackDb::new(id, conn).unwrap());
    }
    let refs: Vec<&PackDb> = packs.iter().collect();
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden_queries.tsv");
    let checks = parse(&std::fs::read_to_string(&file).unwrap());

    let (mut pass, mut regressions, mut fixed, mut known) = (0, Vec::new(), Vec::new(), 0);
    for c in &checks {
        let rows = search(&refs, &c.query, None).unwrap().rows;
        let ok = passes(c, &rows);
        let tag = match (ok, c.known) {
            (true, false) => "pass ",
            (true, true) => "FIXED",
            (false, true) => "known",
            (false, false) => "FAIL ",
        };
        if ok {
            pass += 1;
        }
        if c.known && !ok {
            known += 1;
        }
        let top: Vec<String> = rows
            .iter()
            .take(6)
            .map(|r| match r.hanja.as_deref().filter(|h| !h.is_empty()) {
                Some(h) => format!("{}({h})", r.headword),
                None => r.headword.clone(),
            })
            .collect();
        let check = match c.kind {
            Kind::Top if c.n == 1 => "first".to_string(),
            Kind::Top => format!("top{}", c.n),
            Kind::Hanja => format!("hanja{}", c.n),
            Kind::NotHanja => format!("nothanja{}", c.n),
        };
        let want: Vec<String> =
            c.want.iter().map(|(w, h)| if h.is_empty() { w.clone() } else { format!("{w}={h}") }).collect();
        println!("{tag} {:<22} {check:<9} {:<24} | {}", c.query, want.join("|"), top.join(", "));
        if !ok && !c.known {
            regressions.push(format!("line {}: {} {check} {} ({})", c.line, c.query, want.join("|"), c.note));
        }
        if ok && c.known {
            fixed.push(format!("line {}: {}", c.line, c.query));
        }
    }
    let total = checks.len();
    println!(
        "\nscore {pass}/{total} ({:.1}%) with packs {ids}; {known} known failures, {} fixed, {} regressions",
        100.0 * pass as f64 / total as f64,
        fixed.len(),
        regressions.len()
    );
    if !fixed.is_empty() {
        println!("now passing (drop their 'known' mark):\n  {}", fixed.join("\n  "));
    }
    assert!(regressions.is_empty(), "checks that used to pass now fail:\n  {}", regressions.join("\n  "));
}
