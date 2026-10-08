import type { EnKoPhrase, EnKoSense } from './types';

/** Phrases shown before "Show all". */
export const PHRASES_COLLAPSED = 8;

const POS_SHORT: Record<string, string> = {
  noun: 'n.', verb: 'v.', adjective: 'adj.', adverb: 'adv.', preposition: 'prep.', conjunction: 'conj.',
  interjection: 'interj.', pronoun: 'pron.', numeral: 'num.', determiner: 'det.', 'proper noun': 'prop. n.',
  phrase: 'phrase', prep_phrase: 'phrase', proverb: 'proverb', particle: 'particle', article: 'art.',
};
/** Compact part-of-speech label for the Wiktionary blocks ("noun" -> "n."). */
export const posShort = (pos: string) => POS_SHORT[pos] ?? pos;

/** Senses grouped by part of speech, in the engine's order (one heading per pos). */
export function groupTranslations(senses: EnKoSense[]): { pos: string; senses: EnKoSense[] }[] {
  const out: { pos: string; senses: EnKoSense[] }[] = [];
  for (const s of senses) {
    const g = out.find((x) => x.pos === s.pos);
    if (g) g.senses.push(s); else out.push({ pos: s.pos, senses: [s] });
  }
  return out;
}

/** Translation senses shown before "More senses". */
export const SENSES_COLLAPSED = 5;

/** First `n` items unless open, or unless hiding would save no more than two rows. */
export function collapse<T>(list: T[], open: boolean, n: number): { shown: T[]; hidden: number } {
  if (open || list.length <= n + 2) return { shown: list, hidden: 0 };
  return { shown: list.slice(0, n), hidden: list.length - n };
}
/** Phrases to render: all when open or short enough, else the first PHRASES_COLLAPSED. */
export const visiblePhrases = (list: EnKoPhrase[], open: boolean) => collapse(list, open, PHRASES_COLLAPSED);
