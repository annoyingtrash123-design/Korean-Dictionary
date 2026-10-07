import { get, set } from 'idb-keyval';
import { createStore } from './store';
import { userDb } from './idb';

export interface HistoryItem { key: string; source: string; id: number; headword: string; hanja: string | null; gloss: string | null; ts: number }
export const MAX_HISTORY = 100;
export const history = createStore<HistoryItem[]>([]);
let loaded: Promise<void> | undefined;

export function loadHistory(): Promise<void> {
  return (loaded ??= get<HistoryItem[]>('history', userDb()).catch(() => undefined).then((v) => { if (Array.isArray(v)) history.set(v); }));
}
export async function recordHistory(it: Omit<HistoryItem, 'key' | 'ts' | 'hanja' | 'gloss'> & { hanja?: string | null; gloss?: string | null }): Promise<void> {
  await loadHistory();
  const key = `${it.source}:${it.id}`;
  const next = [{ ...it, hanja: it.hanja ?? null, gloss: it.gloss ?? null, key, ts: Date.now() }, ...history.get().filter((h) => h.key !== key)].slice(0, MAX_HISTORY);
  history.set(next);
  await set('history', next, userDb()).catch(() => undefined);
}
export async function clearHistory(): Promise<void> {
  history.set([]);
  await set('history', [], userDb()).catch(() => undefined);
}
