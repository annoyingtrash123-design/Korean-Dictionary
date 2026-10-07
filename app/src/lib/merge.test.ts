import { describe, expect, it } from 'vitest';
import { groupResults, mergeRows, sameWordRows } from './merge';
import type { ResultRow } from './types';

const row = (o: Partial<ResultRow>): ResultRow => ({
  source: 'krdict', id: 1, headword: '학교', hanja: '學校', pos: 'noun', level: null, gloss: null, kind: 'word', pack: 'core', via: 'exact', ...o,
});

describe('groupResults', () => {
  const rows = [
    row({ source: 'stdict', id: 9, pack: 'stdict', gloss: null }),
    row({ source: 'wikt', id: 5, gloss: 'school (wikt)' }),
    row({ source: 'krdict', id: 1, level: 1, gloss: 'school' }),
    row({ source: 'krdict', id: 2, headword: '학생', hanja: '學生', gloss: 'student', via: 'prefix' }),
    row({ source: 'krdict', id: 3, headword: '배', hanja: null, homonym: 1, gloss: 'ship' }),
    row({ source: 'krdict', id: 4, headword: '배', hanja: null, homonym: 2, gloss: 'pear' }),
  ];
  const g = groupResults(rows);
  it('groups by headword + hanja, keeping engine order', () => {
    expect(g.map((x) => x.key)).toEqual(['학교|學校', '학생|學生', '배|']);
  });
  it('prefers krdict as primary and carries level/gloss/sources', () => {
    expect(g[0].primary.source).toBe('krdict');
    expect(g[0].level).toBe(1);
    expect(g[0].gloss).toBe('school');
    expect(g[0].sources).toEqual(['krdict', 'wikt', 'stdict']);
    expect(g[0].rows).toHaveLength(3);
  });
  it('merges homonyms without hanja into one group', () => {
    expect(g[2].rows).toHaveLength(2);
  });
  it('falls back to Korean-only rows', () => {
    const only = groupResults([row({ source: 'stdict', pack: 'stdict', id: 7 })]);
    expect(only[0].primary.source).toBe('stdict');
    expect(only[0].gloss).toBe('');
  });
  it('uses the best match tier for the group', () => {
    const x = groupResults([row({ via: 'prefix', id: 1 }), row({ via: 'exact', source: 'wikt', id: 2 })]);
    expect(x[0].via).toBe('exact');
  });
});

describe('mergeRows', () => {
  it('dedupes by source:id keeping the better tier', () => {
    const m = mergeRows([row({ via: 'prefix' })], [row({ via: 'exact' }), row({ id: 2 })]);
    expect(m).toHaveLength(2);
    expect(m[0].via).toBe('exact');
  });
});

describe('sameWordRows', () => {
  it('keeps only same headword+hanja, ordered by source', () => {
    const all = [
      { headword: '학교', hanja: '學校', source: 'stdict' as const, homonym: null, id: 1 },
      { headword: '학교', hanja: '學校', source: 'krdict' as const, homonym: null, id: 2 },
      { headword: '학교', hanja: null, source: 'wikt' as const, homonym: null, id: 3 },
    ];
    expect(sameWordRows(all, { headword: '학교', hanja: '學校' }).map((e) => e.id)).toEqual([2, 1]);
  });
});
