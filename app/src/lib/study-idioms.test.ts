import { beforeEach, describe, expect, it, vi } from 'vitest';

const calls: unknown[][] = [];
vi.mock('../db/client', () => ({
  packStatus$: { get: () => null, set: () => undefined, subscribe: () => () => undefined },
  db: {
    studyIdioms: vi.fn(async (list: string, limit: number, offset: number) => {
      calls.push([list, limit, offset]);
      return [{ rank: offset + 1, kind: 'seongeo', title: '설상가상', headword: '설상가상', gloss: '눈 위에 서리가 덮인다는 뜻으로', pack: 'stdict', entry: null }];
    }),
  },
}));

import { fetchRows } from './study-source';
import { GROUP_IDIOMS, idiomId, idiomRow, idiomSummaries, parseIdiomId } from './study-idioms';
import { parseStudyId, summariesOf } from './study-lists';

beforeEach(() => { calls.length = 0; });

describe('idioms & proverbs study lists', () => {
  it('ids round-trip and unknown lists are not ours', () => {
    expect(parseIdiomId(idiomId('sokdam100'))).toBe('sokdam100');
    expect(parseStudyId(idiomId('seongeo300'))).toEqual({ kind: 'idiom', list: 'seongeo300' });
    expect(parseStudyId('study:idiom:nope')).toBeUndefined();
    expect(parseStudyId('study:vocab:3')).toEqual({ kind: 'vocab', level: 3 });
  });

  it('the group shows the lists with counts, and is absent for a pack without the table', () => {
    const s = idiomSummaries([{ list: 'sokdam100', count: 100 }, { list: 'sokdam_all', count: 655 }, { list: 'seongeo100', count: 100 }, { list: 'seongeo300', count: 300 }]);
    expect(s.map((x) => [x.name, x.count, x.group])).toEqual([
      ['Common proverbs — top 100', 100, GROUP_IDIOMS], ['All learner proverbs', 655, GROUP_IDIOMS],
      ['Essential 성어 — top 100', 100, GROUP_IDIOMS], ['Common 성어 — top 300', 300, GROUP_IDIOMS],
    ]);
    expect(idiomSummaries(undefined)).toEqual([]);
    expect(idiomSummaries([{ list: 'sokdam100', count: 0 }])).toEqual([]);
    const o = { vocab: [], grammar: [], topics: [] };
    expect(summariesOf(o)).toEqual([]);
    expect(summariesOf({ ...o, idioms: [{ list: 'seongeo100', count: 100 }] }).map((x) => x.id)).toEqual(['study:idiom:seongeo100']);
  });

  it('a row opens the entry when its pack is installed, else searches for the title', () => {
    const base = { rank: 1, kind: 'seongeo' as const, title: '설상가상', headword: '설상가상', gloss: 'g', pack: 'stdict' };
    const open = idiomRow({ ...base, entry: { source: 'stdict', id: 12, homonym: null, pos: null } });
    expect(open.path).toBe('/entry/stdict/12?hw=%EC%84%A4%EC%83%81%EA%B0%80%EC%83%81');
    expect(open.tag).toBe('성어');
    const closed = idiomRow({ ...base, entry: null });
    expect(closed.path).toBe('/word/%EC%84%A4%EC%83%81%EA%B0%80%EC%83%81');
    expect(closed.item).toMatchObject({ kind: 'entry', source: 'stdict', headword: '설상가상' });
    // a core (krdict) proverb is saved under the exact headword, shown by its clean title
    const p = idiomRow({ rank: 3, kind: 'sokdam', title: '가난 구제는 나라님도 못한다', headword: '가난 구제는 나라님[임금님]도 못한다', gloss: 'g', pack: 'core', entry: null });
    expect(p.tag).toBe('속담');
    expect(p.item).toMatchObject({ source: 'krdict', headword: '가난 구제는 나라님[임금님]도 못한다' });
  });

  it('pages come from the engine by list and offset', async () => {
    const rows = await fetchRows(idiomId('seongeo100'), '', 100, 100);
    expect(calls).toEqual([['seongeo100', 100, 100]]);
    expect(rows[0].title).toBe('설상가상');
  });
});
