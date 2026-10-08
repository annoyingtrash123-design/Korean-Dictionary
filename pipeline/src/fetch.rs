//! Download the source datasets (idempotent; every source is optional).

use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub const NIKL_URL: &str = "https://github.com/spellcheck-ko/korean-dict-nikl";
pub const FILES: &[(&str, &str)] = &[
    ("kaikki-ko.jsonl", "https://kaikki.org/dictionary/Korean/kaikki.org-dictionary-Korean.jsonl"),
    ("kengdic.tsv", "https://raw.githubusercontent.com/garfieldnate/kengdic/master/kengdic.tsv"),
    ("ko_50k.txt", "https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/ko/ko_50k.txt"),
    ("Unihan.zip", "https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip"),
    ("tatoeba/kor_sentences.tsv.bz2", "https://downloads.tatoeba.org/exports/per_language/kor/kor_sentences.tsv.bz2"),
    ("tatoeba/eng_sentences.tsv.bz2", "https://downloads.tatoeba.org/exports/per_language/eng/eng_sentences.tsv.bz2"),
    ("tatoeba/links.tar.bz2", "https://downloads.tatoeba.org/exports/links.tar.bz2"),
    // optional Chinese packs (no verified GitHub mirror of CC-CEDICT exists; mdbg.net works from CI)
    ("cedict.zip", "https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.zip"),
    // multi-GB; skipped when KDICT_SKIP_ZHWIKT is set (the zhwikt pack is then not built)
    ("kaikki-zh.jsonl", "https://kaikki.org/dictionary/Chinese/kaikki.org-dictionary-Chinese.jsonl"),
];

/// English Wiktionary extract (several GB). `fetch` streams it and keeps only the lines that
/// mention Korean ([`crate::enwikt::mentions_korean`]), so the work directory and the CI source
/// cache hold a few hundred MB at most. Skipped when `KDICT_SKIP_ENWIKT` is set.
pub const ENWIKT_FILE: &str = "kaikki-en-ko.jsonl";
pub const ENWIKT_URL: &str = "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl";

/// Copy the lines of `rd` for which `keep` holds into `out`; returns (lines read, lines kept).
pub fn filter_lines<R: Read, W: Write>(rd: R, out: &mut W, keep: fn(&[u8]) -> bool) -> Result<(u64, u64)> {
    let mut br = BufReader::with_capacity(1 << 20, rd);
    let (mut n, mut k) = (0u64, 0u64);
    let mut line = Vec::new();
    loop {
        line.clear();
        if br.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        n += 1;
        if keep(&line) {
            out.write_all(&line)?;
            if !line.ends_with(b"\n") {
                out.write_all(b"\n")?;
            }
            k += 1;
        }
    }
    out.flush()?;
    Ok((n, k))
}

/// Stream `url` through [`filter_lines`] into `dest` (ureq, then curl as a fallback).
pub fn download_filtered(url: &str, dest: &Path, keep: fn(&[u8]) -> bool) -> Result<(u64, u64)> {
    if let Some(d) = dest.parent() {
        fs::create_dir_all(d)?;
    }
    let tmp = dest.with_extension("part");
    let via_ureq = || -> Result<(u64, u64)> {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(30))
            .timeout_read(Duration::from_secs(120))
            .user_agent("kdict-pipeline/0.1 (+https://github.com/)")
            .try_proxy_from_env(true)
            .build();
        let resp = agent.get(url).call().map_err(|e| anyhow!("{e}"))?;
        let mut f = io::BufWriter::new(fs::File::create(&tmp)?);
        filter_lines(resp.into_reader(), &mut f, keep)
    };
    let via_curl = || -> Result<(u64, u64)> {
        let mut child = Command::new("curl").args(["-fsSL", "--retry", "2"]).arg(url).stdout(Stdio::piped()).spawn()?;
        let mut f = io::BufWriter::new(fs::File::create(&tmp)?);
        let r = filter_lines(child.stdout.take().ok_or_else(|| anyhow!("no curl stdout"))?, &mut f, keep);
        let st = child.wait()?;
        if !st.success() {
            return Err(anyhow!("curl exit {st}"));
        }
        r
    };
    let res = match via_ureq() {
        Ok(r) if r.1 > 0 => Ok(r),
        first => {
            let first = match first {
                Ok(_) => "no matching lines".to_string(),
                Err(e) => e.to_string(),
            };
            log::debug!("filtered download via ureq failed for {url}: {first}");
            match via_curl() {
                Ok(r) if r.1 > 0 => Ok(r),
                Ok(_) => Err(anyhow!("{first}; curl fallback: no matching lines")),
                Err(e) => Err(anyhow!("{first}; curl fallback: {e}")),
            }
        }
    };
    match res {
        Ok(r) => {
            fs::rename(&tmp, dest)?;
            Ok(r)
        }
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

fn download_ureq(url: &str, tmp: &Path) -> Result<u64> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(120))
        .user_agent("kdict-pipeline/0.1 (+https://github.com/)")
        .try_proxy_from_env(true)
        .build();
    let resp = agent.get(url).call().map_err(|e| anyhow!("{e}"))?;
    let mut rd = resp.into_reader();
    let mut f = fs::File::create(tmp)?;
    Ok(io::copy(&mut rd.by_ref(), &mut f)?)
}

