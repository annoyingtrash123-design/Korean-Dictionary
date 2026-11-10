import { describe, expect, it } from 'vitest';
import { dialogueTurns, speakable } from './examples';

describe('example sentences', () => {
  it('splits dictionary dialogues into speaker turns', () => {
    expect(dialogueTurns('가: 어디 가요?\n나: 학교에 가요.')).toEqual([{ who: '가', text: '어디 가요?' }, { who: '나', text: '학교에 가요.' }]);
    expect(dialogueTurns('학교에 가요.')).toBeNull();
    expect(dialogueTurns('첫 줄\n둘째 줄')).toBeNull();   // two lines but not a dialogue
  });
  it('reads dialogues without the speaker labels', () => {
    expect(speakable('가: 어디 가요?\n나: 학교에 가요.')).toBe('어디 가요? 학교에 가요.');
    expect(speakable('학교에 가요.')).toBe('학교에 가요.');
  });
});
