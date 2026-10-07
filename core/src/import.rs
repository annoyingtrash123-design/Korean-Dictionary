//! Streaming pack import. The app downloads gzip chunks, decompresses them
//! (`DecompressionStream`) and hands the plain SQLite bytes to [`Importers::write`] in order.
//! Bytes go straight into the OPFS file through the sahpool import API, so the whole database
//! (100-300 MB) is never held in memory; only the current chunk is buffered.

use crate::db::{file_name, valid_pack_id, Store};
use rsqlite_vfs::transfer::{DbImport, DbTransfer};
use sqlite_wasm_vfs::sahpool::OpfsSAHImportTarget;
use std::collections::HashMap;

struct Active {
    import: DbImport<OpfsSAHImportTarget<'static>>,
    total: u64,
    written: u64,
}

#[derive(Default)]
pub struct Importers {
    active: HashMap<String, Active>,
    scratch: Vec<u8>,
}

impl Importers {
    /// Start importing `pack` (exactly `total_bytes` of SQLite file). Any installed copy of the
    /// pack is removed first so an interrupted import never leaves a half-valid pack behind.
    pub fn begin(&mut self, store: &mut Store, pack: &str, total_bytes: u64) -> Result<(), String> {
        if !valid_pack_id(pack) {
            return Err(format!("invalid pack id '{pack}'"));
        }
        self.abort(pack);
        store.remove_pack(pack)?;
        let import = store
            .util
            .begin_import(&file_name(pack), total_bytes)
            .map_err(|e| format!("cannot start import of '{pack}': {e}"))?;
        self.active.insert(pack.to_string(), Active { import, total: total_bytes, written: 0 });
        Ok(())
    }

    /// Append the next chunk (already decompressed, sequential).
    pub fn write(&mut self, pack: &str, chunk: &js_sys::Uint8Array) -> Result<(), String> {
        let n = chunk.length() as usize;
        if self.scratch.len() < n {
            self.scratch.resize(n, 0);
        }
        chunk.copy_to(&mut self.scratch[..n]);
        let act = self.active.get_mut(pack).ok_or_else(|| format!("no import in progress for '{pack}'"))?;
        if let Err(e) = act.import.write(&self.scratch[..n]) {
            // a failed write poisons the import: reclaim the slot
            self.active.remove(pack);
            return Err(format!("import of '{pack}' failed: {e}"));
        }
        act.written += n as u64;
        Ok(())
    }

    /// Verify the byte count, publish the file, open it and record `version`.
    pub fn finish(&mut self, store: &mut Store, pack: &str, version: &str) -> Result<(), String> {
        let act = self.active.remove(pack).ok_or_else(|| format!("no import in progress for '{pack}'"))?;
        let (total, written) = (act.total, act.written);
        if written != total {
            // dropping the import reclaims the reserved file slot
            return Err(format!("import of '{pack}' incomplete: {written} of {total} bytes"));
        }
        act.import.finish().map_err(|e| format!("import of '{pack}' failed: {e}"))?;
        // the published file must be a readable pack
        match store.open_pack(pack).and_then(|_| store.record_install(pack, version, total)) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = store.remove_pack(pack);
                Err(format!("'{pack}' is not a valid dictionary pack: {e}"))
            }
        }
    }

    /// Abandon an in-progress import (also used when a new one starts).
    pub fn abort(&mut self, pack: &str) {
        self.active.remove(pack);
    }
}