fn download_curl(url: &str, tmp: &Path) -> Result<u64> {
    let st = Command::new("curl").args(["-fsSL", "--retry", "2", "-o"]).arg(tmp).arg(url).status()?;
    if !st.success() {
        return Err(anyhow!("curl exit {st}"));
    }
    Ok(fs::metadata(tmp)?.len())
}

pub fn download(url: &str, dest: &Path) -> Result<()> {
    if let Some(d) = dest.parent() {
        fs::create_dir_all(d)?;
    }
    let tmp = dest.with_extension("part");
    let mut last = None;
    for attempt in 0..2 {
        match download_ureq(url, &tmp) {
            Ok(n) if n > 0 => {
                fs::rename(&tmp, dest)?;
                return Ok(());
            }
            Ok(_) => last = Some(anyhow!("empty download")),
            Err(e) => last = Some(e),
        }
        log::debug!("download attempt {attempt} failed for {url}: {:#}", last.as_ref().unwrap());
    }
    // fall back to curl (honours the CA bundle / proxy of the host environment)
    match download_curl(url, &tmp) {
        Ok(n) if n > 0 => {
            fs::rename(&tmp, dest)?;
            Ok(())
        }
        other => {
            let _ = fs::remove_file(&tmp);
            let curl = match other {
                Ok(_) => "empty download".to_string(),
                Err(e) => e.to_string(),
            };
            let first = last.map(|e| e.to_string()).unwrap_or_else(|| "download failed".into());
            Err(anyhow!("{first}; curl fallback: {curl}"))
        }
    }
}

fn git(args: &[&str], dir: Option<&Path>) -> Result<()> {
    let mut c = Command::new("git");
    // avoid slow auto-gc and interactive prompts
    c.args(["-c", "gc.auto=0"]).args(args).env("GIT_TERMINAL_PROMPT", "0");
    if let Some(d) = dir {
        c.current_dir(d);
    }
    let st = c.status().map_err(|e| anyhow!("running git: {e}"))?;
    if st.success() {
        Ok(())
    } else {
        Err(anyhow!("git {} failed: {st}", args.join(" ")))
    }
}

pub fn fetch_nikl(work: &Path) -> Result<()> {
    let dir = work.join("nikl");
    if !dir.join(".git").exists() && !dir.join("krdict").exists() {
        git(&["clone", "--depth", "1", "--filter=blob:none", "--sparse", NIKL_URL, &dir.to_string_lossy()], None)?;
    }
    if dir.join(".git").exists() {
        git(&["sparse-checkout", "set", "krdict", "stdict", "opendict"], Some(&dir))?;
        // make sure every blob in the sparse set is materialised (no-op when already present)
        if let Err(e) = git(&["checkout", "HEAD", "--", "krdict", "stdict", "opendict"], Some(&dir)) {
            log::warn!("NIKL checkout: {e:#}");
        }
        // idempotent refresh; ignore failure (offline reruns keep what we have)
        if let Err(e) = git(&["pull", "--ff-only", "--depth", "1"], Some(&dir)) {
            log::debug!("NIKL pull skipped: {e:#}");
        }
    }
    let n = |d: &str| fs::read_dir(dir.join(d)).map(|r| r.count()).unwrap_or(0);
    if n("krdict") == 0 {
        return Err(anyhow!("no krdict files after checkout"));
    }
    log::info!("NIKL: {} krdict files, {} stdict files", n("krdict"), n("stdict"));
    Ok(())
}

/// The English Wiktionary extract, cut to lines mentioning Korean (optional; failures only warn).
fn fetch_enwikt(work: &Path) {
    let dest = work.join(ENWIKT_FILE);
    if std::env::var_os("KDICT_SKIP_ENWIKT").is_some() {
        log::info!("{ENWIKT_FILE}: KDICT_SKIP_ENWIKT is set, skipping");
    } else if fs::metadata(&dest).map(|m| m.len() > 0).unwrap_or(false) {
        log::info!("{ENWIKT_FILE}: already present, skipping");
    } else {
        log::info!("downloading {ENWIKT_URL} (keeping lines that mention Korean)");
        match download_filtered(ENWIKT_URL, &dest, crate::enwikt::mentions_korean) {
            Ok((n, k)) => log::info!("{ENWIKT_FILE}: kept {k} of {n} lines, {} bytes", fs::metadata(&dest).map(|m| m.len()).unwrap_or(0)),
            Err(e) => log::warn!("{ENWIKT_FILE}: download failed ({ENWIKT_URL}): {e:#}"),
        }
    }
}

pub fn run(work: &Path) -> Result<()> {
    fs::create_dir_all(work)?;
    if let Err(e) = fetch_nikl(work) {
        log::warn!("NIKL (krdict/stdict/opendict) fetch failed: {e:#}");
    }
    fetch_enwikt(work);
    for (name, url) in FILES {
        if *name == "kaikki-zh.jsonl" && std::env::var_os("KDICT_SKIP_ZHWIKT").is_some() {
            log::info!("{name}: KDICT_SKIP_ZHWIKT is set, skipping");
            continue;
        }
        let dest = work.join(name);
        if dest.exists() && fs::metadata(&dest).map(|m| m.len() > 0).unwrap_or(false) {
            log::info!("{name}: already present, skipping");
            continue;
        }
        log::info!("downloading {url}");
        match download(url, &dest) {
            Ok(()) => log::info!("{name}: {} bytes", fs::metadata(&dest).map(|m| m.len()).unwrap_or(0)),
            Err(e) => log::warn!("{name}: download failed ({url}): {e:#}"),
        }
    }
    Ok(())
}
