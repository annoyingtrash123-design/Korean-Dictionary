import { describe, expect, it } from 'vitest';
import { extendSpan, nextCp, nextWordStart, prevCp, prevWordChar, rangeAt, runAt, shrinkSpan, snapOffset, splitHighlight, textOffsetIn } from './reader-text';

const ASTRAL = '𠮷野家 먹었다'; // 𠮷 is U+20BB7: two UTF-16 code units

describe('offset mapping', () => {
  it('counts astral characters as two UTF-16 units and snaps into the pair', () => {
    expect(ASTRAL.length).toBe(ASTRAL.codePointAt(0)! > 0xffff ? [...ASTRAL].length + 1 : 0);
    expect(snapOffset(ASTRAL, 1)).toBe(0);           // inside the surrogate pair -> start of 𠮷
    expect(snapOffset(ASTRAL, 2)).toBe(2);
    expect(nextCp(ASTRAL, 0)).toBe(2);
    expect(prevCp(ASTRAL, 2)).toBe(0);
    expect(prevCp(ASTRAL, 3)).toBe(2);
  });
  it('runAt finds the script run, including past-the-end and astral', () => {
    expect(runAt(ASTRAL, 0)).toEqual({ start: 0, end: 4 });
    expect(runAt(ASTRAL, 1)).toEqual({ start: 0, end: 4 });     // middle of the pair
    expect(runAt(ASTRAL, 4)).toEqual({ start: 0, end: 4 });     // the space just after counts as the end of the run
    expect(runAt(ASTRAL, 6)).toEqual({ start: 5, end: 8 });
    expect(runAt('안녕, 세상', 2)).toEqual({ start: 0, end: 2 });
    expect(runAt('안녕, 세상', 3)).toBeNull();
    expect(runAt('', 0)).toBeNull();
    expect(runAt('學校는 좋다', 0)).toEqual({ start: 0, end: 2 }); // hanja and hangul are separate runs
    expect(runAt('學校는 좋다', 2)).toEqual({ start: 2, end: 3 });
  });
  it('word stepping skips punctuation and spaces', () => {
    const t = '나는, 학교에 간다.';
    expect(nextWordStart(t, 2)).toBe(4);
    expect(nextWordStart(t, 10)).toBeNull();
    expect(prevWordChar(t, 4)).toBe(1);
    expect(prevWordChar(t, 0)).toBeNull();
    expect(prevWordChar(ASTRAL, 5)).toBe(3);   // 家
  });
  it('splitHighlight never splits a surrogate pair', () => {
    expect(splitHighlight(ASTRAL, { start: 1, end: 3 })).toEqual(['', '𠮷野', '家 먹었다']);
    expect(splitHighlight('abc', null)).toEqual(['abc', '', '']);
    expect(splitHighlight('abc', { start: 5, end: 9 })).toEqual(['abc', '', '']);
  });
});

describe('selection shrink / extend', () => {
  const t = '학교에서 𠮷野';
  it('shrinks and extends by one character', () => {
    expect(shrinkSpan(t, { start: 0, end: 4 })).toEqual({ start: 0, end: 3 });
    expect(extendSpan(t, { start: 0, end: 2 })).toEqual({ start: 0, end: 3 });
    expect(extendSpan(t, { start: 5, end: 5 + 0 })).toEqual({ start: 5, end: 7 }); // astral = 2 units
  });
  it('never shrinks below one character and stops at the text end', () => {
    expect(shrinkSpan(t, { start: 0, end: 1 })).toEqual({ start: 0, end: 1 });
    expect(shrinkSpan(t, { start: 5, end: 7 })).toEqual({ start: 5, end: 7 });
    expect(extendSpan(t, { start: 0, end: t.length })).toEqual({ start: 0, end: t.length });
  });
  it('shrinks off an astral character in one step', () => {
    expect(shrinkSpan(t, { start: 5, end: 8 })).toEqual({ start: 5, end: 7 });
  });
});

describe('DOM offsets', () => {
  it('maps (node, offset) to a paragraph offset across the highlight element', () => {
    const p = document.createElement('p');
    p.innerHTML = '';
    p.append('가나', Object.assign(document.createElement('mark'), { textContent: '다라' }), '마𠮷');
    document.body.append(p);
    const mark = p.querySelector('mark')!;
    expect(textOffsetIn(p, p.firstChild!, 1)).toBe(1);
    expect(textOffsetIn(p, mark.firstChild!, 1)).toBe(3);
    expect(textOffsetIn(p, p.lastChild!, 3)).toBe(7);   // after 마 + 𠮷 (2 units)
    const r = rangeAt(p, 1, 5)!;
    expect(r.toString()).toBe('나다라마');
    expect(rangeAt(p, 5, 7)!.toString()).toBe('𠮷');
    expect(rangeAt(p, 9, 10)).toBeNull();
    p.remove();
  });
});
