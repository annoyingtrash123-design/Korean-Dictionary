// A private, on-device error log: a ring buffer of the last MAX_ENTRIES errors, kept in IndexedDB
// so a problem can be diagnosed after a reload. Nothing is ever sent anywhere. Every function here
// is safe to call from anywhere and never throws (a broken log must not break the app).
import { get, set, del } from 'idb-keyval';
import { userDb } from './idb';
import { createStore } from './store';

export interface ErrEntry { t: number; src: string; msg: string; stack?: string }
export const MAX_ENTRIES = 200;
const KEY = 'errlog';
const MAX_MSG = 500;
const MAX_STACK = 600;

export const errors$ = createStore<ErrEntry[]>([]); // oldest first

const inWorker = () => typeof WorkerGlobalScope !== 'undefined' && typeof self !== 'undefined' && self instanceof WorkerGlobalScope;

export function makeEntry(src: string, err: unknown, now = Date.now()): ErrEntry {
  let msg: string; let stack: string | undefined;
  try {
    if (err instanceof Error) { msg = `${err.name}: ${err.message}`; stack = err.stack; }
    else if (err && typeof err === 'object' && 'message' in err) { msg = String((err as { message: unknown }).message); stack = (err as { stack?: string }).stack; }
    else msg = typeof err === 'string' ? err : JSON.stringify(err) ?? String(err);
  } catch { msg = String(err); }
  const e: ErrEntry = { t: now, src: String(src).slice(0, 40), msg: msg.slice(0, MAX_MSG) };
  if (typeof stack === 'string' && stack) {
    // Keep only a few frames (not the repeated message line).
    const frames = stack.split('\n').filter((l) => /^\s*at |@/.test(l)).slice(0, 4).join('\n');
    if (frames) e.stack = frames.slice(0, MAX_STACK);
  }
  return e;
}

/** Parse untrusted data (stored or imported) into valid entries, capped to the ring size. */
export function sanitizeEntries(raw: unknown): ErrEntry[] {
  if (!Array.isArray(raw)) return [];
  const out: ErrEntry[] = [];
  for (const r of raw) {
    if (r && typeof r === 'object' && typeof r.t === 'number' && typeof r.src === 'string' && typeof r.msg === 'string') {
      out.push({ t: r.t, src: r.src.slice(0, 40), msg: r.msg.slice(0, MAX_MSG), ...(typeof r.stack === 'string' ? { stack: r.stack.slice(0, MAX_STACK) } : {}) });
    }
  }
  return out.slice(-MAX_ENTRIES);
}

let loaded: Promise<void> | undefined;
let chain: Promise<unknown> = Promise.resolve();

function ensureLoaded(): Promise<void> {
  return (loaded ??= (async () => {
    try {
      const stored = sanitizeEntries(await get(KEY, userDb()));
      if (stored.length) errors$.set((mem) => [...stored, ...mem].slice(-MAX_ENTRIES));
    } catch { /* storage unavailable: the in-memory log still works */ }
  })());
}
function persist(): void {
  chain = chain.then(async () => {
    await ensureLoaded();
    try { await set(KEY, errors$.get(), userDb()); } catch { /* storage unavailable or full: keep going in memory */ }
  }).catch(() => undefined);
}

/** Add an already-built entry (used for entries forwarded from the DB worker). */
export function recordEntry(e: ErrEntry): void {
  try {
    errors$.set((l) => [...l, e].slice(-MAX_ENTRIES));
    persist();
  } catch { /* never throw from the logger */ }
}

/** Log an error. In the DB worker it is forwarded to the main thread, which owns the log. */
export function logError(src: string, err: unknown): void {
  try {
    const e = makeEntry(src, err);
    if (inWorker()) { (self as unknown as Worker).postMessage({ event: 'log', payload: e }); return; }
    recordEntry(e);
  } catch { /* never throw from the logger */ }
}

/** Read the persisted log into memory (call when showing it). */
export function loadErrors(): Promise<ErrEntry[]> {
  return ensureLoaded().then(() => errors$.get(), () => errors$.get());
}

export async function clearErrors(): Promise<void> {
  try {
    await ensureLoaded();
    errors$.set([]);
    chain = chain.then(() => del(KEY, userDb())).catch(() => undefined);
    await chain;
  } catch { /* never throw from the logger */ }
}

/** Plain text, newest first, for the clipboard. */
export function formatErrors(list: ErrEntry[]): string {
  return [...list].reverse().map((e) => `${new Date(e.t).toISOString()} [${e.src}] ${e.msg}${e.stack ? '\n' + e.stack : ''}`).join('\n\n');
}

/** Capture uncaught errors and unhandled rejections in the main thread. */
export function installErrorCapture(): void {
  try {
    window.addEventListener('error', (ev) => {
      logError('window', ev.error ?? `${ev.message} (${ev.filename ?? ''}:${ev.lineno ?? ''})`);
    });
    window.addEventListener('unhandledrejection', (ev) => logError('promise', ev.reason));
    void ensureLoaded();
  } catch { /* no window */ }
}

/** Capture uncaught errors inside the DB worker (forwarded to the main thread). */
export function installWorkerErrorCapture(): void {
  try {
    self.addEventListener('error', (ev) => logError('worker', (ev as ErrorEvent).error ?? (ev as ErrorEvent).message));
    self.addEventListener('unhandledrejection', (ev) => logError('worker', (ev as PromiseRejectionEvent).reason));
  } catch { /* not in a worker */ }
}
