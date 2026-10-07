//! Streaming pack import. The app downloads gzip chunks, decompresses them
//! (`DecompressionStream`) and hands the plain SQLite bytes to [`Importers::write`] in order.
//! Bytes go straight into the OPFS file through the sahpool import API, so the whole database
//! (100-300 MB) is never held in memory; only the current chunk is buffered.

use crate::db::{valid_pack_id, Store};
use crate::packfile::{import_slot, slot_file, verify_sha256};
use rsqlite_vfs::transfer::{DbImport, DbTransfer};
use sqlite_wasm_vfs::sahpool::OpfsSAHImportTarget;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

struct Active {
    import: DbImport<OpfsSAHImportTarget<'static>>,
    total: u64,
    written: u64,
    /// Slot file the new copy is written to (never the live one).
    file: String,
    /// SHA-256 over the decompressed bytes written so far.
    hasher: Sha256,
}

#[derive(Default)]
pub struct Importers {
    active: HashMap<String, Active>,
    scratch: Vec<u8>,
}

impl Importers {
    /// Start importing `pack` (exactly `total_bytes` of SQLite file) into the pack's spare file
    /// slot. The installed copy (if any) is left open and usable until [`Self::finish`] has
    /// verified the new file; a failed or aborted import leaves it intact (the sahpool does not
    /// publish an import until its commit, and a dropped import reclaims its slot).
    pub fn begin(&mut self, store: &mut Store, pack: &str, total_bytes: u64) -> Result<(), String> {
        if !valid_pack_id(pack) {
            return Err(format!("invalid pack id '{pack}'"));
        }
        self.abort(pack);
        let file = slot_file(pack, import_slot(pack, store.live_file(pack)));
        // leftover from an interrupted earlier install (never the live file)
        store.remove_file(&file)?;
        let import = store
            .util
            .begin_import(&file, total_bytes)
            .map_err(|e| format!("cannot start import of '{pack}': {e}"))?;
        self.active.insert(pack.to_string(), Active { import, total: total_bytes, written: 0, file, hasher: Sha256::new() });
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
        act.hasher.update(&self.scratch[..n]);
        act.written += n as u64;
        Ok(())
    }

    /// Verify the byte count and (if given) the SHA-256 of the whole file, publish the new
    /// file, open it, record `version`, then retire the previous copy. Every failure before
    /// the swap leaves the previously installed pack untouched.
    pub fn finish(&mut self, store: &mut Store, pack: &str, version: &str, sha256: Option<&str>) -> Result<(), String> {
        let act = self.active.remove(pack).ok_or_else(|| format!("no import in progress for '{pack}'"))?;
        let Active { import, total, written, file, hasher } = act;
        if written != total {
            // dropping the import reclaims the reserved file slot
            return Err(format!("import of '{pack}' incomplete: {written} of {total} bytes"));
        }
        // before publishing anything: a corrupt download is dropped here
        verify_sha256(sha256, &hasher.finalize())?;
        import.finish().map_err(|e| format!("import of '{pack}' failed: {e}"))?;
        // the published file must be a readable pack
        store.swap_in(pack, &file, version, total).map_err(|e| format!("'{pack}' is not a valid dictionary pack: {e}"))
    }

    /// Abandon an in-progress import (also used when a new one starts).
    pub fn abort(&mut self, pack: &str) {
        self.active.remove(pack);
    }
}
