import { describe, expect, it } from 'vitest';
import { headingsOf, hidden, sectionOf } from './reader-sections';

const long = (t: string) => t + '가'.repeat(1000);
const doc = ['intro', '## 1', long('a'), long('b'), '## 2', long('c'), '## 3'];

describe('reader sections', () => {
  it('finds headings; a single heading is not folded', () => {
    expect(headingsOf(doc)).toEqual([1, 4, 6]);
    expect(headingsOf(['## title', long('a'), long('b')])).toEqual([]);
    expect(headingsOf(['a', 'b'])).toEqual([]);
    // a short poem with "original / modern" version headings is not folded
    expect(headingsOf(['## 원문', '님은 갔습니다', '## 현대어', '님은 갔습니다'])).toEqual([]);
  });
  it('maps paragraphs to sections', () => {
    const h = headingsOf(doc);
    expect([0, 1, 2, 3, 4, 5, 6].map((n) => sectionOf(h, n))).toEqual([-1, 1, 1, 1, 4, 4, 6]);
  });
  it('hides body paragraphs of closed sections only', () => {
    const h = headingsOf(doc);
    const open = new Set([4]);
    expect([0, 1, 2, 3, 4, 5, 6].map((n) => hidden(h, open, n))).toEqual([false, false, true, true, false, false, false]);
  });
});
