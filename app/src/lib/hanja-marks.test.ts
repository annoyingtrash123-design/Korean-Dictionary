import { describe, expect, it } from 'vitest';
import { altReadings, hanjaKey, hanjaParts, usualReading } from './hanja-marks';

describe('hanja marks', () => {
  it('compares without the marks', () => {
    expect(hanjaKey('卷▽煙▽')).toBe('卷煙');
    expect(hanjaKey('困難▽하다')).toBe('困難하다');
  });
  it('flags the marked characters', () => {
    expect(hanjaParts('困難▽')).toEqual([{ ch: '困', alt: false }, { ch: '難', alt: true }]);
  });
  it('finds the syllable a marked character is read as', () => {
    expect(altReadings('곤란', '困難▽')).toEqual([{ ch: '難', here: '란' }]);
    expect(altReadings('권련', '卷▽煙▽')).toEqual([{ ch: '卷', here: '권' }, { ch: '煙', here: '련' }]);
    expect(altReadings('곤란하다', '困難▽하다')).toEqual([{ ch: '難', here: '란' }]);
    expect(altReadings('가 나', '加那▽陀')).toEqual([]);   // does not line up: say nothing
  });
  it('takes the first Hangul reading', () => {
    expect(usualReading('난,란')).toBe('난');
    expect(usualReading('nan')).toBeUndefined();
  });
});
