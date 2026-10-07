import { describe, expect, it } from 'vitest';
import { detectMode, ftsQuery, normHeadword, stemOf, hanChars } from './search-mode';

describe('detectMode', () => {
  it('detects scripts', () => {
    expect(detectMode('학교')).toBe('hangul');
    expect(detectMode('갔어요')).toBe('hangul');
    expect(detectMode('ㅎㄱ')).toBe('hangul');
    expect(detectMode('school')).toBe('latin');
    expect(detectMode('to eat')).toBe('latin');
    expect(detectMode('學')).toBe('han');
    expect(detectMode('學校')).toBe('han');
    expect(detectMode('  ')).toBe('empty');
    expect(detectMode('')).toBe('empty');
  });
  it('Han wins over Hangul in mixed input', () => {
    expect(detectMode('學교')).toBe('han');
    expect(hanChars('學교校')).toEqual(['學', '校']);
  });
});
describe('helpers', () => {
  it('normalises headwords', () => {
    expect(normHeadword('-아서/어서')).toBe('아서/어서');
    expect(normHeadword('눈치가 빠르다')).toBe('눈치가빠르다');
    expect(normHeadword('^먹다')).toBe('먹다');
  });
  it('builds FTS queries with a prefix on the last token', () => {
    expect(ftsQuery('to eat')).toBe('"to" "eat"*');
    expect(ftsQuery('school')).toBe('"school"*');
    expect(ftsQuery('!!!')).toBeNull();
  });
  it('stems verbs for example search', () => {
    expect(stemOf('먹다')).toBe('먹다');
    expect(stemOf('사랑하다')).toBe('사랑하');
    expect(stemOf('학교')).toBe('학교');
  });
});
