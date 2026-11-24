import { describe, expect, it } from 'vitest';
import type { Confusable, WritingExpr } from '../db/types';
import { confusablePath, confusableTitle, filterConfusables, filterWriting, firstSentence, GROUP_USAGE, parseUsageId, usagePath, usageSummaries, writingGroups } from './study-usage';
import { parseStudyId, studyId, summariesOf } from './study-lists';
import { sourceOf } from './list-sources';

const w = (id: string, group: string, expr: string, meaning = ''): WritingExpr => ({
  id, group, expr, meaning, register: 'Written, formal.', examples: [{ ko: '예문이다.', en: 'An example.' }], entry: null, ai: true,
});
const rows = [w('W1', 'Reasons', '-기 때문이다', 'the reason is that'), w('W2', 'Reasons', '-(으)로 인해', 'because of'), w('W3', 'Conclusion', '결론적으로', 'in conclusion')];

const c = (id: string, kind: Confusable['kind'], words: [string, boolean][], explanation: string, mistake = 'm'): Confusable => ({
  id, kind, explanation, mistake, ai: true,
  words: words.map(([word, wrong]) => ({ word, wrong, ko: wrong ? '-' : `${word}.`, en: wrong ? '-' : 'x', entry: null })),
});
const groups = [
  c('C1', 'meaning', [['바라다', false], ['원하다', false]], '“바라다” is to hope. “원하다” is to want.', 'Writing “바래요”.'),
  c('C2', 'spelling', [['바람', false], ['바램', true]], 'The noun is “바람”.'),
  c('C3', 'grammar', [['-(으)로서', false], ['-(으)로써', false]], 'Status or means.'),
];

describe('usage list ids and overview rows', () => {
  it('round-trip through the study ids, and belong to the study source', () => {
    expect(parseUsageId('study:usage:writing')).toBe('writing');
    expect(parseUsageId('study:usage:confusable')).toBe('confusable');
    expect(parseUsageId('study:usage:other')).toBeUndefined();
    expect(parseStudyId('study:usage:writing')).toEqual({ kind: 'usage', list: 'writing' });
    expect(studyId({ kind: 'usage', list: 'confusable' })).toBe('study:usage:confusable');
    expect(sourceOf('study:usage:writing')?.id).toBe('study');
  });
  it('make a "Writing & usage" group with the two lists, only where the pack has rows', () => {
    const s = usageSummaries({ writing: 162, confusables: 100 });
    expect(s.map((x) => [x.name, x.count, x.unit, x.group])).toEqual([
      ['쓰기 expressions (TOPIK writing)', 162, 'expression', GROUP_USAGE],
      ['Confusable words', 100, 'group', GROUP_USAGE],
    ]);
    expect(usageSummaries({ writing: 0, confusables: 5 }).map((x) => x.id)).toEqual(['study:usage:confusable']);
    expect(usageSummaries(undefined)).toEqual([]);
    expect(usageSummaries({ writing: 0, confusables: 0 })).toEqual([]);
  });
  it('are part of the study overview after the other groups, and absent from an older overview', () => {
    const o = { vocab: [], grammar: [], topics: [], usage: { writing: 3, confusables: 2 } };
    expect(summariesOf(o).map((x) => x.group)).toEqual([GROUP_USAGE, GROUP_USAGE]);
    expect(summariesOf({ vocab: [], grammar: [], topics: [] })).toEqual([]);
  });
});

describe('쓰기 expressions', () => {
  it('group by function in file order', () => {
    expect(writingGroups(rows).map((g) => [g.name, g.rows.length])).toEqual([['Reasons', 2], ['Conclusion', 1]]);
    expect(writingGroups([])).toEqual([]);
  });
  it('filter on expression, meaning, group and sentences, ignoring hyphens, spaces and parentheses', () => {
    expect(filterWriting(rows, '').length).toBe(3);
    expect(filterWriting(rows, '때문').map((r) => r.id)).toEqual(['W1']);
    expect(filterWriting(rows, '으로인해').map((r) => r.id)).toEqual(['W2']);
    expect(filterWriting(rows, 'In Conclusion').map((r) => r.id)).toEqual(['W3']);
    expect(filterWriting(rows, 'reasons').map((r) => r.id)).toEqual(['W1', 'W2']);
    expect(filterWriting(rows, 'zzz')).toEqual([]);
  });
});

describe('confusable groups', () => {
  it('title is the correct words only', () => {
    expect(confusableTitle(groups[0])).toBe('바라다 · 원하다');
    expect(confusableTitle(groups[1])).toBe('바람');
  });
  it('filter by kind and by text over words, explanation and mistake', () => {
    expect(filterConfusables(groups, 'all', '').length).toBe(3);
    expect(filterConfusables(groups, 'spelling', '').map((g) => g.id)).toEqual(['C2']);
    expect(filterConfusables(groups, 'all', '바래요').map((g) => g.id)).toEqual(['C1']);
    expect(filterConfusables(groups, 'all', '바램').map((g) => g.id)).toEqual(['C2']);
    expect(filterConfusables(groups, 'meaning', '바램')).toEqual([]);
    expect(filterConfusables(groups, 'grammar', '로서').map((g) => g.id)).toEqual(['C3']);
  });
  it('first sentence of an explanation ends at its first full stop', () => {
    expect(firstSentence('“a” is x. “b” is y.')).toBe('“a” is x.');
    expect(firstSentence('No full stop')).toBe('No full stop');
  });
  it('open as ?g= on the list route, and links fall back to a search', () => {
    expect(confusablePath('/lists/study%3Ausage%3Aconfusable', 'C007')).toBe('/lists/study%3Ausage%3Aconfusable?g=C007');
    expect(usagePath(null, '바라다')).toBe('/word/%EB%B0%94%EB%9D%BC%EB%8B%A4');
    expect(usagePath({ source: 'krdict', id: 5, homonym: null, pos: null }, '바라다')).toContain('/entry/krdict/5');
  });
});
