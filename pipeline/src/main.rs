
use anyhow::Result;
use kdict_pipeline::{build, fetch, pack};
use kdict_pipeline::{texts_fetch, texts_news};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Info
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            let lvl = match r.level() {
                log::Level::Warn => "WARNING",
                log::Level::Error => "ERROR",
                _ => "INFO",
            };
            eprintln!("{lvl}: {}", r.args());
        }
    }
    fn flush(&self) {}
}

#[derive(Parser)]
#[command(name = "kdict-pipeline", about = "Build the offline Korean-English dictionary data packs")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download source datasets into the work directory (idempotent, every source optional)
    Fetch {
        #[arg(long, default_value = "pipeline/.work")]
        work: PathBuf,
    },
    /// Parse the sources and produce core.sqlite, stdict.sqlite and site-data/
    Build {
        #[arg(long, default_value = "pipeline/.work")]
        work: PathBuf,
        #[arg(long, default_value = "pipeline/out")]
        out: PathBuf,
        /// Only read the first N krdict and N stdict XML files (for quick test builds)
        #[arg(long)]
        limit_files: Option<usize>,
        /// Do not fail when the result looks degraded (core < 150k entries, krdict < 50k, wikt < 20k,
        /// sentences < 10k, hanja_chars < 5k, stdict < 400k). For local partial builds only; CI omits it.
        #[arg(long)]
        allow_partial: bool,
    },
    /// Reader: resolve the text catalogue against Wikisource etc. and write raw texts + provenance
    FetchTexts {
        #[arg(long, default_value = "pipeline/texts/catalog.toml")]
        catalog: PathBuf,
        #[arg(long, default_value = "pipeline/texts/raw")]
        out: PathBuf,
        /// Only these catalogue ids (comma-separated)
        #[arg(long, value_delimiter = ',')]
        only: Vec<String>,
        /// Re-fetch entries that already have a raw file
        #[arg(long)]
        force: bool,
        /// Maximum number of subpages fetched per work
        #[arg(long, default_value_t = 150)]
        max_subpages: usize,
    },
    /// Reader: recent 정책브리핑 (korea.kr) articles marked 공공누리 제1유형
    FetchNews {
        #[arg(long, default_value = "pipeline/texts/catalog.toml")]
        catalog: PathBuf,
        #[arg(long, default_value = "pipeline/texts/raw/news")]
        out: PathBuf,
        /// Stop after this many KOGL type 1 articles
        #[arg(long, default_value_t = 30)]
        max: usize,
        #[arg(long)]
        force: bool,
        /// RSS feed URL (repeatable); overrides [news] feeds in the catalogue
        #[arg(long)]
        feed: Vec<String>,
    },
}

fn main() -> Result<()> {
    log::set_logger(&Logger).ok();
    log::set_max_level(log::LevelFilter::Info);
    match Cli::parse().cmd {
        Cmd::Fetch { work } => fetch::run(&work),
        Cmd::Build { work, out, limit_files, allow_partial } => {
            let sources = build::Sources::discover(&work, limit_files);
            build::run(&build::BuildOpts { out, sources, chunk_bytes: pack::CHUNK_BYTES, allow_partial })?;
            Ok(())
        }
        Cmd::FetchTexts { catalog, out, only, force, max_subpages } => texts_fetch::run(&texts_fetch::Opts { catalog, out, only, force, max_subpages }),
        Cmd::FetchNews { catalog, out, max, force, feed } => texts_news::run(&texts_news::NewsOpts { catalog, out, max, force, feeds: feed }),
    }
}
