import { describe, expect, it } from 'vitest';
import { chainFollows, choseong, dueumStarts, lastSyllable, sameKorean, shortGloss } from './hangul';

describe('games: hangul', () => {
  it('initial consonants', () => {
    expect(choseong('학교')).toBe('ㅎㄱ');
    expect(choseong('선생님')).toBe('ㅅㅅㄴ');
    expect(choseong('까치')).toBe('ㄲㅊ');
  });
  it('두음법칙', () => {
    expect(dueumStarts('력')).toEqual(['력', '역']);
    expect(dueumStarts('락')).toEqual(['락', '낙']);
    expect(dueumStarts('녀')).toEqual(['녀', '여']);
    expect(dueumStarts('나')).toEqual(['나']);
    expect(dueumStarts('교')).toEqual(['교']);
  });
  it('word chain', () => {
    expect(lastSyllable('학교')).toBe('교');
    expect(chainFollows('학교', '교실')).toBe(true);
    expect(chainFollows('노력', '역사')).toBe(true);
    expect(chainFollows('노력', '사과')).toBe(false);
  });
  it('typed answers', () => {
    expect(sameKorean(' 학 교 ', '학교')).toBe(true);
    expect(sameKorean('학교', '학교02')).toBe(true);
    expect(sameKorean('학생', '학교')).toBe(false);
    expect(sameKorean('', '학교')).toBe(false);
  });
  it('short gloss', () => {
    expect(shortGloss('to hold; to enter')).toBe('to hold');
  });
});
