import { createStore } from './store';

export interface ReaderPrefs { size: number; lh: number; serif: boolean; en: boolean; modern: boolean; reading: boolean }
export const DEFAULT_PREFS: ReaderPrefs = { size: 20, lh: 1.95, serif: true, en: true, modern: false, reading: true };
const KEY = 'kd.reader.prefs';
const clamp = (n: unknown, lo: number, hi: number, d: number) => (typeof n === 'number' && isFinite(n) ? Math.min(hi, Math.max(lo, n)) : d);

export function sanitizePrefs(raw: any): ReaderPrefs {
  const r = raw && typeof raw === 'object' ? raw : {};
  const b = (v: unknown, d: boolean) => (typeof v === 'boolean' ? v : d);
  return { size: clamp(r.size, 14, 32, DEFAULT_PREFS.size), lh: clamp(r.lh, 1.4, 2.6, DEFAULT_PREFS.lh), serif: b(r.serif, true), en: b(r.en, true), modern: b(r.modern, false), reading: b(r.reading, true) };
}
const load = (): ReaderPrefs => { try { return sanitizePrefs(JSON.parse(localStorage.getItem(KEY) || '{}')); } catch { return { ...DEFAULT_PREFS }; } };
export const readerPrefs = createStore<ReaderPrefs>(load());
readerPrefs.subscribe((p) => { try { localStorage.setItem(KEY, JSON.stringify(p)); } catch { /* storage unavailable */ } });
export const updatePrefs = (patch: Partial<ReaderPrefs>) => readerPrefs.set((p) => sanitizePrefs({ ...p, ...patch }));

// Reading position per text: paragraph index + fraction of that paragraph already scrolled past.
export interface ReadPos { n: number; f: number }
const POS_KEY = 'kd.reader.pos';
const MAX_POS = 300;
export function sanitizePos(raw: any): Record<string, ReadPos> {
  const out: Record<string, ReadPos> = {};
  if (raw && typeof raw === 'object') for (const [id, v] of Object.entries(raw as Record<string, any>)) {
    if (v && Number.isInteger(v.n) && v.n >= 0) out[id] = { n: v.n, f: clamp(v.f, v.n === 0 ? -200 : 0, 1, 0) };   // f < 0 on paragraph 0 = still in the header above the text
  }
  return out;
}
export function getPos(id: string): ReadPos | null {
  try { return sanitizePos(JSON.parse(localStorage.getItem(POS_KEY) || '{}'))[id] ?? null; } catch { return null; }
}
export function setPos(id: string, pos: ReadPos): void {
  try {
    const all = sanitizePos(JSON.parse(localStorage.getItem(POS_KEY) || '{}'));
    delete all[id]; all[id] = pos;            // most recent last
    const keys = Object.keys(all);
    for (const k of keys.slice(0, Math.max(0, keys.length - MAX_POS))) delete all[k];
    localStorage.setItem(POS_KEY, JSON.stringify(all));
  } catch { /* storage unavailable */ }
}
export function readIds(): string[] { try { return Object.keys(sanitizePos(JSON.parse(localStorage.getItem(POS_KEY) || '{}'))); } catch { return []; } }
