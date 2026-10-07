//! Download the source datasets (idempotent; every source is optional).

use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
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
];

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
        git(&["sparse-checkout", "set", "krdict", "stdict"], Some(&dir))?;
        // make sure every blob in the sparse set is materialised (no-op when already present)
        if let Err(e) = git(&["checkout", "HEAD", "--", "krdict", "stdict"], Some(&dir)) {
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

pub fn run(work: &Path) -> Result<()> {
    fs::create_dir_all(work)?;
    if let Err(e) = fetch_nikl(work) {
        log::warn!("NIKL (krdict/stdict) fetch failed: {e:#}");
    }
    for (name, url) in FILES {
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
