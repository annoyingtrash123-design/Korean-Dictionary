import { describe, expect, it } from 'vitest';
import { lessonForm, parseLessons } from './lesson';

const body = { label: 'x y', meaning_en: 'm', usage: 'u', examples: [{ ko: '가.', en: 'a.', ctx: 'essay' }], compare: [], pitfalls: [] };

describe('parseLessons', () => {
  it('parses the JSON body and keeps the row level and form', () => {
    const [l] = parseLessons([{ level: 6, form: '-거들랑1', body: JSON.stringify(body) }]);
    expect(l).toMatchObject({ level: 6, form: '-거들랑1', label: 'x y', meaning_en: 'm' });
    expect(l.examples).toHaveLength(1);
  });
  it('fills missing compare / pitfalls lists and skips a damaged body', () => {
    const { compare: _c, pitfalls: _p, ...thin } = body;
    const out = parseLessons([
      { level: 6, form: 'a', body: JSON.stringify(thin) },
      { level: 6, form: 'b', body: '{not json' },
      { level: 6, form: 'c', body: '{"hello":1}' },
    ]);
    expect(out.map((l) => l.form)).toEqual(['a']);
    expect(out[0].compare).toEqual([]);
    expect(out[0].pitfalls).toEqual([]);
  });
});

describe('lessonForm', () => {
  it('drops the homonym digit and the list comma but keeps the hyphen', () => {
    expect(lessonForm('-거들랑1')).toBe('-거들랑');
    expect(lessonForm('-을망정,')).toBe('-을망정');
    expect(lessonForm('-기 일쑤이다')).toBe('-기 일쑤이다');
    expect(lessonForm('이라고2')).toBe('이라고');
  });
});
