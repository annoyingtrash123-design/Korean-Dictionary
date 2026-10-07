// TEMPORARY stand-in for the Rust engine's deconjugator (used only by the sqlite-wasm test engine).
// Handles regular 해요체/past/formal endings well enough for fixture testing.
export interface Cand { lemma: string; rule: string }
const SYL = (c: string) => c.charCodeAt(0) - 0xac00;
const jong = (c: string) => SYL(c) % 28;
const withJong = (c: string, j: number) => String.fromCharCode(c.charCodeAt(0) - jong(c) + j);
const strip = (c: string) => withJong(c, 0);

export function tempDeconj(input: string): Cand[] {
  const out: Cand[] = [];
  const s = input.replace(/\s/g, '');
  const add = (stem: string, rule: string) => stem && out.push({ lemma: stem + '다', rule });
  const tails: [RegExp, string][] = [
    [/(.*)(어요|아요|해요)$/, 'present polite (-아요/어요)'],
    [/(.*)(었어요|았어요|했어요)$/, 'past tense (polite)'],
    [/(.*)(습니다|ㅂ니다)$/, 'formal polite'],
  ];
  for (const [re, rule] of tails) {
    const m = s.match(re); if (m) add(m[1], rule);
  }
  // contracted past: 갔어요 -> 가 + 았어요 (final ㅆ on the stem syllable)
  const past = s.match(/^(.*)(.)(어요|았어요|었어요)$/);
  if (past && jong(past[2]) === 20) { add(past[1] + strip(past[2]), 'past tense (polite)'); add(past[1] + withJong(past[2], 0), 'past tense (polite)'); }
  const past2 = s.match(/^(.*)(.)(다|어|아|서)$/);
  if (past2 && jong(past2[2]) === 20) add(past2[1] + strip(past2[2]), 'past tense');
  const pol = s.match(/^(.*)(.)요$/);
  if (pol) { add(pol[1] + pol[2], 'polite -요'); add(pol[1] + strip(pol[2]), 'present polite (-아요/어요)'); }
  const fin = s.match(/^(.*)(.)$/);
  if (fin && jong(fin[2]) === 4) add(fin[1] + strip(fin[2]), 'present adnominal / -ㄴ');
  return out;
}
export const tempHints = (input: string): string[] =>
  /서$/.test(input) ? ['-아서/어서: because; and then'] : /고$/.test(input) ? ['-고: and'] : [];
