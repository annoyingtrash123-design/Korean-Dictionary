//! Pack storage on the browser: SQLite (sqlite-wasm-rs, bundled with FTS5) over the OPFS
//! "sahpool" VFS (sqlite-wasm-vfs). The sahpool VFS uses synchronous access handles inside a
//! dedicated worker, so it works on iOS Safari 16.4+ without COOP/COEP headers.
//!
//! Each pack (`core`, `stdict`) is one database file in the pool (`<pack>.sqlite3`). A tiny
//! `_installed.sqlite3` database records which version of each pack was installed.

use crate::search::PackDb;
use crate::sql::Conn;
use rsqlite_vfs::VfsFilesManager;
use sqlite_wasm_rs::WasmOsCallback;
use sqlite_wasm_vfs::sahpool::{install, OpfsSAHPoolCfgBuilder, OpfsSAHPoolUtil};

pub const VFS_NAME: &str = "kdict-sahpool";
const POOL_DIR: &str = ".kdict-sahpool";
const POOL_CAPACITY: usize = 10;
const INSTALLED_FILE: &str = "_installed.sqlite3";

/// `core`, `stdict`, ... (lower-case ascii, digits, `_`, `-`).
pub fn valid_pack_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

pub fn file_name(pack: &str) -> String {
    format!("{pack}.sqlite3")
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledPack {
    pub id: String,
    pub version: String,
    pub bytes: u64,
}

pub struct Store {
    pub util: &'static OpfsSAHPoolUtil,
    installed: Conn,
    pub packs: Vec<PackDb>,
}

impl Store {
    /// Install the sahpool VFS, open the install record and every pack already in the pool.
    pub async fn open() -> Result<Store, String> {
        let cfg = OpfsSAHPoolCfgBuilder::new()
            .vfs_name(VFS_NAME)
            .directory(POOL_DIR)
            .initial_capacity(POOL_CAPACITY)
            .build();
        let util = install::<WasmOsCallback>(&cfg, false).await.map_err(|e| format!("OPFS sahpool: {e}"))?;
        util.ensure_capacity(POOL_CAPACITY).await.map_err(|e| format!("OPFS sahpool capacity: {e}"))?;
        // the util handle lives as long as the worker
        let util: &'static OpfsSAHPoolUtil = Box::leak(Box::new(util));

        let installed = Conn::open(INSTALLED_FILE, Some(VFS_NAME), false).map_err(|e| e.to_string())?;
        installed
            .exec(
                "CREATE TABLE IF NOT EXISTS installed (pack TEXT PRIMARY KEY, version TEXT NOT NULL, \
                 bytes INTEGER NOT NULL, installed_at INTEGER)",
            )
            .map_err(|e| e.to_string())?;

        let mut store = Store { util, installed, packs: Vec::new() };
        let mut names = util.names().map_err(|e| e.to_string())?;
        names.sort();
        for n in names {
            if let Some(id) = n.strip_suffix(".sqlite3") {
                if valid_pack_id(id) {
                    if let Err(e) = store.open_pack(id) {
                        web_sys_log(&format!("kdict-core: cannot open pack '{id}': {e}"));
                    }
                }
            }
        }
        Ok(store)
    }

    /// (Re)open a pack file read-only and register it.
    pub fn open_pack(&mut self, id: &str) -> Result<(), String> {
        self.close_pack(id);
        let conn = Conn::open(&file_name(id), Some(VFS_NAME), true).map_err(|e| e.to_string())?;
        // Every page miss is a read through OPFS, so keep hot index pages in memory: the default
        // 2 MB cache made short prefix and English queries ~50x slower than native.
        let cache_kib = if id == "core" { 48 * 1024 } else { 16 * 1024 };
        // Packs are read-only and nothing else writes them while open: exclusive locking keeps the
        // shared lock and page cache across statements instead of re-reading the header each time.
        conn.exec(&format!("PRAGMA cache_size = -{cache_kib}; PRAGMA temp_store = MEMORY; PRAGMA locking_mode = EXCLUSIVE;"))
            .map_err(|e| e.to_string())?;
        let db = PackDb::new(id, conn).map_err(|e| e.to_string())?;
        self.packs.push(db);
        Ok(())
    }

    /// Close the connection of a pack (the file stays).
    pub fn close_pack(&mut self, id: &str) {
        self.packs.retain(|p| p.id != id);
    }

    /// Close and delete a pack's file and its install record.
    pub fn remove_pack(&mut self, id: &str) -> Result<(), String> {
        self.close_pack(id);
        let _ = self
            .installed
            .query("DELETE FROM installed WHERE pack = ?1", &[id.into()])
            .map_err(|e| e.to_string())?;
        // remove the file and any stray journal left by an interrupted writer
        for suffix in ["", "-journal", "-wal"] {
            self.util.remove(&format!("{}{}", file_name(id), suffix)).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn record_install(&mut self, id: &str, version: &str, bytes: u64) -> Result<(), String> {
        self.installed
            .query(
                "INSERT OR REPLACE INTO installed (pack, version, bytes, installed_at) VALUES (?1, ?2, ?3, ?4)",
                &[id.into(), version.into(), (bytes as i64).into(), (js_sys::Date::now() as i64).into()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn installed_packs(&self) -> Vec<InstalledPack> {
        let recorded = self
            .installed
            .query("SELECT pack, version, bytes FROM installed", &[])
            .unwrap_or_default();
        let mut out = Vec::new();
        for p in &self.packs {
            let rec = recorded.iter().find(|r| r.string(0) == p.id);
            let version = rec
                .map(|r| r.string(1))
                .or_else(|| p.meta("version"))
                .unwrap_or_default();
            let bytes = rec.and_then(|r| r.int(2)).map(|v| v as u64).unwrap_or_else(|| {
                p.conn
                    .query("SELECT page_count * page_size FROM pragma_page_count, pragma_page_size", &[])
                    .ok()
                    .and_then(|r| r.first().and_then(|r| r.int(0)))
                    .unwrap_or(0) as u64
            });
            out.push(InstalledPack { id: p.id.clone(), version, bytes });
        }
        out
    }

    /// The opened packs among `ids`, in the order given.
    pub fn select(&self, ids: &[String]) -> Vec<&PackDb> {
        let mut out = Vec::new();
        for id in ids {
            if let Some(p) = self.packs.iter().find(|p| &p.id == id) {
                if !out.iter().any(|q: &&PackDb| q.id == p.id) {
                    out.push(p);
                }
            }
        }
        out
    }

    /// All opened packs (core first).
    pub fn all(&self) -> Vec<&PackDb> {
        let mut v: Vec<&PackDb> = self.packs.iter().collect();
        v.sort_by_key(|p| (p.id != "core", p.id.clone()));
        v
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console, js_name = warn)]
    fn console_warn(msg: &str);
}

fn web_sys_log(msg: &str) {
    console_warn(msg);
}
