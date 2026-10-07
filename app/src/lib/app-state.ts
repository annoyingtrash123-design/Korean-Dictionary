import { createStore } from './store';
import type { Manifest } from '../db/types';

/** Current text in the search bar. */
export const query = createStore('');
/** Latest manifest seen online and whether it differs from the installed core version. */
export const update = createStore<{ manifest: Manifest | null; available: boolean }>({ manifest: null, available: false });

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
