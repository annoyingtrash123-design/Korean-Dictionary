import { describe, expect, it } from 'vitest';
import { distractors, gapQuestion, meaningQuestion, choQuestion } from './quiz';
import type { GameWord } from '../types';

const w = (headword: string, gloss: string, pos = 'noun', examples: string[] = []): GameWord => ({ source: 'krdict', id: 1, headword, pos, gloss, examples });
const POOL = [w('학교', 'school', 'noun', ['요즘은 많은 학교에서 한국어를 배울 수 있다.']), w('학생', 'student'), w('선생님', 'teacher'), w('사랑', 'love'), w('가다', 'to go', 'verb', ['내일 시장에 갈 거예요.', '친구와 같이 학교에 가요.']), w('먹다', 'to eat', 'verb'), w('보다', 'to see', 'verb'), w('오다', 'to come', 'verb'), w('교실', 'school', 'noun')];

describe('games: questions', () => {
  it('distractors differ in headword and meaning, same pos first', () => {
    const d = distractors(POOL[0], POOL, 3);
    expect(d).toHaveLength(3);
    expect(d.every((x) => x.pos === 'noun')).toBe(true);
    expect(d.some((x) => x.headword === '교실')).toBe(false);   // same meaning "school"
  });
  it('meaning question has one right answer', () => {
    const q = meaningQuestion(POOL[0], POOL, 'ko-en');
    expect(q.prompt).toBe('학교');
    expect(q.choices.filter((c) => c.correct).map((c) => c.label)).toEqual(['school']);
    expect(q.choices).toHaveLength(4);
    const r = meaningQuestion(POOL[0], POOL, 'en-ko');
    expect(r.choices.filter((c) => c.correct).map((c) => c.label)).toEqual(['학교']);
  });
  it('gap keeps the particle of a noun', () => {
    const q = gapQuestion(POOL[0], POOL)!;
    expect(q.before).toBe('요즘은 많은 ');
    expect(q.after).toBe('에서 한국어를 배울 수 있다.');
  });
  it('gap blanks an inflected verb', () => {
    const q = gapQuestion(POOL[4], POOL, () => 0)!;
    expect(q).not.toBeNull();
    expect(q.before + '___' + q.after).toMatch(/___/);
    expect(q.sentence.replace(q.before, '').replace(q.after, '')).toMatch(/^가|^갈/);
  });
  it('no example, no gap question', () => {
    expect(gapQuestion(POOL[1], POOL)).toBeNull();
  });
  it('초성', () => {
    expect(choQuestion(POOL[2])!.cho).toBe('ㅅㅅㄴ');
    expect(choQuestion(w('학', 'crane'))).toBeNull();
  });
});
