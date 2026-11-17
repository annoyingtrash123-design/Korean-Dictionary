import { DATA_BASE } from '../lib/native';
// Main-thread proxy for the DB worker.
import type { Evt, Req, Res } from './rpc';
import type { Entry, GameWord, GrammarLessonRow, GrammarRow, HomeData, HanjaChar, Manifest, ManifestPack, PackStatus, Progress, ResultRow, SearchResult, Sentence, StudyGrammar, StudyOverview, StudyWord, TextDoc, TextMatch, TextSummary } from './types';
import { createStore } from '../lib/store';
import { settings } from '../lib/settings';
import { activePacks } from '../lib/packs';
import { logError, recordEntry, sanitizeEntries } from '../lib/errlog';

let worker: Worker | undefined;
let seq = 0;
const pending = new Map<number, { ok: (v: any) => void; err: (e: Error) => void }>();
export const progress = createStore<Record<string, Progress>>({});
export const packStatus$ = createStore<PackStatus | null>(null);

function releaseWorker() {
  worker?.terminate(); worker = undefined;
  for (const p of pending.values()) p.err(new Error('Worker restarted')); pending.clear();
}

function w(): Worker {
  if (worker) return worker;
  worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (ev: MessageEvent<Res | Evt>) => {
    const m = ev.data as Res & Evt;
    if (m.event === 'log') { const [e] = sanitizeEntries([m.payload]); if (e) recordEntry(e); return; } // worker errors go to the on-device log
    if (m.event === 'progress') { const p = m.payload as Progress; progress.set((s) => ({ ...s, [p.pack]: p })); return; }
    const p = pending.get(m.id); if (!p) return;
    pending.delete(m.id);
    if (m.error) p.err(new Error(m.error)); else p.ok(m.result);
  };
  worker.onerror = (e) => { logError('worker', e.message || 'Worker error'); for (const p of pending.values()) p.err(new Error(e.message || 'Worker error')); pending.clear(); };
  return worker;
}
function call<T>(method: string, ...args: unknown[]): Promise<T> {
  return new Promise<T>((ok, err) => {
    const id = ++seq; pending.set(id, { ok, err });
    w().postMessage({ id, method, args } satisfies Req);
  });
}

/** A new worker with fresh storage handles, keeping the UI state (used to recover a download whose
 *  storage folders vanished under it). */
export function freshWorker() { releaseWorker(); }

/** Drop the worker (and with it a failed engine init) and start a fresh one. */
export function restartWorker() {
  releaseWorker();
  packStatus$.set(null);
}

export const PACKS_HINT = 'kd.packs';
/** Installed pack ids remembered from the last run, so the UI can start before the engine is ready. */
export const packsHint = (): string[] => { try { return JSON.parse(localStorage.getItem(PACKS_HINT) || '[]'); } catch { return []; /* storage unavailable or unparsable: no hint */ } };
/** What the device looks like, for Settings → Technical details. */
export interface StorageReport { usageMB: number | null; lastOk: string | null; build: string; diag: string }
const LAST_OK = 'kd.lastOk';
export async function makeReport(): Promise<StorageReport> {
  const est = await navigator.storage?.estimate?.().catch(() => undefined);
  const diag = await call<string>('diagnostics').catch((e) => String(e?.message ?? e));
  let lastOk: string | null = null;
  try { lastOk = localStorage.getItem(LAST_OK); } catch { /* storage unavailable: the report just omits it */ }
  return { usageMB: est?.usage != null ? Math.round(est.usage / 1048576) : null, lastOk, build: __BUILD__, diag };
}
const saveHint = (s: PackStatus) => { try { localStorage.setItem(PACKS_HINT, JSON.stringify(Object.keys(s.packs).filter((k) => s.packs[k].installed))); } catch { /* storage unavailable: the hint is only a startup optimisation */ } };
export async function refreshStatus(): Promise<PackStatus> {
  const s = await call<PackStatus>('packStatus');
  try { performance.mark('kd-engine-ready'); } catch { /* performance API unavailable: timing marks are optional */ }
  if (s.packs.core?.installed) {
    try { localStorage.setItem(LAST_OK, `${new Date().toISOString().slice(0, 16)} · app ${__BUILD__} · ${Object.keys(s.packs).join(', ')}`); } catch { /* storage unavailable: this note is only a diagnostic aid */ }
  }
  saveHint(s);
  packStatus$.set(s);
  return s;
}
export const enabledPacks = (): string[] => activePacks(settings.get(), packStatus$.get(), packsHint());

