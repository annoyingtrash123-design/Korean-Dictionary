//! Compression of a SQLite pack into one gzip stream split into <= 20 MB chunks.

use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Meta keys that change on every build without the data changing.
const VOLATILE_META: &[&str] = &["version", "built_at"];

/// Fingerprint of what a pack *contains* (16 hex digits), independent of when it was built: every
/// table's schema and rows, except the build stamp in `meta` and FTS indexes (derived from rows that
/// are already hashed). It is the pack's version in the manifest, so the app re-downloads a pack
/// only when its content changed, not on every release.
pub fn content_digest(db: &Path) -> Result<String> {
    let conn = rusqlite::Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let virt: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND sql LIKE 'CREATE VIRTUAL TABLE%'")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let tables: Vec<(String, String)> = conn
        .prepare("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut h = Sha256::new();
    for (name, sql) in tables {
        if virt.iter().any(|v| name == *v || name.starts_with(&format!("{v}_"))) {
            h.update(sql.as_bytes()); // the index's definition still counts
            continue;
        }
        h.update(b"\x00table\x00");
        h.update(name.as_bytes());
        h.update(sql.as_bytes());
        let mut st = conn.prepare(&format!("SELECT * FROM \"{name}\""))?;
        let n = st.column_count();
        let mut rows = st.query([])?;
        while let Some(r) = rows.next()? {
            if name == "meta" {
                if let Ok(k) = r.get::<_, String>(0) {
                    if VOLATILE_META.contains(&k.as_str()) {
                        continue;
                    }
                }
            }
            h.update(b"\x01");
            for i in 0..n {
                use rusqlite::types::ValueRef::*;
                match r.get_ref(i)? {
                    Null => h.update(b"n"),
                    Integer(v) => { h.update(b"i"); h.update(v.to_le_bytes()); }
                    Real(v) => { h.update(b"r"); h.update(v.to_le_bytes()); }
                    Text(v) => { h.update(b"t"); h.update((v.len() as u64).to_le_bytes()); h.update(v); }
                    Blob(v) => { h.update(b"b"); h.update((v.len() as u64).to_le_bytes()); h.update(v); }
                }
            }
        }
    }
    Ok(h.finalize().iter().take(8).map(|b| format!("{b:02x}")).collect())
}

pub const CHUNK_BYTES: u64 = 20_000_000;

struct ChunkWriter {
    dir: PathBuf,
    base: String,
    limit: u64,
    index: usize,
    cur: Option<File>,
    cur_bytes: u64,
    sizes: Vec<(String, u64)>,
}

impl ChunkWriter {
    fn name(&self, i: usize) -> String {
        format!("{}.{:03}", self.base, i)
    }
    fn roll(&mut self) -> std::io::Result<()> {
        self.close_cur();
        let name = self.name(self.index);
        self.cur = Some(File::create(self.dir.join(&name))?);
        self.cur_bytes = 0;
        self.sizes.push((name, 0));
        self.index += 1;
        Ok(())
    }
    fn close_cur(&mut self) {
        if let (Some(f), Some(last)) = (self.cur.take(), self.sizes.last_mut()) {
            let _ = f.sync_all();
            last.1 = self.cur_bytes;
        }
    }
}

impl Write for ChunkWriter {
    fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize> {
        let total = buf.len();
        while !buf.is_empty() {
            if self.cur.is_none() || self.cur_bytes >= self.limit {
                self.roll()?;
            }
            let room = (self.limit - self.cur_bytes) as usize;
            let n = room.min(buf.len());
            self.cur.as_mut().unwrap().write_all(&buf[..n])?;
            self.cur_bytes += n as u64;
            buf = &buf[n..];
        }
        Ok(total)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(f) = self.cur.as_mut() {
            f.flush()?;
        }
        Ok(())
    }
}

/// Compress `db` into `<out_dir>/<file>.gz.NNN`; returns the manifest pack object.
pub fn compress_pack(id: &str, required: bool, db: &Path, out_dir: &Path, counts: &Value, chunk_bytes: u64) -> Result<Value> {
    let file = db.file_name().unwrap().to_string_lossy().into_owned();
    let version = content_digest(db).ok(); // not SQLite (tests): the file checksum below stands in
    let base = format!("{file}.gz");
    // remove stale chunks of this pack
    for e in std::fs::read_dir(out_dir)? {
        let e = e?;
        if e.file_name().to_string_lossy().starts_with(&format!("{base}.")) {
            std::fs::remove_file(e.path())?;
        }
    }
    let mut cw = ChunkWriter { dir: out_dir.to_path_buf(), base, limit: chunk_bytes, index: 0, cur: None, cur_bytes: 0, sizes: vec![] };
    cw.roll()?; // ensure at least one chunk exists
    let mut gz = GzEncoder::new(cw, Compression::new(9));
    let mut src = File::open(db)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut bytes = 0u64;
    loop {
        let n = src.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        gz.write_all(&buf[..n])?;
        bytes += n as u64;
    }
    let mut cw = gz.finish()?;
    cw.close_cur();
    let sha: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    let gz_bytes: u64 = cw.sizes.iter().map(|(_, s)| s).sum();
    let chunks: Vec<Value> = cw.sizes.iter().map(|(n, s)| json!({"file": n, "bytes": s})).collect();
    Ok(json!({
        "id": id, "version": version.unwrap_or_else(|| sha[..16].to_string()), "required": required, "file": file, "bytes": bytes, "gz_bytes": gz_bytes,
        "sha256": sha, "chunks": chunks, "counts": counts,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db(dir: &Path, name: &str, built_at: &str, gloss: &str) -> PathBuf {
        let p = dir.join(name);
        let c = rusqlite::Connection::open(&p).unwrap();
        c.execute_batch("CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT); CREATE TABLE entries(id INTEGER PRIMARY KEY, gloss TEXT); CREATE VIRTUAL TABLE entries_fts USING fts5(gloss);").unwrap();
        c.execute("INSERT INTO meta VALUES ('version', ?1), ('built_at', ?1), ('schema', '1')", [built_at]).unwrap();
        c.execute("INSERT INTO entries(gloss) VALUES (?1)", [gloss]).unwrap();
        c.execute("INSERT INTO entries_fts(gloss) VALUES (?1)", [gloss]).unwrap();
        p
    }

    #[test]
    fn content_digest_ignores_the_build_stamp_but_not_the_data() {
        let d = tempfile::tempdir().unwrap();
        let a = content_digest(&db(d.path(), "a.sqlite", "20261101-0000", "school")).unwrap();
        let b = content_digest(&db(d.path(), "b.sqlite", "20261201-0930", "school")).unwrap();
        let c = content_digest(&db(d.path(), "c.sqlite", "20261101-0000", "schools")).unwrap();
        assert_eq!(a.len(), 16);
        assert_eq!(a, b, "same rows, different build stamp: same version (no re-download)");
        assert_ne!(a, c, "a changed row must change the version");
    }
}
