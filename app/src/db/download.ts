// Pack download manager (runs inside the DB worker).
// 1. fetch the gzip chunks sequentially into OPFS files (resumable: finished chunks are skipped),
// 2. stream chunk files -> ONE DecompressionStream('gzip') -> engine.writeChunk (never buffers the DB).
import { get, set, del } from 'idb-keyval';
import type { Engine, ManifestPack, Progress } from './types';

export interface InstalledRecord { version: string; bytes: number; installedAt: number }
const DL_DIR = 'kd-dl';

interface DlState { version: string; done: number }

export const getInstalled = (id: string) => get<InstalledRecord>('installed:' + id);

function chunkList(pack: ManifestPack, manifestUrl: string) {
  return pack.chunks.map((c, i) => {
    const o = typeof c === 'string' ? { file: c } : c;
    const file = o.file ?? o.name ?? o.url ?? `${pack.file}.gz.${String(i).padStart(3, '0')}`;
    return { name: file.split('/').pop()!, url: new URL(file, manifestUrl).toString(), bytes: o.bytes ?? o.gz_bytes };
  });
}

async function dirFor(id: string, create: boolean) {
  const root = await navigator.storage.getDirectory();
  const top = await root.getDirectoryHandle(DL_DIR, { create });
  return top.getDirectoryHandle(id, { create });
}

async function fetchToFile(url: string, dir: FileSystemDirectoryHandle, name: string, onBytes: (n: number) => void): Promise<number> {
  const fh = await dir.getFileHandle(name, { create: true });
  const ah = await (fh as unknown as { createSyncAccessHandle(): Promise<FileSystemSyncAccessHandle> }).createSyncAccessHandle();
  try {
    ah.truncate(0);
    const res = await fetch(url);
    if (!res.ok || !res.body) throw new Error(`Download failed (${res.status}) for ${name}`);
    const rd = res.body.getReader();
    let pos = 0;
    for (;;) {
      const { value, done } = await rd.read();
      if (done) break;
      ah.write(value, { at: pos }); pos += value.byteLength; onBytes(value.byteLength);
    }
    ah.flush();
    return pos;
  } finally { ah.close(); }
}

export async function installPack(
  engine: Engine, pack: ManifestPack, manifestUrl: string, version: string, emit: (p: Progress) => void,
): Promise<void> {
  const chunks = chunkList(pack, manifestUrl);
  const dir = await dirFor(pack.id, true);
  const stateKey = 'dl:' + pack.id;
  let st = await get<DlState>(stateKey);
  if (!st || st.version !== version) { // new version: discard stale chunks
    for await (const k of (dir as unknown as { keys(): AsyncIterable<string> }).keys()) await dir.removeEntry(k);
    st = { version, done: 0 };
    await set(stateKey, st);
  }
  const total = pack.gz_bytes || chunks.reduce((a, c) => a + (c.bytes ?? 0), 0);
  let got = 0;
  const report = (i: number) => emit({ pack: pack.id, phase: 'download', done: got, total, chunk: i, chunks: chunks.length });
  // resume: count already finished chunks (verify they still exist with the right size)
  let start = 0;
  for (; start < Math.min(st.done, chunks.length); start++) {
    try {
      const f = await (await dir.getFileHandle(chunks[start].name)).getFile();
      if (chunks[start].bytes != null && f.size !== chunks[start].bytes) break;
      got += f.size;
    } catch { break; }
  }
  report(start);
  for (let i = start; i < chunks.length; i++) {
    const before = got;
    const size = await fetchToFile(chunks[i].url, dir, chunks[i].name, (n) => { got += n; report(i); });
    if (chunks[i].bytes != null && size !== chunks[i].bytes) { got = before; throw new Error(`Chunk ${chunks[i].name} has wrong size (${size})`); }
    await set(stateKey, { version, done: i + 1 });
  }
  emit({ pack: pack.id, phase: 'download', done: total, total, chunk: chunks.length, chunks: chunks.length });

  // ---- install: chunk files -> single gzip stream -> engine ----
  let idx = 0;
  let cur: ReadableStreamDefaultReader<Uint8Array> | undefined;
  const joined = new ReadableStream<Uint8Array>({
    async pull(c) {
      for (;;) {
        if (!cur) {
          if (idx >= chunks.length) { c.close(); return; }
          const f = await (await dir.getFileHandle(chunks[idx++].name)).getFile();
          cur = f.stream().getReader();
        }
        const { value, done } = await cur.read();
        if (done) { cur = undefined; continue; }
        c.enqueue(value); return;
      }
    },
  });
  const reader = joined.pipeThrough(new DecompressionStream("gzip") as unknown as ReadableWritablePair<Uint8Array, Uint8Array>).getReader();
  await engine.beginImport(pack.id, pack.bytes);
  let written = 0;
  emit({ pack: pack.id, phase: 'install', done: 0, total: pack.bytes });
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    await engine.writeChunk(pack.id, value);
    written += value.byteLength;
    emit({ pack: pack.id, phase: 'install', done: written, total: pack.bytes });
  }
  if (written !== pack.bytes) throw new Error(`Decompressed size mismatch for ${pack.id}: ${written} vs ${pack.bytes}`);
  await engine.finishImport(pack.id, version);
  await set('installed:' + pack.id, { version, bytes: pack.bytes, installedAt: Date.now() } satisfies InstalledRecord);
  await del(stateKey);
  try { const root = await navigator.storage.getDirectory(); const top = await root.getDirectoryHandle(DL_DIR); await top.removeEntry(pack.id, { recursive: true }); } catch { /* */ }
  emit({ pack: pack.id, phase: 'done', done: pack.bytes, total: pack.bytes });
}

export async function forgetPack(id: string) { await del('installed:' + id); await del('dl:' + id); }
