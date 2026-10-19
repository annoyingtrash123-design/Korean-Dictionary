import { describe, expect, it } from 'vitest';
import { cardKey, dayKey, pickRound, review, streak, type Deck } from './srs';
import type { GameWord } from '../types';

const w = (headword: string, pos = 'noun'): GameWord => ({ source: 'krdict', id: 1, headword, pos, gloss: headword, examples: [] });
const DAY = 86_400_000;
const NOW = new Date(2026, 9, 19, 12).getTime();

describe('games: smart review', () => {
  it('moves boxes', () => {
    const a = review(undefined, w('학교'), true, NOW);
    expect(a.box).toBe(1);
    expect(a.due).toBe(NOW + DAY);
    const b = review(a, w('학교'), true, NOW);
    expect(b.box).toBe(2);
    const c = review(b, w('학교'), false, NOW);
    expect(c.box).toBe(0);
    expect(c.due).toBe(NOW);
    expect(c.seen).toBe(3);
    expect(c.right).toBe(2);
  });
  it('due first, then new, then later', () => {
    const deck: Deck = {
      [cardKey(w('가'))]: { word: w('가'), box: 3, due: NOW + 5 * DAY, seen: 3, right: 3 },
      [cardKey(w('나'))]: { word: w('나'), box: 0, due: NOW - DAY, seen: 1, right: 0 },
    };
    const r = pickRound([w('가'), w('다'), w('나'), w('라')], deck, 3, NOW);
    expect(r.map((x) => x.headword)).toEqual(['나', '다', '라']);
  });
  it('keys by headword and part of speech', () => {
    expect(cardKey(w('학교02'))).toBe(cardKey(w('학교')));
    expect(cardKey(w('배', 'noun'))).not.toBe(cardKey(w('배', 'verb')));
  });
  it('streak', () => {
    const today = dayKey(NOW), y = dayKey(NOW - DAY), yy = dayKey(NOW - 2 * DAY);
    expect(streak([yy, y, today], NOW)).toBe(3);
    expect(streak([yy, y], NOW)).toBe(2);           // today not played yet: still alive
    expect(streak([yy], NOW)).toBe(0);
  });
});
