import { beforeEach, describe, expect, it } from 'vitest';
import type { StudyGrammar, StudyOverview, StudyWord } from '../db/types';
import { GROUP_GRAMMAR, GROUP_TOPICS, GROUP_VOCAB, grammarRow, parseStudyId, studyId, studyMemory, subOf, summariesOf, topOf, wordRow } from './study-lists';
import { sourceOf } from './list-sources';

const overview: StudyOverview = {
  vocab: [{ level: 1, count: 735 }, { level: 6, count: 2575 }],
  grammar: [{ level: 1, count: 45 }, { level: 2, count: 0 }],
  topics: [{ category: '인간 > 감각', count: 100 }, { category: '인간 > 감정', count: 50 }, { category: '자연 > 지형', count: 7 }],
};

describe('study list ids', () => {
  it('round-trip, and reject anything else', () => {
    for (const r of [{ kind: 'vocab', level: 6 }, { kind: 'grammar', level: 1 }, { kind: 'topic', category: '인간 > 감각' }, { kind: 'topic', category: '인간' }] as const) {
      expect(parseStudyId(studyId(r))).toEqual(r);
    }
    expect(studyId({ kind: 'vocab', level: 6 })).toBe('study:vocab:6');
    for (const bad of ['saved', 'study:vocab:7', 'study:vocab:x', 'study:other:1', 'study:topic:', 'topik:vocab:1']) expect(parseStudyId(bad)).toBeUndefined();
  });
  it('are owned by the study source, never by the user lists (which never contain ":")', () => {
    expect(sourceOf('study:vocab:6')?.id).toBe('study');
    expect(sourceOf('l1abc')?.id).toBe('user');
  });
  it('split a category into its top level and sub-category', () => {
    expect([topOf('인간 > 감각'), subOf('인간 > 감각'), topOf('인간'), subOf('인간')]).toEqual(['인간', '감각', '인간', '인간']);
  });
});

describe('overview rows', () => {
  const s = summariesOf(overview);
  it('label levels as the curriculum, never plain TOPIK, with the TOPIK hint in the tooltip', () => {
    const v = s.find((x) => x.id === 'study:vocab:6')!;
    expect(v).toMatchObject({ name: '6급 · 표준 교육과정', count: 2575, group: GROUP_VOCAB, unit: 'word', hint: 'National standard Korean curriculum level 6 (≈ TOPIK 6)' });
    for (const x of s) expect(x.name).not.toMatch(/TOPIK/);
  });
  it('list levels in order and skip empty ones', () => {
    expect(s.filter((x) => x.group === GROUP_VOCAB).map((x) => x.id)).toEqual(['study:vocab:1', 'study:vocab:6']);
    expect(s.filter((x) => x.group === GROUP_GRAMMAR).map((x) => [x.id, x.unit])).toEqual([['study:grammar:1', 'pattern']]);
  });
  it('fold categories into top-level topics with their sub-categories nested', () => {
    const tops = s.filter((x) => x.group === GROUP_TOPICS && !x.parent);
    expect(tops.map((t) => [t.name, t.count])).toEqual([['인간', 150], ['자연', 7]]);
    const subs = s.filter((x) => x.parent === 'study:topic:인간');
    expect(subs.map((x) => [x.id, x.name, x.fullName, x.count])).toEqual([
      ['study:topic:인간 > 감각', '감각', '인간 > 감각', 100], ['study:topic:인간 > 감정', '감정', '인간 > 감정', 50],
    ]);
  });
  it('are empty for a pack without the tables (the lists are hidden)', () => {
    expect(summariesOf(undefined)).toEqual([]);
    expect(summariesOf(null)).toEqual([]);
    expect(summariesOf({ vocab: [], grammar: [], topics: [] })).toEqual([]);
  });
});

describe('rows', () => {
  const w: StudyWord = { source: 'krdict', id: 42, headword: '가격', homonym: 1, hanja: '價格', pos: 'noun', level: 2, gloss: 'price' };
  it('a word row opens its entry by id and saves like any entry', () => {
    const r = wordRow(w);
    expect(r.path).toBe('/entry/krdict/42?hw=%EA%B0%80%EA%B2%A9');
    expect(r.item).toEqual({ kind: 'entry', source: 'krdict', headword: '가격', homonym: 1, pos: 'noun', hanja: '價格', gloss: 'price' });
    expect(r.level).toBe(2);
  });
  it('a grammar row shows the official form but saves the grammar row\'s pattern, opening the linked entry', () => {
    const g: StudyGrammar = { form: '-아서/어서', pattern: '-아서/어서2', summary_en: 'because', entry: { source: 'krdict', id: 7, homonym: null, pos: 'ending' } };
    const r = grammarRow(g, 3, 5);
    expect(r).toMatchObject({ title: '-아서/어서', gloss: 'because', path: '/entry/krdict/7?hw=%EA%B8%B0'.replace('%EA%B8%B0', encodeURIComponent('-아서/어서2')) });
    expect(r.item).toEqual({ kind: 'grammar', pattern: '-아서/어서2', summary: 'because', level: 3, ref: { source: 'krdict', homonym: null, pos: 'ending' } });
    // the official list's homonym numbers ("이라고1") are not part of the pattern shown
    expect(grammarRow({ form: '이라고1', pattern: '이라고', summary_en: null, entry: null }, 3, 0).title).toBe('이라고');
    // no linked entry: a search by the pattern
    expect(grammarRow({ form: '-기', pattern: '-기', summary_en: null, entry: null }, 1, 0)).toMatchObject({ path: `/word/${encodeURIComponent('-기')}`, gloss: '', item: { ref: null } });
  });
});

describe('remembered position', () => {
  beforeEach(() => localStorage.clear());
  it('remembers the last list and the first row of each', () => {
    expect([studyMemory.last(), studyMemory.row('study:vocab:6')]).toEqual([undefined, 0]);
    studyMemory.remember('study:vocab:6', 1204.7);
    studyMemory.remember('study:grammar:1', 3);
    expect([studyMemory.last(), studyMemory.row('study:vocab:6'), studyMemory.row('study:grammar:1')]).toEqual(['study:grammar:1', 1204, 3]);
  });
  it('survives damaged storage', () => {
    localStorage.setItem('kd.study.pos', '{nope');
    expect([studyMemory.last(), studyMemory.row('x')]).toEqual([undefined, 0]);
    localStorage.setItem('kd.study.pos', JSON.stringify({ last: 5, rows: { a: -3, b: 'x', c: 8 } }));
    expect([studyMemory.last(), studyMemory.row('a'), studyMemory.row('c')]).toEqual([undefined, 0, 8]);
  });
  it('keeps the file small', () => {
    for (let i = 0; i < 60; i++) studyMemory.remember(`study:topic:t${i}`, i);
    expect(Object.keys(JSON.parse(localStorage.getItem('kd.study.pos')!).rows).length).toBeLessThanOrEqual(40);
    expect(studyMemory.row('study:topic:t59')).toBe(59);
  });
});
