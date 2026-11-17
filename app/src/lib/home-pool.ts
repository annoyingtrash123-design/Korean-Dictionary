/** The idiom pool of the home page: entry ids of the krdict idioms and proverbs that can be shown (English gloss, usable example).
 *  Finding them means reading most of the core pack (no index on `kind`), so it is done in slices of entry ids with a pause
 *  between them (searches slip in), and the result is kept per pack version. */
export interface IdiomSlice { ids: number[]; next: number | null }
export type SliceFn = (lo: number, span: number) => Promise<IdiomSlice>;
export interface PoolStorage { getItem(k: string): string | null; setItem(k: string, v: string): void }

export const POOL_KEY = 'kd.home.idioms';
const SPAN = 4000;

const safe = (): PoolStorage | undefined => { try { return localStorage; } catch { return undefined; } };

/** The saved pool for a pack version, if any. */
export function cachedPool(version: string, store: PoolStorage | undefined = safe()): number[] | undefined {
  try {
    const v = JSON.parse(store?.getItem(POOL_KEY) ?? 'null') as { v?: string; ids?: unknown } | null;
    return v?.v === version && Array.isArray(v.ids) && v.ids.every((x) => typeof x === 'number') ? (v.ids as number[]) : undefined;
  } catch { return undefined; }
}

export async function loadIdiomPool(version: string, slice: SliceFn, opts: { store?: PoolStorage; pauseMs?: number; span?: number } = {}): Promise<number[]> {
  const store = 'store' in opts ? opts.store : safe();
  const hit = cachedPool(version, store);
  if (hit) return hit;
  const ids: number[] = [];
  for (let lo = 0, guard = 0; guard < 10_000; guard++) {
    const s = await slice(lo, opts.span ?? SPAN);
    ids.push(...s.ids);
    if (s.next == null) break;
    lo = s.next;
    await new Promise((r) => setTimeout(r, opts.pauseMs ?? 40));
  }
  try { store?.setItem(POOL_KEY, JSON.stringify({ v: version, ids })); } catch { /* storage full or blocked: the pool is just found again next time */ }
  return ids;
}
