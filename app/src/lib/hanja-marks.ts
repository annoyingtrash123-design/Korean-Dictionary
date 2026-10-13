// 표준국어대사전-style marks in a hanja field: ▽/▼ after a character mean it is read here with
// other than its usual sound (困難▽ 곤란: 難 is usually 난), × that it has no Korean reading.
// The marks are kept in the data (they are interesting) but never shown raw.

const MARKS = /[▽▼×]/g;

export interface HanjaPart { ch: string; alt: boolean }

/** The hanja without marks: use it to compare two entries' hanja. */
export const hanjaKey = (h?: string | null) => (h ?? '').replace(MARKS, '').trim();

export const hasMarks = (h?: string | null) => /[▽▼]/.test(h ?? '');

/** Characters of the field, each flagged when the dictionary marks a non-usual reading. */
export function hanjaParts(h: string): HanjaPart[] {
  const out: HanjaPart[] = [];
  for (const c of h) {
    if (c === '▽' || c === '▼') { if (out.length) out[out.length - 1].alt = true; }
    else if (c !== '×') out.push({ ch: c, alt: false });
  }
  return out;
}

const isHangulSyllable = (c: string) => c >= '가' && c <= '힣';

/** For each marked character, the syllable of the headword it is read as here: [{ ch, here }].
 *  Needs the hanja/hangul characters to line up one-to-one with the headword's syllables. */
export function altReadings(headword: string, hanja: string): { ch: string; here: string }[] {
  const parts = hanjaParts(hanja).filter((p) => p.ch.trim() && !/[-―—←]/.test(p.ch));
  const syl = [...headword].filter(isHangulSyllable);
  if (parts.length !== syl.length) return [];
  return parts.flatMap((p, i) => (p.alt ? [{ ch: p.ch, here: syl[i] }] : []));
}

/** First Hangul reading listed for a character ("난,란" -> "난"). */
export const usualReading = (readings?: string | null) => (readings ?? '').split(/[,;\s]+/).find((r) => /^[가-힣]$/.test(r));
