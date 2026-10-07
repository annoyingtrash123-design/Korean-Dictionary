import { beforeEach, describe, expect, it } from 'vitest';
import { NO_FILTERS, byPeriod, byShelf, byline, fmtYear, levelId, matches, periodId, themesOf } from './reader-library';
import { DEFAULT_PREFS, getPos, sanitizePos, sanitizePrefs, setPos } from './reader-prefs';
import type { TextSummary } from '../db/types';

const t = (o: Partial<TextSummary>): TextSummary => ({ id: 'x', shelf: 'graded', script: 'hangul', chars: 500, title_ko: '제목', themes: [], labels: {}, ...o });

describe('library model', () => {
  it('normalises period slugs, display names and bare years', () => {
    expect(periodId(t({ period: 'joseon-late' }))).toBe('joseon-late');
    expect(periodId(t({ period: 'Colonial era' }))).toBe('colonial');
    expect(periodId(t({ period: 'Joseon', year: 1446 }))).toBe('joseon-early');
    expect(periodId(t({ period: 'Joseon', year: 1800 }))).toBe('joseon-late');
    expect(periodId(t({ period: null, year: -2333 }))).toBe('ancient');
    expect(periodId(t({ period: null, year: 1987 }))).toBe('modern');
    expect(periodId(t({}))).toBeNull();
  });
  it('maps themes and levels', () => {
    expect(themesOf(t({ themes: ['Sino-Korean relations', 'joseon', 'bogus'] }))).toEqual(['sino-korean', 'joseon']);
    expect(levelId('TOPIK 5–6')).toBe('topik56');
    expect(levelId('TOPIK 2')).toBe('topik12');
    expect(levelId('classical')).toBe('classical');
    expect(levelId(null)).toBeNull();
  });
  it('filters and groups', () => {
    const list = [t({ id: 'a', shelf: 'verse', year: 1500, period: 'joseon-early', script: 'mixed', chars: 4000 }), t({ id: 'b', year: 1910, chars: 20000, period: 'colonial', themes: ['sino-korean'] })];
    expect(list.filter((x) => matches(x, { ...NO_FILTERS, script: 'mixed' })).map((x) => x.id)).toEqual(['a']);
    expect(list.filter((x) => matches(x, { ...NO_FILTERS, length: 'long' })).map((x) => x.id)).toEqual(['b']);
    expect(list.filter((x) => matches(x, { ...NO_FILTERS, theme: 'sino-korean' })).map((x) => x.id)).toEqual(['b']);
    expect(byShelf(list, NO_FILTERS).map((s) => s.shelf.id)).toEqual(['verse', 'graded']);
    expect(byPeriod(list, NO_FILTERS).map((p) => p.period.id)).toEqual(['joseon-early', 'colonial']);
  });
  it('formats years and bylines', () => {
    expect(fmtYear(-2333)).toBe('2333 BCE');
    expect(fmtYear(698)).toBe('698 CE');
    expect(fmtYear(1446)).toBe('1446');
    expect(byline({ author_ko: '김소월', date: '1922', year: 1922 })).toBe('김소월 · 1922');
  });
});

describe('reader prefs and position', () => {
  beforeEach(() => localStorage.clear());
  it('sanitises prefs', () => {
    expect(sanitizePrefs(null)).toEqual(DEFAULT_PREFS);
    expect(sanitizePrefs({ size: 999, lh: 'x', serif: false }).size).toBe(32);
    expect(sanitizePrefs({ size: 999, lh: 'x', serif: false }).lh).toBe(DEFAULT_PREFS.lh);
  });
  it('stores position per text', () => {
    setPos('a', { n: 3, f: 0.4 });
    setPos('b', { n: 1, f: 2 });
    expect(getPos('a')).toEqual({ n: 3, f: 0.4 });
    expect(getPos('b')).toEqual({ n: 1, f: 1 });
    expect(getPos('c')).toBeNull();
    expect(sanitizePos({ a: { n: -1 }, b: 'x' })).toEqual({});
  });
});
