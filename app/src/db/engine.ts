// Engine adapter. TEMPORARY implementation on top of @sqlite.org/sqlite-wasm (opfs-sahpool);
// the Rust/wasm engine replaces this file behind the same `Engine` interface (see types.ts).
import sqlite3InitModule from '@sqlite.org/sqlite-wasm';
import * as q from './queries';
import type { Dbs, Db } from './queries';
import type { Engine, Entry, GrammarRow, HanjaChar, ResultRow, SearchResult, Source } from './types';

type SahPool = Awaited<ReturnType<Awaited<ReturnType<typeof sqlite3InitModule>>['installOpfsSAHPoolVfs']>>;
type Sqlite = Awaited<ReturnType<typeof sqlite3InitModule>>;

const fileOf = (id: string) => `/${id}.sqlite3`;

interface Import {
  total: number; written: number; promise: Promise<number>; failed: Promise<never>;
  waiter?: (v: Uint8Array | undefined) => void; queued?: { data: Uint8Array | undefined; done: () => void }; ack?: () => void;
}

export function createEngine(): Engine {
  let sqlite3: Sqlite;
  let pool: SahPool;
  const dbs: Dbs = {};
  const dbObjs: Record<string, { close(): void }> = {};
  const imports = new Map<string, Import>();

  const open = (id: string) => {
    const d = new pool.OpfsSAHPoolDb(fileOf(id));
    dbObjs[id] = d; dbs[id] = d as unknown as Db;
  };
  const close = (id: string) => { try { dbObjs[id]?.close(); } catch { /* ignore */ } delete dbObjs[id]; delete dbs[id]; };
  const packsOf = (packs: string[]) => packs.filter((p) => dbs[p]);

  return {
    async init() {
      sqlite3 = await sqlite3InitModule();
      pool = await sqlite3.installOpfsSAHPoolVfs({ initialCapacity: 10, clearOnInit: false });
      for (const f of pool.getFileNames()) {
        const m = f.match(/^\/(\w+)\.sqlite3$/);
        if (m) { try { open(m[1]); } catch (e) { console.warn('open failed', f, e); } }
      }
    },
    async installedPacks() {
      return Object.keys(dbs).map((id) => {
        const d = dbs[id]!;
        const version = String(d.selectObjects("SELECT value FROM meta WHERE key='version'")[0]?.value ?? '');
        const n = d.selectObjects('SELECT page_count * page_size AS b FROM pragma_page_count, pragma_page_size')[0];
        return { id, version, bytes: Number(n?.b ?? 0) };
      });
    },
    async beginImport(id, total) {
      close(id);
      const imp = { total, written: 0 } as Import;
      const pull = (): Promise<Uint8Array | undefined> => {
        imp.ack?.(); imp.ack = undefined;
        if (imp.queued) { const it = imp.queued; imp.queued = undefined; imp.ack = it.done; return Promise.resolve(it.data); }
        return new Promise((res) => { imp.waiter = res; });
      };
      imp.promise = pool.importDb(fileOf(id), pull);
      imp.failed = imp.promise.then(() => new Promise<never>(() => {}), (e) => { throw e; });
      imp.failed.catch(() => undefined);
      imports.set(id, imp);
    },
    async writeChunk(id, bytes) {
      const imp = imports.get(id); if (!imp) throw new Error('no import in progress');
      imp.written += bytes.byteLength;
      const p = new Promise<void>((done) => {
        if (imp.waiter) { const w = imp.waiter; imp.waiter = undefined; imp.ack = done; w(bytes); } else imp.queued = { data: bytes, done };
      });
      await Promise.race([p, imp.failed]);
    },
    async finishImport(id) {
      const imp = imports.get(id); if (!imp) throw new Error('no import in progress');
      imports.delete(id);
      if (imp.waiter) { const w = imp.waiter; imp.waiter = undefined; w(undefined); } else imp.queued = { data: undefined, done: () => undefined };
      const n = await imp.promise;
      if (n !== imp.total) { try { pool.unlink(fileOf(id)); } catch { /* */ } throw new Error(`Size mismatch for ${id}: got ${n}, expected ${imp.total}`); }
      open(id);
    },
    async deletePack(id) { close(id); try { pool.unlink(fileOf(id)); } catch { /* not present */ } },
    async search(query, opts): Promise<SearchResult> { return q.search(dbs, query, { ...opts, packs: packsOf(opts.packs) }); },
    async entriesByHeadword(hw, packs): Promise<Entry[]> { return q.getEntriesByHeadword(dbs, hw, packsOf(packs)); },
    async entry(source, id) { return q.getEntry(dbs, source as Source, id); },
    async hanjaChar(ch): Promise<HanjaChar | null> { return q.hanjaChar(dbs, ch); },
    async wordsWithHanja(ch, limit, offset): Promise<ResultRow[]> { return q.wordsWithHanja(dbs, ch, limit, offset, packsOf(['core', 'stdict'])); },
    async sentences(text, limit) { return q.sentences(dbs, text, limit); },
    async grammarList(): Promise<GrammarRow[]> { return q.grammarList(dbs); },
    async wordOfDay(date) { return q.wordOfDay(dbs, date); },
  };
}
