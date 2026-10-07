//! Compression of a SQLite pack into one gzip stream split into <= 20 MB chunks.

use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

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
        "id": id, "required": required, "file": file, "bytes": bytes, "gz_bytes": gz_bytes,
        "sha256": sha, "chunks": chunks, "counts": counts,
    }))
}
