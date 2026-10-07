import { createStore } from './store';
import type { Manifest } from '../db/types';

/** Current text in the search bar. */
export const query = createStore('');
/** Latest manifest seen online; `stale` lists installed packs whose version differs from it (each pack is compared). */
export const update = createStore<{ manifest: Manifest | null; available: boolean; stale: string[] }>({ manifest: null, available: false, stale: [] });

/** Installed packs whose version differs from the manifest's (core first). */
export function stalePacks(packs: Record<string, { installed: boolean; version?: string }> | undefined, m: { version: string; packs: { id: string }[] } | null): string[] {
  if (!m || !packs) return [];
  return Object.keys(packs).filter((id) => packs[id].installed && m.packs.some((p) => p.id === id) && packs[id].version !== m.version)
    .sort((a, b) => (a === 'core' ? -1 : b === 'core' ? 1 : a.localeCompare(b)));
}

export function todayStr(d = new Date()): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}
export function fmtBytes(n: number | undefined): string {
  if (n == null || isNaN(n)) return '–';
  if (n < 1024) return `${n} B`;
  if (n < 1048576) return `${(n / 1024).toFixed(0)} KB`;
  if (n < 1073741824) return `${(n / 1048576).toFixed(n < 10485760 ? 1 : 0)} MB`;
  return `${(n / 1073741824).toFixed(2)} GB`;
}
