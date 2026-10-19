// Smart review (Leitner boxes): a missed word comes back in the next round, a known word after
// 1, 3, 7, 16, then 35 days. Kept on the device in IndexedDB; keyed by headword + part of speech
// (entry ids change between dictionary builds, words do not).
import { get, set } from 'idb-keyval';
import { userDb } from '../idb';
import type { GameWord } from '../types';
import { syllables } from './hangul';

export const INTERVAL_DAYS = [0, 1, 3, 7, 16, 35];
const DAY = 86_400_000;

export interface Card { word: GameWord; box: number; due: number; seen: number; right: number }
export type Deck = Record<string, Card>;

export const cardKey = (w: Pick<GameWord, 'headword' | 'pos'>) => `${syllables(w.headword) || w.headword}|${w.pos ?? ''}`;

/** The card after an answer: right -> next box, wrong -> back to box 0 (due now). */
export function review(card: Card | undefined, word: GameWord, correct: boolean, now: number): Card {
  const box = correct ? Math.min((card?.box ?? 0) + 1, INTERVAL_DAYS.length - 1) : 0;
  return { word, box, due: now + INTERVAL_DAYS[box] * DAY, seen: (card?.seen ?? 0) + 1, right: (card?.right ?? 0) + (correct ? 1 : 0) };
}

/** Words for a round of `n` from `pool`: due cards first (most overdue first), then words never
 *  seen, then the rest (soonest due first). Words learned well and not due come last. */
export function pickRound(pool: GameWord[], deck: Deck, n: number, now: number): GameWord[] {
  const seen = new Set<string>();
  const uniq = pool.filter((w) => { const k = cardKey(w); if (seen.has(k)) return false; seen.add(k); return true; });
  const card = (w: GameWord) => deck[cardKey(w)];
  const due = uniq.filter((w) => card(w) && card(w)!.due <= now).sort((a, b) => card(a)!.due - card(b)!.due);
  const fresh = uniq.filter((w) => !card(w));
  const later = uniq.filter((w) => card(w) && card(w)!.due > now).sort((a, b) => card(a)!.due - card(b)!.due);
  return [...due, ...fresh, ...later].slice(0, n);
}

/** Due cards from the whole deck (e.g. to top up a level's pool with words to review). */
export const dueWords = (deck: Deck, now: number, filter: (w: GameWord) => boolean = () => true) =>
  Object.values(deck).filter((c) => c.due <= now && filter(c.word)).sort((a, b) => a.due - b.due).map((c) => c.word);

// ---- daily streak --------------------------------------------------------------------------

export const dayKey = (t: number) => { const d = new Date(t); return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`; };

/** Consecutive days played, ending today (or yesterday, when today has not been played yet). */
export function streak(days: string[], now: number): number {
  const set = new Set(days);
  let t = now;
  if (!set.has(dayKey(t))) t -= DAY;
  let n = 0;
  while (set.has(dayKey(t))) { n++; t -= DAY; }
  return n;
}

// ---- persistence ---------------------------------------------------------------------------

export const loadDeck = () => get<Deck>('games:deck', userDb()).then((d) => d ?? {}).catch(() => ({} as Deck));
export const loadDays = () => get<string[]>('games:days', userDb()).then((d) => d ?? []).catch(() => [] as string[]);

/** Record a finished round's answers and mark today as played. */
export async function saveRound(answers: { word: GameWord; correct: boolean }[], now = Date.now()): Promise<void> {
  const deck = await loadDeck();
  for (const a of answers) deck[cardKey(a.word)] = review(deck[cardKey(a.word)], a.word, a.correct, now);
  const days = await loadDays();
  const today = dayKey(now);
  await Promise.all([
    set('games:deck', deck, userDb()),
    days.includes(today) ? Promise.resolve() : set('games:days', [...days, today].slice(-400), userDb()),
  ]).catch(() => undefined);
}
