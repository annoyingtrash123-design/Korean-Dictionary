// Where a round's words come from: a TOPIK level, recent lookups, or a Reader text.
import { db } from '../../db/client';
import type { GameWord } from '../types';
import { history, loadHistory } from '../history';
import { dueWords, loadDeck } from './srs';

export type Source = { kind: 'level'; level: 1 | 2 | 3 } | { kind: 'recent' } | { kind: 'text'; id: string; title: string };

export const LEVELS = [
  { level: 1 as const, ko: '초급', en: 'Beginner' },
  { level: 2 as const, ko: '중급', en: 'Intermediate' },
  { level: 3 as const, ko: '고급', en: 'Advanced' },
];

export function sourceLabel(s: Source): string {
  if (s.kind === 'level') { const l = LEVELS.find((x) => x.level === s.level)!; return `TOPIK ${l.ko} · ${l.en}`; }
  if (s.kind === 'recent') return 'Recent lookups';
  return s.title;
}

const textCache = new Map<string, GameWord[]>();
const MAX_TOKENS = 160;

/** Content words of a Reader text: each distinct eojeol looked up as the Reader does, kept when
 *  it has an English-glossed dictionary word. */
async function textWords(id: string): Promise<GameWord[]> {
  const hit = textCache.get(id);
  if (hit) return hit;
  const doc = await db.getText(id);
  if (!doc) return [];
  const tokens: string[] = [];
  const seen = new Set<string>();
  for (const p of doc.paragraphs) {
    for (const t of (p.modern || p.orig).split(/\s+/)) {
      const k = t.replace(/[^가-힣]/g, '');
      if (k.length < 2 || seen.has(k)) continue;
      seen.add(k); tokens.push(t);
      if (tokens.length >= MAX_TOKENS) break;
    }
    if (tokens.length >= MAX_TOKENS) break;
  }
  const heads: string[] = [];
  for (const t of tokens) {
    const m = await db.lookupInText(t, 0, 3).catch(() => null);
    const r = m?.rows.find((x) => x.kind === 'word' && x.pos && ['noun', 'verb', 'adjective', 'adverb'].includes(x.pos));
    if (r && !heads.includes(r.headword)) heads.push(r.headword);
  }
  const words = await db.gameWordsFor(heads);
  textCache.set(id, words);
  return words;
}

/** The pool a round draws from (due review words of the source first, see `pickRound`). */
export async function loadPool(s: Source): Promise<GameWord[]> {
  const now = Date.now();
  if (s.kind === 'level') {
    const [fresh, deck] = await Promise.all([db.gameWords(s.level, 80), loadDeck()]);
    return [...dueWords(deck, now, (w) => w.level === s.level), ...fresh];
  }
  if (s.kind === 'recent') {
    await loadHistory();
    return db.gameWordsFor(history.get().slice(0, 100).map((h) => h.headword));
  }
  return textWords(s.id);
}
