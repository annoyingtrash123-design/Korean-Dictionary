//! Source provenance: what exactly went into a build, and what changed since the previous one.
//!
//! `fetch` records HTTP metadata per download (`<work>/fetch-meta.json`); the `build` command then
//! writes `sources.json` next to `manifest.json` (git commit for NIKL; final URL, Last-Modified,
//! ETag, size and sha256 for files) and a short `sources` summary into the manifest. A later build
//! given the previous `sources.json` / `manifest.json` logs what changed ([`diff_sources`],
//! [`diff_counts`]); changes only warn, they never fail the build.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

pub const FETCH_META: &str = "fetch-meta.json";
/// A pack's count dropping by more than this fraction versus the previous build is warned about.
pub const DROP_WARN: f64 = 0.02;
/// Counts below this in the previous build are too small for a percentage to mean anything.
const MIN_PREV_COUNT: i64 = 50;

/// HTTP metadata of one download, as seen by `fetch`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct HttpMeta {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// One entry of `sources.json`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SourceRecord {
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Git sources: the checked-out commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Git sources: true when the commit was pinned (`KDICT_NIKL_REF` / `fetch::NIKL_PIN`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SourcesDoc {
    pub sources: Vec<SourceRecord>,
}

/// Merge `meta` into `<work>/fetch-meta.json` (name -> metadata).
pub fn record_fetch(work: &Path, name: &str, meta: HttpMeta) {
    let mut all = load_fetch_meta(work);
    all.insert(name.to_string(), meta);
    if let Ok(s) = serde_json::to_string_pretty(&all) {
        if let Err(e) = fs::write(work.join(FETCH_META), s) {
            log::warn!("could not write {FETCH_META}: {e}");
        }
    }
}

pub fn load_fetch_meta(work: &Path) -> BTreeMap<String, HttpMeta> {
    fs::read_to_string(work.join(FETCH_META)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn sha256_file(p: &Path) -> Result<String> {
    let mut f = fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The commit checked out in `dir`, if it is a git checkout.
pub fn git_head(dir: &Path) -> Option<String> {
    let o = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(dir).output().ok()?;
    if !o.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (s.len() >= 7 && s.bytes().all(|b| b.is_ascii_hexdigit())).then_some(s)
}

/// Provenance of every source present in `work`.
pub fn collect(work: &Path) -> SourcesDoc {
    let meta = load_fetch_meta(work);
    let mut sources = Vec::new();
    let nikl = work.join("nikl");
    if nikl.exists() {
        sources.push(SourceRecord {
            name: "nikl".into(),
            url: crate::fetch::NIKL_URL.into(),
            commit: git_head(&nikl),
            pinned: Some(crate::fetch::nikl_ref().is_some()),
            ..Default::default()
        });
    }
    let mut files: Vec<(&str, &str)> = crate::fetch::FILES.to_vec();
    files.push((crate::fetch::ENWIKT_FILE, crate::fetch::ENWIKT_URL));
    for (name, url) in files {
        let p = work.join(name);
        let Ok(md) = fs::metadata(&p) else { continue };
        if md.len() == 0 {
            continue;
        }
        let m = meta.get(name).cloned().unwrap_or_default();
        let sha256 = sha256_file(&p).map_err(|e| log::warn!("sha256 of {name}: {e}")).ok();
        sources.push(SourceRecord {
            name: name.into(),
            url: url.into(),
            final_url: m.final_url,
            last_modified: m.last_modified,
            etag: m.etag,
            bytes: Some(md.len()),
            sha256,
            ..Default::default()
        });
    }
    SourcesDoc { sources }
}

/// Short identifier per source for the manifest (`commit` or sha256 prefix).
pub fn summary(doc: &SourcesDoc) -> Value {
    let mut m = Map::new();
    for s in &doc.sources {
        let id = s.commit.clone().or_else(|| s.sha256.clone()).unwrap_or_default();
        m.insert(s.name.clone(), Value::String(id.chars().take(12).collect()));
    }
    Value::Object(m)
}

/// Write `sources.json` into `site` and a `sources` summary into its `manifest.json`.
pub fn write_site(work: &Path, site: &Path) -> Result<SourcesDoc> {
    let doc = collect(work);
    fs::write(site.join("sources.json"), serde_json::to_string_pretty(&doc)?)?;
    let mp = site.join("manifest.json");
    if let Ok(s) = fs::read_to_string(&mp) {
        let mut m: Value = serde_json::from_str(&s)?;
        m["sources"] = summary(&doc);
        fs::write(&mp, serde_json::to_string_pretty(&m)?)?;
    }
    Ok(doc)
}

fn describe(s: &SourceRecord) -> String {
    if let Some(c) = &s.commit {
        return format!("commit {}", &c[..c.len().min(12)]);
    }
    let sha = s.sha256.as_deref().map(|h| &h[..h.len().min(12)]).unwrap_or("?");
    format!("sha256 {sha}, {} bytes, last-modified {}", s.bytes.unwrap_or(0), s.last_modified.as_deref().unwrap_or("-"))
}

/// Human-readable differences between two builds' sources (empty = nothing changed).
pub fn diff_sources(prev: &SourcesDoc, cur: &SourcesDoc) -> Vec<String> {
    let mut out = Vec::new();
    for c in &cur.sources {
        match prev.sources.iter().find(|p| p.name == c.name) {
            None => out.push(format!("{}: NEW source ({})", c.name, describe(c))),
            Some(p) => {
                let changed = if c.commit.is_some() || p.commit.is_some() {
                    c.commit != p.commit
                } else if c.sha256.is_some() && p.sha256.is_some() {
                    c.sha256 != p.sha256
                } else {
                    c.etag != p.etag || c.last_modified != p.last_modified || c.bytes != p.bytes
                };
                if changed {
                    out.push(format!("{}: CHANGED {} -> {}", c.name, describe(p), describe(c)));
                }
            }
        }
    }
    for p in &prev.sources {
        if !cur.sources.iter().any(|c| c.name == p.name) {
            out.push(format!("{}: MISSING now (was {})", p.name, describe(p)));
        }
    }
    out
}

fn pack_counts(manifest: &Value) -> BTreeMap<String, i64> {
    let mut m = BTreeMap::new();
    for p in manifest.get("packs").and_then(Value::as_array).into_iter().flatten() {
        let id = p.get("id").and_then(Value::as_str).unwrap_or("?");
        for (k, v) in p.get("counts").and_then(Value::as_object).into_iter().flatten() {
            if let Some(n) = v.as_i64() {
                m.insert(format!("{id}.{k}"), n);
            }
        }
    }
    m
}

/// Warnings for every pack count that dropped by more than `max_drop` (fraction) versus the previous
/// manifest, and for packs whose `entries` count vanished.
pub fn diff_counts(prev: &Value, cur: &Value, max_drop: f64) -> Vec<String> {
    let (p, c) = (pack_counts(prev), pack_counts(cur));
    let mut out = Vec::new();
    for (k, &old) in &p {
        if old < MIN_PREV_COUNT {
            continue;
        }
        match c.get(k) {
            None => {
                if k.ends_with(".entries") {
                    out.push(format!("{k}: gone (was {old})"));
                }
            }
            Some(&new) if (old - new) as f64 > old as f64 * max_drop => {
                out.push(format!("{k}: {old} -> {new} ({:.1}% drop)", (old - new) as f64 * 100.0 / old as f64));
            }
            _ => {}
        }
    }
    out
}

/// Compare against the previous build (either input optional); logs warnings, prints a markdown
/// report and writes it to `<site>/changes.md`. Returns the report text.
pub fn report(site: &Path, cur_sources: &SourcesDoc, prev_sources: Option<&Path>, prev_manifest: Option<&Path>) -> String {
    let mut md = String::from("### Data source changes\n\n");
    let read = |p: &Path| fs::read_to_string(p).ok();
    match prev_sources.and_then(read).and_then(|s| serde_json::from_str::<SourcesDoc>(&s).ok()) {
        Some(prev) => {
            let d = diff_sources(&prev, cur_sources);
            if d.is_empty() {
                md += "No source changed since the previous deploy.\n";
            }
            for l in &d {
                log::warn!("source changed: {l}");
                md += &format!("- {l}\n");
            }
        }
        None => md += "No previous `sources.json` available: nothing to compare.\n",
    }
    md += "\n### Pack count changes\n\n";
    let cur_manifest = fs::read_to_string(site.join("manifest.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok());
    match (prev_manifest.and_then(read).and_then(|s| serde_json::from_str::<Value>(&s).ok()), cur_manifest) {
        (Some(prev), Some(cur)) => {
            let d = diff_counts(&prev, &cur, DROP_WARN);
            if d.is_empty() {
                md += &format!("No pack count dropped more than {}%.\n", DROP_WARN * 100.0);
            }
            for l in &d {
                log::warn!("pack count drop: {l}");
                md += &format!("- WARNING {l}\n");
            }
        }
        _ => md += "No previous manifest available: nothing to compare.\n",
    }
    let _ = fs::write(site.join("changes.md"), &md);
    println!("{md}");
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn file(name: &str, sha: &str) -> SourceRecord {
        SourceRecord { name: name.into(), url: "u".into(), sha256: Some(sha.into()), bytes: Some(10), ..Default::default() }
    }
    fn git(commit: &str) -> SourceRecord {
        SourceRecord { name: "nikl".into(), url: "u".into(), commit: Some(commit.into()), ..Default::default() }
    }

    #[test]
    fn unchanged_sources_give_no_diff() {
        let d = SourcesDoc { sources: vec![git("abcdef1234567"), file("a", "11")] };
        assert!(diff_sources(&d, &d.clone()).is_empty());
    }

    #[test]
    fn detects_changed_new_and_missing() {
        let prev = SourcesDoc { sources: vec![git("aaaaaaaaaaaa"), file("a", "11"), file("gone", "22")] };
        let cur = SourcesDoc { sources: vec![git("bbbbbbbbbbbb"), file("a", "12"), file("fresh", "33")] };
        let d = diff_sources(&prev, &cur);
        assert_eq!(d.len(), 4, "{d:?}");
        assert!(d.iter().any(|l| l.starts_with("nikl: CHANGED") && l.contains("bbbbbbbbbbbb")));
        assert!(d.iter().any(|l| l.starts_with("a: CHANGED")));
        assert!(d.iter().any(|l| l.starts_with("fresh: NEW")));
        assert!(d.iter().any(|l| l.starts_with("gone: MISSING")));
    }

    #[test]
    fn falls_back_to_headers_without_hash() {
        let mut a = file("a", "");
        a.sha256 = None;
        a.etag = Some("\"1\"".into());
        let mut b = a.clone();
        assert!(diff_sources(&SourcesDoc { sources: vec![a.clone()] }, &SourcesDoc { sources: vec![b.clone()] }).is_empty());
        b.etag = Some("\"2\"".into());
        assert_eq!(diff_sources(&SourcesDoc { sources: vec![a] }, &SourcesDoc { sources: vec![b] }).len(), 1);
    }

    fn manifest(core: i64, od: i64) -> Value {
        json!({"packs": [{"id": "core", "counts": {"entries": core, "tiny": 3}}, {"id": "opendict", "counts": {"entries": od}}]})
    }

    #[test]
    fn warns_only_on_drops_over_two_percent() {
        let prev = manifest(1000, 1000);
        // core -1.5% (fine), opendict -3% (warn)
        let w = diff_counts(&prev, &manifest(985, 970), 0.02);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].starts_with("opendict.entries: 1000 -> 970"));
        // growth and exactly 2% are fine
        assert!(diff_counts(&prev, &manifest(2000, 980), 0.02).is_empty());
    }

    #[test]
    fn ignores_tiny_counts_and_flags_vanished_packs() {
        let prev = manifest(1000, 1000);
        let cur = json!({"packs": [{"id": "core", "counts": {"entries": 1000, "tiny": 0}}]});
        let w = diff_counts(&prev, &cur, 0.02);
        assert_eq!(w, vec!["opendict.entries: gone (was 1000)".to_string()]);
    }

    #[test]
    fn summary_prefers_commit() {
        let d = SourcesDoc { sources: vec![git("0123456789abcdef"), file("a", "ffeeddccbbaa99887766")] };
        let s = summary(&d);
        assert_eq!(s["nikl"], "0123456789ab");
        assert_eq!(s["a"], "ffeeddccbbaa");
    }

    #[test]
    fn sources_json_roundtrip_and_manifest_summary() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("manifest.json"), r#"{"version":"v","packs":[]}"#).unwrap();
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("kengdic.tsv"), "abc").unwrap();
        record_fetch(work.path(), "kengdic.tsv", HttpMeta { url: "u".into(), etag: Some("e".into()), ..Default::default() });
        let doc = write_site(work.path(), dir.path()).unwrap();
        let back: SourcesDoc = serde_json::from_str(&fs::read_to_string(dir.path().join("sources.json")).unwrap()).unwrap();
        assert_eq!(doc, back);
        assert_eq!(back.sources[0].etag.as_deref(), Some("e"));
        assert_eq!(back.sources[0].bytes, Some(3));
        let m: Value = serde_json::from_str(&fs::read_to_string(dir.path().join("manifest.json")).unwrap()).unwrap();
        assert_eq!(m["version"], "v");
        assert!(m["sources"]["kengdic.tsv"].is_string());
    }
}
