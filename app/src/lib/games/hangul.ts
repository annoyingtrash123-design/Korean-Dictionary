// Hangul helpers for the games: initial consonants (초성), 두음법칙 for 끝말잇기, answer matching.

const CHO = 'ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ';
const BASE = 0xac00, LAST = 0xd7a3;
const isSyl = (c: string) => { const n = c.codePointAt(0)!; return n >= BASE && n <= LAST; };

/** Initial consonants of each syllable (학교 -> ㅎㄱ); other characters are kept. */
export function choseong(word: string): string {
  return [...word].map((c) => (isSyl(c) ? CHO[Math.floor((c.codePointAt(0)! - BASE) / 588)] : c)).join('');
}

/** Hangul syllables only (drops spaces, hyphens, homonym digits). */
export const syllables = (word: string) => [...word.normalize('NFC')].filter(isSyl).join('');

export const lastSyllable = (word: string) => { const s = [...syllables(word)]; return s[s.length - 1] ?? ''; };
export const firstSyllable = (word: string) => [...syllables(word)][0] ?? '';

/** Syllables a word may start with after `syl` under 두음법칙: 력 -> 력, 역; 녀 -> 녀, 여; 라 -> 라, 나. */
export function dueumStarts(syl: string): string[] {
  if (!isSyl(syl)) return [syl];
  const n = syl.codePointAt(0)! - BASE;
  const cho = Math.floor(n / 588), jung = Math.floor((n % 588) / 28), jong = n % 28;
  const make = (c: number) => String.fromCodePoint(BASE + c * 588 + jung * 28 + jong);
  // vowels ㅑ ㅕ ㅖ ㅛ ㅠ ㅣ (jung indices 2, 6, 7, 12, 17, 20) take ㅇ after ㄴ/ㄹ
  const iotized = [2, 6, 7, 12, 17, 20].includes(jung);
  const out = [syl];
  if (cho === 5) out.push(make(iotized ? 11 : 2));      // ㄹ -> ㅇ / ㄴ
  else if (cho === 2 && iotized) out.push(make(11));    // ㄴ -> ㅇ
  return out;
}

/** Does `next` legally follow `prev` in a word chain? */
export function chainFollows(prev: string, next: string): boolean {
  return dueumStarts(lastSyllable(prev)).includes(firstSyllable(next));
}

/** Typed answer vs headword: ignore spaces, hyphens, homonym digits and Unicode form. */
export const sameKorean = (typed: string, headword: string) => syllables(typed) !== '' && syllables(typed) === syllables(headword);

/** First gloss item, short enough for a button: "to hold; to enter" -> "to hold". */
export function shortGloss(gloss: string, max = 40): string {
  const first = gloss.split(/;\s*/)[0].trim();
  return first.length > max ? first.slice(0, max - 1).trimEnd() + '…' : first;
}
