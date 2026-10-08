import { describe, expect, it } from 'vitest';
import { collapse, groupTranslations, posShort, visiblePhrases, PHRASES_COLLAPSED } from './enko';
import type { EnKoPhrase, EnKoSense } from './types';

const s = (pos: string, sense: string, ...ko: string[]): EnKoSense => ({ term: 'report', pos, sense, words: ko.map((k) => ({ ko: k })) });

describe('enko', () => {
  it('groups senses by part of speech in order', () => {
    const g = groupTranslations([s('noun', 'account', '보고', '보고서'), s('noun', 'noise', '폭음'), s('verb', 'relay', '보고하다')]);
    expect(g.map((x) => [x.pos, x.senses.map((y) => y.sense)])).toEqual([['noun', ['account', 'noise']], ['verb', ['relay']]]);
  });
  it('shortens pos labels and keeps unknown ones', () => {
    expect(posShort('noun')).toBe('n.');
    expect(posShort('proper noun')).toBe('prop. n.');
    expect(posShort('suffix')).toBe('suffix');
  });
  it('collapses long phrase lists only', () => {
    const ph = (n: number): EnKoPhrase[] => Array.from({ length: n }, (_, i) => ({ term: `report ${i}`, pos: 'noun', words: [{ ko: '보고' }] }));
    expect(visiblePhrases(ph(10), false)).toMatchObject({ hidden: 0 });
    const long = visiblePhrases(ph(30), false);
    expect([long.shown.length, long.hidden]).toEqual([PHRASES_COLLAPSED, 30 - PHRASES_COLLAPSED]);
    expect(visiblePhrases(ph(30), true).shown.length).toBe(30);
  });
  it('collapse keeps short lists whole', () => {
    expect(collapse([1, 2, 3, 4, 5, 6, 7], false, 5)).toEqual({ shown: [1, 2, 3, 4, 5, 6, 7], hidden: 0 });
    expect(collapse([1, 2, 3, 4, 5, 6, 7, 8], false, 5)).toEqual({ shown: [1, 2, 3, 4, 5], hidden: 3 });
  });
});
