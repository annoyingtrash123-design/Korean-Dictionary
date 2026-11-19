import { describe, expect, it } from 'vitest';
import { curriculumHint, curriculumLabel, dayNumber } from './home-day';

describe('dayNumber', () => {
  it('counts local calendar days since 2026-01-01', () => {
    expect(dayNumber(new Date(2026, 0, 1, 0, 0, 0))).toBe(0);
    expect(dayNumber(new Date(2026, 0, 1, 23, 59, 59))).toBe(0);
    expect(dayNumber(new Date(2026, 0, 2, 0, 0, 0))).toBe(1);
    expect(dayNumber(new Date(2026, 11, 31, 12))).toBe(364);
    expect(dayNumber(new Date(2027, 0, 1, 12))).toBe(365);
  });
  it('uses the local date, not the UTC date', () => {
    // built from local fields: whatever zone the machine is in, 23:30 and 00:30 are different days
    const late = new Date(2026, 5, 10, 23, 30), early = new Date(2026, 5, 11, 0, 30);
    expect(dayNumber(early) - dayNumber(late)).toBe(1);
    expect(dayNumber(new Date(2026, 5, 10, 0, 1))).toBe(dayNumber(late));
  });
  it('counts calendar days across clock changes (not 24 h blocks)', () => {
    for (const [m, d] of [[2, 28], [2, 29], [2, 30], [9, 24], [9, 25], [9, 26], [10, 1], [10, 2]]) {
      expect(dayNumber(new Date(2026, m, d + 1, 12)) - dayNumber(new Date(2026, m, d, 12))).toBe(1);
      expect(dayNumber(new Date(2026, m, d + 1, 0, 30)) - dayNumber(new Date(2026, m, d, 23, 30))).toBe(1);
    }
  });
  it('never goes negative (a clock set before 2026)', () => {
    expect(dayNumber(new Date(2020, 0, 1))).toBe(0);
  });
});

describe('level labels', () => {
  it('name the curriculum, not TOPIK, in the label', () => {
    expect(curriculumLabel(4)).toBe('4급 · 표준 교육과정');
    expect(curriculumLabel(5)).toBe('5급 · 표준 교육과정');
    expect(curriculumLabel(6)).not.toMatch(/TOPIK/);
    expect(curriculumHint(5)).toBe('National standard Korean curriculum level 5 (≈ TOPIK 5)');
  });
});