// Reader library (texts pack), cached until the pack changes.
let textList: Promise<TextSummary[]> | undefined;
let grammarRows: Promise<GrammarRow[]> | undefined;
const textDocs = new Map<string, Promise<TextDoc | null>>();
/** Forget cached texts (after the texts pack is installed or removed). */
export const resetTexts = () => { textList = undefined; grammarRows = undefined; textDocs.clear(); };

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
  similarWords: (source: string, id: number, limit = 8) => call<ResultRow[]>('similarWords', source, id, limit),
  gameWords: (level: number, limit: number) => call<GameWord[]>('gameWords', level, limit),
  gameWordsFor: (headwords: string[]) => call<GameWord[]>('gameWordsFor', headwords),
  chainCandidates: (starts: string[], used: string[], limit = 20) => call<GameWord[]>('chainCandidates', starts, used, limit),
  chainCheck: (word: string) => call<GameWord | null>('chainCheck', word),
  sentences: (text: string, limit: number) => call<Sentence[]>('sentences', text, limit),
  /** The grammar reference table (cached until packs change). */
  grammarList: (): Promise<GrammarRow[]> => (grammarRows ??= call<GrammarRow[]>('grammarList').catch((e) => { grammarRows = undefined; throw e; })),
  /** Reviewed mini-lessons of a grammar entry (empty: none, or a pack from before lessons). */
  grammarLessons: (source: string, id: number) => call<GrammarLessonRow[]>('grammarLessons', source, id),
  /** Home page picks for a day number (see `lib/home-day.ts`). */
  home: (day: number) => call<HomeData>('home', day),
  /** Study lists (see `lib/study-lists.ts`): counts, then pages of words / the grammar of a level. */
  studyOverview: () => call<StudyOverview>('studyOverview'),
  studyWords: (level: number, limit: number, offset: number) => call<StudyWord[]>('studyWords', level, limit, offset),
  studyGrammar: (level: number) => call<StudyGrammar[]>('studyGrammar', level),
  studyTopicWords: (category: string, byLevel: boolean, limit: number, offset: number) => call<StudyWord[]>('studyTopicWords', category, byLevel, limit, offset),
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
    installsInFlight++;
    try { const s = await call<PackStatus>('install', pack, manifestUrl, version); packStatus$.set(s); return s; }
    finally {
      installsInFlight--;
      resetTexts(); await refreshStatus().catch(() => undefined);
    }
  },
  removePack: async (id: string) => { const s = await call<PackStatus>('removePack', id); resetTexts(); saveHint(s); packStatus$.set(s); return s; },
};

export const MANIFEST_URL = () => new URL(DATA_BASE + 'data/manifest.json', location.origin).toString();
const MANIFEST_CACHE = 'kd.manifest';
export async function fetchManifest(): Promise<Manifest> {
  const res = await fetch(MANIFEST_URL(), { cache: 'no-store' });
  if (!res.ok) throw new Error(`Could not load the data manifest (${res.status})`);
  const m = (await res.json()) as Manifest;
  try { localStorage.setItem(MANIFEST_CACHE, JSON.stringify(m)); } catch { /* storage unavailable or full: the manifest cache is optional */ }
  return m;
}
export const cachedManifest = (): Manifest | null => { try { return JSON.parse(localStorage.getItem(MANIFEST_CACHE) || 'null'); } catch { return null; /* storage unavailable or unparsable: no cached manifest */ } };

let installsInFlight = 0;
// Test hook: number of pack installs still downloading/importing.
(globalThis as unknown as { __kdInstalling?: () => number }).__kdInstalling = () => installsInFlight;

// Troubleshooting hook: `await __kdDiagnostics()` in the console prints the engine's storage log.
(globalThis as unknown as { __kdDiagnostics?: () => Promise<string> }).__kdDiagnostics = () => call<string>('diagnostics');
