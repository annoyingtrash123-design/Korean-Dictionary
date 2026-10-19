// Building questions from quiz words: plausible wrong answers, gap sentences, 초성 prompts.
import type { GameWord } from '../types';
import { choseong, shortGloss, syllables } from './hangul';

export type Dir = 'ko-en' | 'en-ko';

export function shuffle<T>(xs: T[], rnd = Math.random): T[] {
  const a = [...xs];
  for (let i = a.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); [a[i], a[j]] = [a[j], a[i]]; }
  return a;
}

/** `n` other words to offer as wrong answers: same part of speech first, never the same headword
 *  or the same (short) meaning. */
export function distractors(word: GameWord, pool: GameWord[], n = 3, rnd = Math.random): GameWord[] {
  const hw = syllables(word.headword), g = shortGloss(word.gloss).toLowerCase();
  const ok = pool.filter((o) => syllables(o.headword) !== hw && shortGloss(o.gloss).toLowerCase() !== g);
  const same = shuffle(ok.filter((o) => o.pos === word.pos), rnd);
  const other = shuffle(ok.filter((o) => o.pos !== word.pos), rnd);
  const out: GameWord[] = [];
  const seenG = new Set([g]), seenH = new Set([hw]);
  for (const o of [...same, ...other]) {
    const og = shortGloss(o.gloss).toLowerCase(), oh = syllables(o.headword);
    if (seenG.has(og) || seenH.has(oh)) continue;
    seenG.add(og); seenH.add(oh); out.push(o);
    if (out.length === n) break;
  }
  return out;
}

export interface Choice { label: string; correct: boolean }
export interface MeaningQ { word: GameWord; dir: Dir; prompt: string; choices: Choice[] }

export function meaningQuestion(word: GameWord, pool: GameWord[], dir: Dir, rnd = Math.random): MeaningQ {
  const opts = [word, ...distractors(word, pool, 3, rnd)];
  const label = (w: GameWord) => (dir === 'ko-en' ? shortGloss(w.gloss) : w.headword.replace(/\d+$/, ''));
  return {
    word, dir,
    prompt: dir === 'ko-en' ? word.headword.replace(/\d+$/, '') : word.gloss,
    choices: shuffle(opts.map((w) => ({ label: label(w), correct: w === word })), rnd),
  };
}

export interface GapQ { word: GameWord; before: string; after: string; sentence: string; choices: Choice[] }

/** A gap sentence from the word's own examples: nouns keep their particle (학교에 -> ___에),
 *  verbs and adjectives blank the whole inflected word; choices are dictionary forms. */
export function gapQuestion(word: GameWord, pool: GameWord[], rnd = Math.random): GapQ | null {
  const hw = syllables(word.headword);
  if (!hw) return null;
  const predicate = /다$/.test(hw) && (word.pos === 'verb' || word.pos === 'adjective');
  const stem = predicate ? hw.slice(0, -1) : hw;
  if (!stem) return null;
  for (const s of shuffle(word.examples, rnd)) {
    const parts = s.split(/(\s+)/);
    for (let i = 0; i < parts.length; i++) {
      const tok = parts[i];
      const core = tok.replace(/^[^가-힣]+/, '');
      if (!core.startsWith(stem)) continue;
      if (predicate && syllables(core).length > stem.length + 4) continue;   // 가 in 가방: not our verb
      const lead = tok.slice(0, tok.length - core.length);
      const tail = predicate ? core.slice(syllables(core.match(/^[가-힣]+/)?.[0] ?? '').length) : core.slice(stem.length);
      const before = parts.slice(0, i).join('') + lead;
      const after = tail + parts.slice(i + 1).join('');
      const opts = [word, ...distractors(word, pool, 3, rnd)];
      if (opts.length < 3) return null;
      return { word, before, after, sentence: s, choices: shuffle(opts.map((w) => ({ label: w.headword.replace(/\d+$/, ''), correct: w === word })), rnd) };
    }
  }
  return null;
}

export interface ChoQ { word: GameWord; cho: string }
export const choQuestion = (word: GameWord): ChoQ | null => {
  const hw = syllables(word.headword);
  return hw.length >= 2 ? { word, cho: choseong(hw) } : null;
};
