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

pub use crate::packfile::valid_pack_id;
use crate::packfile::{parse_file, slot_file};
use std::collections::HashMap;

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
    /// Live file of each open pack (one of its two alternating slots, see `packfile`).
    live: HashMap<String, String>,
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
                 bytes INTEGER NOT NULL, installed_at INTEGER, file TEXT)",
            )
            .map_err(|e| e.to_string())?;
        // databases created before the two-slot scheme lack the column (duplicate-column error is fine)
        let _ = installed.exec("ALTER TABLE installed ADD COLUMN file TEXT");

        let mut store = Store { util, installed, packs: Vec::new(), live: HashMap::new() };
        let names = util.names().map_err(|e| e.to_string())?;
        let recorded: HashMap<String, String> = store
            .installed
            .query("SELECT pack, file FROM installed WHERE file IS NOT NULL", &[])
            .unwrap_or_default()
            .iter()
            .map(|r| (r.string(0), r.string(1)))
            .collect();
        let mut ids: Vec<String> = names.iter().filter_map(|n| parse_file(n)).map(|(id, _)| id).collect();
        ids.sort();
        ids.dedup();
        for id in ids {
            // the recorded file is authoritative; otherwise a legacy/unrecorded file (slot A first)
            let live = recorded
                .get(&id)
                .filter(|f| names.contains(f))
                .cloned()
                .or_else(|| [0u8, 1].iter().map(|s| slot_file(&id, *s)).find(|f| names.contains(f)));
            let Some(live) = live else { continue };
            // an interrupted import or swap can leave the other slot behind: reclaim it
            for slot in [0u8, 1] {
                let f = slot_file(&id, slot);
                if f != live && names.contains(&f) {
                    let _ = store.util.remove(&f);
                }
            }
            if let Err(e) = store.open_pack_file(&id, &live) {
                web_sys_log(&format!("kdict-core: cannot open pack '{id}': {e}"));
            }
        }
        Ok(store)
    }

    /// File name of the live copy of `id`, if installed.
    pub fn live_file(&self, id: &str) -> Option<&str> {
        self.live.get(id).map(String::as_str)
    }

    /// Open `file` read-only as pack `id` (replacing a registered connection of the same id).
    pub fn open_pack_file(&mut self, id: &str, file: &str) -> Result<(), String> {
        let db = self.connect(id, file)?;
        self.close_pack(id);
        self.packs.push(db);
        self.live.insert(id.to_string(), file.to_string());
        Ok(())
    }

    /// Publish a fully written, verified import: open `new_file`, record it as the live copy
    /// and only then close and delete the previous one. On any error the previous pack stays
    /// registered and untouched and `new_file` is deleted.
    pub fn swap_in(&mut self, id: &str, new_file: &str, version: &str, bytes: u64) -> Result<(), String> {
        let res = self.connect(id, new_file).and_then(|db| {
            self.record_install(id, version, bytes, new_file)?;
            Ok(db)
        });
        let db = match res {
            Ok(db) => db,
            Err(e) => {
                let _ = self.remove_file(new_file);
                return Err(e);
            }
        };
        self.close_pack(id);
        self.packs.push(db);
        let old = self.live.insert(id.to_string(), new_file.to_string());
        if let Some(old) = old.filter(|o| o != new_file) {
            let _ = self.remove_file(&old);
        }
        Ok(())
    }

    fn connect(&self, id: &str, file: &str) -> Result<PackDb, String> {
        let conn = Conn::open(file, Some(VFS_NAME), true).map_err(|e| e.to_string())?;
        // Every page miss is a read through OPFS, so keep hot index pages in memory: the default
        // 2 MB cache made short prefix and English queries ~50x slower than native.
        let cache_kib = if id == "core" { 48 * 1024 } else { 16 * 1024 };
        // Packs are read-only and nothing else writes them while open: exclusive locking keeps the
        // shared lock and page cache across statements instead of re-reading the header each time.
        conn.exec(&format!("PRAGMA cache_size = -{cache_kib}; PRAGMA temp_store = MEMORY; PRAGMA locking_mode = EXCLUSIVE;"))
            .map_err(|e| e.to_string())?;
        PackDb::new(id, conn).map_err(|e| e.to_string())
    }

    /// Delete a pool file and any stray journal left by an interrupted writer.
    pub fn remove_file(&self, file: &str) -> Result<(), String> {
        for suffix in ["", "-journal", "-wal"] {
            self.util.remove(&format!("{file}{suffix}")).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Close the connection of a pack (the file stays).
    pub fn close_pack(&mut self, id: &str) {
        self.packs.retain(|p| p.id != id);
    }

    /// Close and delete a pack's files (both slots) and its install record.
    pub fn remove_pack(&mut self, id: &str) -> Result<(), String> {
        self.close_pack(id);
        self.live.remove(id);
        let _ = self
            .installed
            .query("DELETE FROM installed WHERE pack = ?1", &[id.into()])
            .map_err(|e| e.to_string())?;
        for slot in [0u8, 1] {
            self.remove_file(&slot_file(id, slot))?;
        }
        Ok(())
    }

    pub fn record_install(&mut self, id: &str, version: &str, bytes: u64, file: &str) -> Result<(), String> {
        self.installed
            .query(
                "INSERT OR REPLACE INTO installed (pack, version, bytes, installed_at, file) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[id.into(), version.into(), (bytes as i64).into(), (js_sys::Date::now() as i64).into(), file.into()],
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
