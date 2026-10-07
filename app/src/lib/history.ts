import { get, set } from 'idb-keyval';
import { createStore } from './store';
import { userDb } from './idb';
import { legacyKey, stableKey, upgradeLegacy, type EntryRef, type SavedRef } from './entry-key';

export interface HistoryItem extends SavedRef { key: string; ts: number; legacyId?: number }
export const MAX_HISTORY = 100;
export const history = createStore<HistoryItem[]>([]);
let loaded: Promise<void> | undefined;

/** Normalise stored history; items saved as `source:id` by older versions become legacy items. */
export function sanitizeHistory(raw: unknown): HistoryItem[] {
  if (!Array.isArray(raw)) return [];
  const out: HistoryItem[] = [];
  const seen = new Set<string>();
  for (const h of raw as (Partial<HistoryItem> & { id?: unknown })[]) {
    if (!h || typeof h.source !== 'string' || typeof h.headword !== 'string') continue;
    const modern = 'pos' in h || 'homonym' in h;
    const legacyId = !modern && typeof h.id === 'number' ? h.id : undefined;
    if (!modern && legacyId == null) continue;
    const ref: SavedRef = { source: h.source, headword: h.headword, hanja: h.hanja ?? null, gloss: h.gloss ?? null, homonym: typeof h.homonym === 'number' ? h.homonym : null, pos: typeof h.pos === 'string' ? h.pos : null };
    const key = legacyId != null ? legacyKey(h.source, h.headword, legacyId) : stableKey(ref);
    if (seen.has(key)) continue;
    seen.add(key);
    out.push({ ...ref, key, ts: typeof h.ts === 'number' ? h.ts : 0, ...(legacyId != null ? { legacyId } : {}) });
  }
  return out.slice(0, MAX_HISTORY);
}

export function loadHistory(): Promise<void> {
  return (loaded ??= get('history', userDb()).catch(() => undefined).then((v) => { if (v) history.set(sanitizeHistory(v)); }));
}
const save = (h: HistoryItem[]) => set('history', h, userDb()).catch(() => undefined);

export async function upgradeHistory(lookup: Parameters<typeof upgradeLegacy>[1]): Promise<void> {
  await loadHistory();
  const r = await upgradeLegacy(history.get(), lookup);
  if (!r.changed) return;
  const byTs = new Map(r.items.map((i) => [i.ts + '|' + i.source + '|' + i.headword, i]));
  const seen = new Set<string>();
  const next = history.get().map((i) => (i.legacyId != null ? byTs.get(i.ts + '|' + i.source + '|' + i.headword) ?? i : i))
    .filter((i) => (seen.has(i.key) ? false : (seen.add(i.key), true)));
  history.set(next);
  await save(next);
}
export async function recordHistory(it: EntryRef & { hanja?: string | null; gloss?: string | null }): Promise<void> {
  await loadHistory();
  const ref: SavedRef = { source: it.source, headword: it.headword, hanja: it.hanja ?? null, gloss: it.gloss ?? null, homonym: it.homonym ?? null, pos: it.pos ?? null };
  const key = stableKey(ref);
  const next = [{ ...ref, key, ts: Date.now() }, ...history.get().filter((h) => h.key !== key)].slice(0, MAX_HISTORY);
  history.set(next);
  await save(next);
}
export async function clearHistory(): Promise<void> {
  history.set([]);
  await save([]);
}
