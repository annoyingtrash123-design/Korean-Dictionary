// Main-thread proxy for the DB worker.
import type { Evt, Req, Res } from './rpc';
import type { Entry, GrammarRow, HanjaChar, Manifest, ManifestPack, PackStatus, Progress, ResultRow, SearchResult, Sentence } from './types';
import { createStore } from '../lib/store';

let worker: Worker | undefined;
let seq = 0;
const pending = new Map<number, { ok: (v: any) => void; err: (e: Error) => void }>();
export const progress = createStore<Record<string, Progress>>({});
export const packStatus$ = createStore<PackStatus | null>(null);

function w(): Worker {
  if (worker) return worker;
  worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (ev: MessageEvent<Res | Evt>) => {
    const m = ev.data as Res & Evt;
    if (m.event === 'progress') { const p = m.payload as Progress; progress.set((s) => ({ ...s, [p.pack]: p })); return; }
    const p = pending.get(m.id); if (!p) return;
    pending.delete(m.id);
    if (m.error) p.err(new Error(m.error)); else p.ok(m.result);
  };
  worker.onerror = (e) => { for (const p of pending.values()) p.err(new Error(e.message || 'Worker error')); pending.clear(); };
  return worker;
}
function call<T>(method: string, ...args: unknown[]): Promise<T> {
  return new Promise<T>((ok, err) => {
    const id = ++seq; pending.set(id, { ok, err });
    w().postMessage({ id, method, args } satisfies Req);
  });
}

export async function refreshStatus(): Promise<PackStatus> {
  const s = await call<PackStatus>('packStatus');
  packStatus$.set(s);
  return s;
}
export const installedPacks = (stdictEnabled: boolean): string[] => {
  const s = packStatus$.get();
  return ['core', ...(stdictEnabled ? ['stdict'] : [])].filter((p) => s?.packs[p]?.installed);
};

export const db = {
  search: (q: string, o: { stdict: boolean; limit?: number }) => call<SearchResult>('search', q, { packs: installedPacks(o.stdict), limit: o.limit }),
  getEntriesByHeadword: (hw: string, stdict: boolean) => call<Entry[]>('entriesByHeadword', hw, installedPacks(stdict)),
  getEntry: (source: string, id: number) => call<Entry | null>('entry', source, id),
  hanjaChar: (ch: string) => call<HanjaChar | null>('hanjaChar', ch),
  wordsWithHanja: (ch: string, limit: number, offset = 0) => call<ResultRow[]>('wordsWithHanja', ch, limit, offset),
  sentences: (text: string, limit: number) => call<Sentence[]>('sentences', text, limit),
  grammarList: () => call<GrammarRow[]>('grammarList'),
  randomWordOfDay: (date: string) => call<ResultRow | null>('wordOfDay', date),
  packStatus: refreshStatus,
  install: async (pack: ManifestPack, manifestUrl: string, version: string) => {
    const s = await call<PackStatus>('install', pack, manifestUrl, version); packStatus$.set(s); return s;
  },
  removePack: async (id: string) => { const s = await call<PackStatus>('removePack', id); packStatus$.set(s); return s; },
};

export const MANIFEST_URL = () => new URL(import.meta.env.BASE_URL + 'data/manifest.json', location.origin).toString();
const MANIFEST_CACHE = 'kd.manifest';
export async function fetchManifest(): Promise<Manifest> {
  const res = await fetch(MANIFEST_URL(), { cache: 'no-store' });
  if (!res.ok) throw new Error(`Could not load the data manifest (${res.status})`);
  const m = (await res.json()) as Manifest;
  try { localStorage.setItem(MANIFEST_CACHE, JSON.stringify(m)); } catch { /* */ }
  return m;
}
export const cachedManifest = (): Manifest | null => { try { return JSON.parse(localStorage.getItem(MANIFEST_CACHE) || 'null'); } catch { return null; } };
