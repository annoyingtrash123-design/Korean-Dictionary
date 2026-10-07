// Main-thread proxy for the DB worker.
import type { Evt, Req, Res } from './rpc';
import type { Entry, HanjaChar, Manifest, ManifestPack, PackStatus, Progress, ResultRow, SearchResult, Sentence, TextDoc, TextMatch, TextSummary } from './types';
import { createStore } from '../lib/store';
import { settings } from '../lib/settings';
import { activePacks } from '../lib/packs';
import { installActive } from '../lib/sw-update';

let worker: Worker | undefined;
let seq = 0;
const pending = new Map<number, { ok: (v: any) => void; err: (e: Error) => void }>();
export const progress = createStore<Record<string, Progress>>({});
export const packStatus$ = createStore<PackStatus | null>(null);

// Only one copy of the app can hold the dictionary files (OPFS access handles) at a time, but
// an installed app and a browser tab of the same site are separate copies. Whoever starts an
// engine announces it; every other copy hands its engine over by terminating its worker (it
// starts a fresh one, and claims the files back, the next time it is used), so the copy in use wins.
const instanceId = Math.random().toString(36).slice(2);
const channel: BroadcastChannel | undefined = typeof BroadcastChannel === 'function' ? new BroadcastChannel('kdict-engine') : undefined;
export const otherWindowInstalling = createStore(false);
channel?.addEventListener('message', (ev: MessageEvent<{ t: string; from: string }>) => {
  if (ev.data?.t === 'busy-installing' && ev.data.from !== instanceId) { otherWindowInstalling.set(true); return; }
  if (ev.data?.t !== 'claim' || ev.data.from === instanceId || !worker) return;
  if (installActive.get()) { channel?.postMessage({ t: 'busy-installing', from: instanceId }); return; } // never abort a download
  releaseWorker(true);
});
// After handing over, this copy must not take the files back for background work (status
// refreshes, history upgrades…) — that would knock out the copy in use. Calls wait until the user
// actually comes back here: a tap, a key, focus, or the page becoming visible again.
let handedOver = false;
let waiters: (() => void)[] = [];
function releaseWorker(handOver = false) {
  worker?.terminate(); worker = undefined;
  for (const p of pending.values()) p.err(new Error('Worker restarted')); pending.clear();
  if (handOver) handedOver = true;
}
function userIsBack() {
  if (!handedOver) return;
  handedOver = false;
  const ws = waiters; waiters = [];
  ws.forEach((f) => f());
  if (packStatus$.get()) refreshStatus().catch(() => undefined);
}
for (const ev of ['pointerdown', 'keydown', 'input', 'focusin'] as const) window.addEventListener(ev, userIsBack, true);
window.addEventListener('focus', userIsBack);
document.addEventListener('visibilitychange', () => { if (document.visibilityState === 'visible') userIsBack(); });

