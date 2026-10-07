/**
 * Hangul syllable utilities: compose / decompose precomposed syllables
 * (U+AC00..U+D7A3) into initial / medial / final jamo indices.
 * Dependency-free. All "text" helpers operate on the LAST syllable of the
 * string unless stated otherwise.
 */

const BASE = 0xac00;
const LAST = 0xd7a3;

/** Initial consonants (choseong), index 0..18. */
export const CHOSEONG = [
  'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ',
  'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
] as const;

/** Vowels (jungseong), index 0..20. */
export const JUNGSEONG = [
  'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ',
  'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
] as const;

/** Final consonants (jongseong), index 0..27; index 0 means "no batchim". */
export const JONGSEONG = [
  '', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ',
  'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ',
  'ㅍ', 'ㅎ',
] as const;

/** Compound batchim -> its two parts, e.g. 'ㄺ' -> ['ㄹ', 'ㄱ']. */
export const COMPOUND_FINALS: Record<string, [string, string]> = {
  'ㄳ': ['ㄱ', 'ㅅ'], 'ㄵ': ['ㄴ', 'ㅈ'], 'ㄶ': ['ㄴ', 'ㅎ'], 'ㄺ': ['ㄹ', 'ㄱ'],
  'ㄻ': ['ㄹ', 'ㅁ'], 'ㄼ': ['ㄹ', 'ㅂ'], 'ㄽ': ['ㄹ', 'ㅅ'], 'ㄾ': ['ㄹ', 'ㅌ'],
  'ㄿ': ['ㄹ', 'ㅍ'], 'ㅀ': ['ㄹ', 'ㅎ'], 'ㅄ': ['ㅂ', 'ㅅ'],
};

export interface Jamo {
  initial: number;
  medial: number;
  final: number;
}

/** True for a single precomposed Hangul syllable (가..힣). */
export function isSyllable(ch: string): boolean {
  if (!ch || ch.length !== 1) return false;
  const c = ch.charCodeAt(0);
  return c >= BASE && c <= LAST;
}

/** True for a compatibility jamo letter (ㄱ..ㅣ). */
export function isJamo(ch: string): boolean {
  if (!ch || ch.length !== 1) return false;
  const c = ch.charCodeAt(0);
  return c >= 0x3131 && c <= 0x3163;
}

/** True if the string is non-empty and consists only of syllables / jamo. */
export function isHangul(text: string): boolean {
  if (!text) return false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (!isSyllable(ch) && !isJamo(ch)) return false;
  }
  return true;
}

/** True if the string contains at least one Hangul syllable. */
export function hasHangul(text: string): boolean {
  for (let i = 0; i < text.length; i++) if (isSyllable(text[i])) return true;
  return false;
}

/** Split a single syllable into jamo indices; null if not a syllable. */
export function decompose(ch: string): Jamo | null {
  if (!isSyllable(ch)) return null;
  const n = ch.charCodeAt(0) - BASE;
  return { initial: Math.floor(n / 588), medial: Math.floor((n % 588) / 28), final: n % 28 };
}

/** Build a syllable from jamo indices. */
export function compose(initial: number, medial: number, final = 0): string {
  if (initial < 0 || initial > 18 || medial < 0 || medial > 20 || final < 0 || final > 27) {
    throw new RangeError(`invalid jamo indices ${initial},${medial},${final}`);
  }
  return String.fromCharCode(BASE + (initial * 21 + medial) * 28 + final);
}

/** Index of an initial consonant letter, or -1. */
export function initialIndex(jamo: string): number {
  return (CHOSEONG as readonly string[]).indexOf(jamo);
}
/** Index of a vowel letter, or -1. */
export function medialIndex(jamo: string): number {
  return (JUNGSEONG as readonly string[]).indexOf(jamo);
}
/** Index of a final consonant letter ('' -> 0), or -1. */
export function finalIndex(jamo: string): number {
  return (JONGSEONG as readonly string[]).indexOf(jamo);
}

/** The last character of a string, or ''. */
export function lastChar(text: string): string {
  return text ? text[text.length - 1] : '';
}

/** Everything except the last character. */
export function allButLast(text: string): string {
  return text.slice(0, -1);
}

/** Does the last syllable have a final consonant (batchim)? */
export function hasBatchim(text: string): boolean {
  const d = decompose(lastChar(text));
  return !!d && d.final !== 0;
}

/** Final consonant letter of the last syllable ('' if none / not a syllable). */
export function finalOf(text: string): string {
  const d = decompose(lastChar(text));
  return d ? JONGSEONG[d.final] : '';
}

/** Initial consonant letter of the last syllable ('' if not a syllable). */
export function initialOf(text: string): string {
  const d = decompose(lastChar(text));
  return d ? CHOSEONG[d.initial] : '';
}

/** Vowel letter of the last syllable ('' if not a syllable). */
export function medialOf(text: string): string {
  const d = decompose(lastChar(text));
  return d ? JUNGSEONG[d.medial] : '';
}

function replaceLast(text: string, f: (d: Jamo) => Jamo | null): string {
  const d = decompose(lastChar(text));
  if (!d) return text;
  const nd = f(d);
  if (!nd) return text;
  return allButLast(text) + compose(nd.initial, nd.medial, nd.final);
}

/** Replace/add the final consonant of the last syllable. */
export function withFinal(text: string, jamo: string): string {
  const idx = finalIndex(jamo);
  if (idx < 0) return text;
  return replaceLast(text, (d) => ({ ...d, final: idx }));
}

/** Remove the final consonant of the last syllable. */
export function withoutFinal(text: string): string {
  return replaceLast(text, (d) => ({ ...d, final: 0 }));
}

/** Replace the vowel of the last syllable. */
export function withMedial(text: string, jamo: string): string {
  const idx = medialIndex(jamo);
  if (idx < 0) return text;
  return replaceLast(text, (d) => ({ ...d, medial: idx }));
}

/** Replace the initial consonant of the last syllable. */
export function withInitial(text: string, jamo: string): string {
  const idx = initialIndex(jamo);
  if (idx < 0) return text;
  return replaceLast(text, (d) => ({ ...d, initial: idx }));
}

/** Decompose a whole string into a flat jamo string ('한' -> 'ㅎㅏㄴ'). */
export function toJamo(text: string): string {
  let out = '';
  for (const ch of text) {
    const d = decompose(ch);
    if (!d) out += ch;
    else out += CHOSEONG[d.initial] + JUNGSEONG[d.medial] + JONGSEONG[d.final];
  }
  return out;
}
