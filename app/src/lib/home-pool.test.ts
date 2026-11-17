import { describe, expect, it } from 'vitest';
import { cachedPool, loadIdiomPool, POOL_KEY, type PoolStorage, type SliceFn } from './home-pool';

const mem = (): PoolStorage & { m: Map<string, string> } => { const m = new Map<string, string>(); return { m, getItem: (k) => m.get(k) ?? null, setItem: (k, v) => void m.set(k, v) }; };
const slices = (calls: number[]): SliceFn => async (lo, span) => { calls.push(lo); return { ids: [lo + 1, lo + 2], next: lo + span < 30 ? lo + span : null }; };

describe('idiom pool', () => {
  it('collects every slice in order and saves the result per version', async () => {
    const store = mem(), calls: number[] = [];
    const ids = await loadIdiomPool('v1', slices(calls), { store, pauseMs: 0, span: 10 });
    expect(calls).toEqual([0, 10, 20]);
    expect(ids).toEqual([1, 2, 11, 12, 21, 22]);
    expect(cachedPool('v1', store)).toEqual(ids);
  });
  it('reuses the saved pool for the same version, rescans for a new one', async () => {
    const store = mem(), calls: number[] = [];
    await loadIdiomPool('v1', slices(calls), { store, pauseMs: 0, span: 10 });
    await loadIdiomPool('v1', slices(calls), { store, pauseMs: 0, span: 10 });
    expect(calls).toHaveLength(3);
    await loadIdiomPool('v2', slices(calls), { store, pauseMs: 0, span: 10 });
    expect(calls).toHaveLength(6);
  });
  it('works without storage and ignores a corrupt saved value', async () => {
    const calls: number[] = [];
    expect(await loadIdiomPool('v', slices(calls), { store: undefined, pauseMs: 0, span: 40 })).toEqual([1, 2]);
    const store = mem(); store.m.set(POOL_KEY, '{nope');
    expect(cachedPool('v', store)).toBeUndefined();
  });
});
