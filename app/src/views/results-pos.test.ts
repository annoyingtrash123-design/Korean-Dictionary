import { describe, expect, it } from 'vitest';
import { byPos, posBucket } from './Results';

describe('English results by part of speech', () => {
  it('buckets Korean POS labels', () => {
    expect(posBucket('noun')).toBe('noun');
    expect(posBucket('bound noun')).toBe('noun');
    expect(posBucket('auxiliary verb')).toBe('verb');
    expect(posBucket('determiner')).toBe('adj');
    expect(posBucket('idiom')).toBe('phrase');
    expect(posBucket(undefined)).toBe('other');
  });
  it('keeps rank order inside sections and orders sections by their best row', () => {
    const g = (h: string, pos: string) => ({ headword: h, pos: [pos] });
    const s = byPos([g('보고서', 'noun'), g('보고하다', 'verb'), g('보고', 'noun'), g('신고하다', 'verb'), g('보도', 'noun')]);
    expect(s.map((x) => x.id)).toEqual(['noun', 'verb']);
    expect(s[0].items.map((x) => x.headword)).toEqual(['보고서', '보고', '보도']);
    expect(s[1].label).toBe('Verbs');
  });
});