function w(): Worker {
  if (worker) return worker;
  channel?.postMessage({ t: 'claim', from: instanceId });
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
async function call<T>(method: string, ...args: unknown[]): Promise<T> {
  if (handedOver) await new Promise<void>((r) => waiters.push(r));
  return new Promise<T>((ok, err) => {
    const id = ++seq; pending.set(id, { ok, err });
    w().postMessage({ id, method, args } satisfies Req);
  });
}

/** Drop the worker (and with it a failed engine init, e.g. storage locked by another tab) and start a fresh one. */
export function restartWorker() {
  releaseWorker();
  packStatus$.set(null);
}

export const PACKS_HINT = 'kd.packs';
/** Installed pack ids remembered from the last run, so the UI can start before the engine is ready. */
export const packsHint = (): string[] => { try { return JSON.parse(localStorage.getItem(PACKS_HINT) || '[]'); } catch { return []; } };
export async function refreshStatus(): Promise<PackStatus> {
  const s = await call<PackStatus>('packStatus');
  try { performance.mark('kd-engine-ready'); } catch { /* ignore */ }
  try { localStorage.setItem(PACKS_HINT, JSON.stringify(Object.keys(s.packs).filter((k) => s.packs[k].installed))); } catch { /* ignore */ }
  packStatus$.set(s);
  return s;
}
export const enabledPacks = (): string[] => activePacks(settings.get(), packStatus$.get(), packsHint());

// Reader library (texts pack), cached until the pack changes.
let textList: Promise<TextSummary[]> | undefined;
const textDocs = new Map<string, Promise<TextDoc | null>>();
/** Forget cached texts (after the texts pack is installed or removed). */
export const resetTexts = () => { textList = undefined; textDocs.clear(); };

// Search coalescing: while one search runs, only the newest pending query is kept; superseded ones resolve empty.
const EMPTY: SearchResult = { mode: 'english', rows: [] };
let searching = false;
let waiting: { q: string; limit?: number; ok: (r: SearchResult) => void; err: (e: Error) => void } | null = null;
// Identical in-flight requests share one promise, so the search bar can start a query on the
// keystroke itself and the results view (which renders a frame later) picks up the same request.
const inflight = new Map<string, Promise<SearchResult>>();
function searchLatest(q: string, limit?: number): Promise<SearchResult> {
  const k = `${q}\u0000${limit ?? ''}\u0000${enabledPacks().join(',')}`;
  const hit = inflight.get(k);
  if (hit) return hit;
  const p = searchQueued(q, limit);
  inflight.set(k, p);
  const drop = () => { if (inflight.get(k) === p) inflight.delete(k); };
  p.then(drop, drop);
  return p;
}
function searchQueued(q: string, limit?: number): Promise<SearchResult> {
  return new Promise((ok, err) => {
    if (waiting) waiting.ok(EMPTY);
    waiting = { q, limit, ok, err };
    if (!searching) void pump();
  });
}
async function pump() {
  searching = true;
  while (waiting) {
    const w = waiting; waiting = null;
    const t0 = performance.now();
    try {
      const r = await call<SearchResult & { engineMs?: number }>('search', w.q, { packs: enabledPacks(), limit: w.limit });
      performance.measure('kd-search', { start: t0, detail: { q: w.q, limit: w.limit, engineMs: r.engineMs } });
      w.ok(r);
    } catch (e) { w.err(e as Error); }
  }
  searching = false;
}

export const db = {
  search: (q: string, o: { limit?: number } = {}) => searchLatest(q, o.limit),
  getEntriesByHeadword: (hw: string) => call<Entry[]>('entriesByHeadword', hw, enabledPacks()),
  getEntry: (source: string, id: number) => call<Entry | null>('entry', source, id),
  hanjaChar: (ch: string) => call<HanjaChar | null>('hanjaChar', ch),
  wordsWithHanja: (ch: string, limit: number, offset = 0) => call<ResultRow[]>('wordsWithHanja', ch, limit, offset),
  sentences: (text: string, limit: number) => call<Sentence[]>('sentences', text, limit),
  listTexts: (): Promise<TextSummary[]> => (textList ??= call<TextSummary[]>('listTexts').catch((e) => { textList = undefined; throw e; })),
  getText: (id: string): Promise<TextDoc | null> => {
    let p = textDocs.get(id);
    if (!p) {
      p = call<TextDoc | null>('getText', id);
      p.catch(() => textDocs.delete(id));
      textDocs.set(id, p);
      if (textDocs.size > 12) textDocs.delete(textDocs.keys().next().value!);
    }
    return p;
  },
  /** Dictionary match around a UTF-16 `offset` of `text` (Hangul eojeol incl. old spelling, or longest hanja word). */
  lookupInText: (text: string, offset: number, limit?: number) =>
    call<TextMatch>('lookupInText', text, offset, { packs: enabledPacks().filter((id) => id !== 'texts'), limit }),
  packStatus: refreshStatus,
  install: async (pack: ManifestPack, manifestUrl: string, version: string) => {
    // The engine keeps the old pack usable until the new import finishes, so status is refreshed after success AND failure.
    installActive.set(true);
    try { const s = await call<PackStatus>('install', pack, manifestUrl, version); packStatus$.set(s); return s; }
    finally { installActive.set(false); resetTexts(); await refreshStatus().catch(() => undefined); }
  },
  removePack: async (id: string) => { const s = await call<PackStatus>('removePack', id); resetTexts(); packStatus$.set(s); return s; },
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

// Troubleshooting hook: `await __kdDiagnostics()` in the console prints the engine's storage log.
(globalThis as unknown as { __kdDiagnostics?: () => Promise<string> }).__kdDiagnostics = () => call<string>('diagnostics');
