import { describe, expect, it } from 'vitest';
import { groupResults, mergeRows, sameWordRows } from './merge';
import type { ResultRow } from './types';

const row = (o: Partial<ResultRow>): ResultRow => ({
  source: 'krdict', id: 1, headword: '학교', hanja: '學校', pos: 'noun', kind: 'word', pack: 'core', via: 'exact', rank: 1, lang: 'en', hw_norm: '학교', ...o,
});

describe('groupResults', () => {
  const rows = [
    row({ source: 'stdict', id: 9, pack: 'stdict', lang: 'ko' }),
    row({ source: 'wikt', id: 5, gloss: 'school (wikt)' }),
    row({ source: 'krdict', id: 1, level: 1, gloss: 'school' }),
    row({ source: 'krdict', id: 2, headword: '학생', hanja: '學生', gloss: 'student', via: 'prefix' }),
    row({ source: 'krdict', id: 3, headword: '배', hanja: undefined, homonym: 1, gloss: 'ship' }),
    row({ source: 'krdict', id: 4, headword: '배', hanja: undefined, homonym: 2, gloss: 'pear' }),
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
    const only = groupResults([row({ source: 'stdict', pack: 'stdict', lang: 'ko', id: 7 })]);
    expect(only[0].primary.source).toBe('stdict');
    expect(only[0].gloss).toBe('');
  });
  it('treats opendict like stdict (never primary over English sources)', () => {
    const x = groupResults([row({ source: 'opendict', pack: 'opendict', lang: 'ko', id: 1 }), row({ source: 'kengdic', id: 2, gloss: 'school' })]);
    expect(x[0].primary.source).toBe('kengdic');
    expect(x[0].sources).toEqual(['kengdic', 'opendict']);
  });
  it('uses the best match tier for the group', () => {
    const x = groupResults([row({ via: 'prefix', id: 1 }), row({ via: 'exact', source: 'wikt', id: 2 })]);
    expect(x[0].via).toBe('exact');
  });
});

describe('hanja-less rows join their word', () => {
  it('folds a Wiktionary row into the only hanja word with that headword, at the better position', () => {
    const x = groupResults([
      row({ source: 'krdict', id: 1, headword: '보고', hanja: '報告', gloss: 'report' }),
      row({ source: 'wikt', id: 2, headword: '보고하다', hanja: undefined, pos: 'verb', gloss: 'to report' }),
      row({ source: 'krdict', id: 3, headword: '보고하다', hanja: '報告하다', pos: 'verb', gloss: 'report' }),
    ]);
    expect(x.map((g) => g.key)).toEqual(['보고|報告', '보고하다|報告하다']);
    expect(x[1].rows).toHaveLength(2);
    expect(x[1].primary.source).toBe('krdict');
  });
  it('picks the homograph whose glosses match, and keeps ambiguous rows apart', () => {
    const x = groupResults([
      row({ source: 'krdict', id: 1, headword: '보고', hanja: '報告', gloss: 'report; account' }),
      row({ source: 'krdict', id: 2, headword: '보고', hanja: '寶庫', gloss: 'treasury; treasure house' }),
      row({ source: 'wikt', id: 3, headword: '보고', hanja: undefined, gloss: 'treasure house' }),
      row({ source: 'wikt', id: 4, headword: '보고', hanja: undefined, gloss: 'see also' }),
    ]);
    expect(x.find((g) => g.hanja === '寶庫')!.rows.map((r) => r.id)).toEqual([2, 3]);
    expect(x.find((g) => !g.hanja)!.rows.map((r) => r.id)).toEqual([4]);
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
    expect(sameWordRows(all, { headword: '학교', hanja: '學校' }).map((e) => e.id)).toEqual([2, 3, 1]);
    // with two homographs the hanja-less row is not attached to either
    const two = [...all, { headword: '학교', hanja: '鶴橋', source: 'stdict' as const, homonym: null, id: 4 }];
    expect(sameWordRows(two, { headword: '학교', hanja: '學校' }).map((e) => e.id)).toEqual([2, 1]);
  });
});
