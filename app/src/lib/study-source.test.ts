import { beforeEach, describe, expect, it, vi } from 'vitest';

const calls: unknown[][] = [];
const word = (id: number) => ({ source: 'krdict', id, headword: `w${id}`, homonym: null, hanja: null, pos: 'noun', level: 1, gloss: `g${id}` });
vi.mock('../db/client', () => ({
  packStatus$: { get: () => null, set: () => undefined, subscribe: () => () => undefined },
  db: {
    studyOverview: vi.fn(),
    studyWords: vi.fn(async (level: number, limit: number, offset: number) => { calls.push(['words', level, limit, offset]); return Array.from({ length: Math.max(0, Math.min(limit, 1200 - offset)) }, (_, i) => word(offset + i)); }),
    studyGrammar: vi.fn(async (level: number) => { calls.push(['grammar', level]); return Array.from({ length: 60 }, (_, i) => ({ form: `-f${i}`, pattern: `-p${i}`, summary_en: 's', entry: null })); }),
    studyTopicWords: vi.fn(async (category: string, byLevel: boolean, limit: number, offset: number) => { calls.push(['topic', category, byLevel, limit, offset]); return [word(offset)]; }),
  },
}));

import { fetchAllRows, fetchRows, loadOverview, overview$, resetOverview } from './study-source';

beforeEach(() => { calls.length = 0; resetOverview(); });

describe('study source: engine glue', () => {
  it('loads the overview once per core version', async () => {
    const o = { vocab: [{ level: 1, count: 1 }], grammar: [], topics: [] };
    const fetch = vi.fn(async () => o);
    loadOverview('v1', fetch);
    loadOverview('v1', fetch);
    await vi.waitFor(() => expect(overview$.get()).toEqual(o));
    expect(fetch).toHaveBeenCalledTimes(1);
    loadOverview('v2', fetch);
    expect(fetch).toHaveBeenCalledTimes(2);
  });
  it('hides the lists (null) when the overview cannot be read', async () => {
    loadOverview('v1', async () => { throw new Error('old engine'); });
    await vi.waitFor(() => expect(overview$.get()).toBeNull());
  });
  it('asks the engine for the right page of each kind of list', async () => {
    expect((await fetchRows('study:vocab:6', '', 100, 100)).map((r) => r.title)).toEqual(Array.from({ length: 100 }, (_, i) => `w${100 + i}`));
    await fetchRows('study:topic:인간 > 감각', 'level', 0, 100);
    await fetchRows('study:topic:인간', 'az', 200, 100);
    expect(calls).toEqual([['words', 6, 100, 100], ['topic', '인간 > 감각', true, 100, 0], ['topic', '인간', false, 100, 200]]);
    expect(await fetchRows('saved', '', 0, 10)).toEqual([]);
  });
  it('slices a grammar level (loaded whole) into pages with stable row keys', async () => {
    const p2 = await fetchRows('study:grammar:3', '', 50, 50);
    expect(p2.length).toBe(10);
    expect(p2[0]).toMatchObject({ key: 'g3:50', title: '-f50' });
    expect(p2[0].item).toMatchObject({ kind: 'grammar', pattern: '-p50', level: 3 });
  });
  it('reads a whole list page by page for "Add all"', async () => {
    const rows = await fetchAllRows('study:vocab:6', '');
    expect(rows.length).toBe(1200);
    expect(new Set(rows.map((r) => r.key)).size).toBe(1200);
    expect(calls).toEqual([['words', 6, 500, 0], ['words', 6, 500, 500], ['words', 6, 500, 1000]]);
  });
});
