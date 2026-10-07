import type { SearchMode } from './types';

const HAN = /[㐀-䶿一-鿿豈-﫿]/u;
const HANGUL = /[ᄀ-ᇿ㄰-㆏가-힯ꥠ-꥿ힰ-퟿]/;
const LATIN = /[A-Za-zÀ-ɏ]/;

export function isHan(ch: string): boolean { return HAN.test(ch); }
export function hanChars(s: string): string[] { return [...s].filter(isHan); }

/** Decide how a query is searched. Han anywhere wins, then Hangul, then Latin. */
export function detectMode(q: string): SearchMode {
  const s = q.trim();
  if (!s) return 'empty';
  if (HAN.test(s)) return 'han';
  if (HANGUL.test(s)) return 'hangul';
  if (LATIN.test(s) || /\p{L}|\p{N}/u.test(s)) return 'latin';
  return 'empty';
}

/** Key used for hw_norm lookups: strip '-', '^', spaces, '·'. */
export function normHeadword(s: string): string { return s.replace(/[-^\s·]/g, ''); }

/** FTS5 MATCH expression: quoted terms, last one prefix-matched. */
export function ftsQuery(q: string): string | null {
  const toks = q.toLowerCase().match(/[\p{L}\p{N}]+/gu);
  if (!toks) return null;
  return toks.map((t, i) => `"${t}"${i === toks.length - 1 ? '*' : ''}`).join(' ');
}

/** Crude stem for example-sentence lookups (먹다 → 먹, 학교 → 학교). */
export function stemOf(headword: string): string {
  const h = normHeadword(headword);
  return h.length > 2 && h.endsWith('다') ? h.slice(0, -1) : h;
}
