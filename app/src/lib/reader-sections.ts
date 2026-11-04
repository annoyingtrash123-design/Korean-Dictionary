/** Collapsible sections of a Reader text: each "## " heading paragraph opens a section that runs
 *  to the next heading. Paragraphs before the first heading are always shown. */

/** Fold only texts with at least this many headings (a lone title heading is not a chapter list)… */
export const MIN_SECTIONS = 2;
/** …and this much text: a short poem's "original / modern" version labels are not chapters. */
export const MIN_CHARS = 3000;

export const isHeading = (text: string) => text.startsWith('## ');

/** Indices of the heading paragraphs, or [] when the text is not worth folding. */
export function headingsOf(paras: string[]): number[] {
  const h = paras.flatMap((p, i) => (isHeading(p) ? [i] : []));
  const chars = paras.reduce((n, p) => n + p.length, 0);
  return h.length >= MIN_SECTIONS && chars >= MIN_CHARS ? h : [];
}

/** The heading index whose section contains paragraph `n` (-1: before the first heading). */
export function sectionOf(headings: number[], n: number): number {
  let s = -1;
  for (const h of headings) { if (h <= n) s = h; else break; }
  return s;
}

/** Paragraph `n` is hidden: inside a closed section and not the heading itself. */
export const hidden = (headings: number[], open: Set<number>, n: number) => {
  const s = sectionOf(headings, n);
  return s >= 0 && s !== n && !open.has(s);
};
