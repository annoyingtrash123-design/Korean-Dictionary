// Pure helpers for the Reader: UTF-16 offsets, word runs, selection shrink/extend.
// Offsets are always UTF-16 code units (JS string indexes), matching the engine's `lookupInText`.

export type CharClass = 'hangul' | 'han' | 'other';

export function classOf(cp: number): CharClass {
  if ((cp >= 0xac00 && cp <= 0xd7a3) || (cp >= 0x1100 && cp <= 0x11ff) || (cp >= 0x3130 && cp <= 0x318f) || (cp >= 0xa960 && cp <= 0xa97f) || (cp >= 0xd7b0 && cp <= 0xd7ff) || cp === 0x302e || cp === 0x302f) return 'hangul';
  if ((cp >= 0x4e00 && cp <= 0x9fff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0xf900 && cp <= 0xfaff) || (cp >= 0x20000 && cp <= 0x2ffff)) return 'han';
  return 'other';
}
export const isWordCp = (cp: number) => classOf(cp) !== 'other';

/** Move `off` back to the start of the code point it falls in (never inside a surrogate pair). */
export function snapOffset(text: string, off: number): number {
  const o = Math.max(0, Math.min(text.length, off));
  if (o > 0 && o < text.length) {
    const c = text.charCodeAt(o);
    const p = text.charCodeAt(o - 1);
    if (c >= 0xdc00 && c <= 0xdfff && p >= 0xd800 && p <= 0xdbff) return o - 1;
  }
  return o;
}
/** Offset just after the code point starting at `i`. */
export const nextCp = (text: string, i: number) => Math.min(text.length, i + (text.codePointAt(i)! > 0xffff ? 2 : 1));
/** Offset of the code point that ends at `i`. */
export function prevCp(text: string, i: number): number {
  if (i <= 0) return 0;
  const c = text.charCodeAt(i - 1);
  return c >= 0xdc00 && c <= 0xdfff && i >= 2 ? snapOffset(text, i - 1) : i - 1;
}

export interface Span { start: number; end: number }

/** The maximal run of same-script word characters (Hangul or Han) containing `off`; a position just past a run counts as its last character. */
export function runAt(text: string, off: number): Span | null {
  let i = snapOffset(text, off);
  const at = (k: number) => (k >= 0 && k < text.length ? classOf(text.codePointAt(k)!) : 'other');
  if (at(i) === 'other') {
    if (i === 0) return null;
    const p = prevCp(text, i);
    if (at(p) === 'other') return null;
    i = p;
  }
  const cls = at(i);
  let s = i;
  while (s > 0 && at(prevCp(text, s)) === cls) s = prevCp(text, s);
  let e = nextCp(text, i);
  while (e < text.length && at(e) === cls) e = nextCp(text, e);
  return { start: s, end: e };
}

/** Offset of the first word character at or after `from`, or null. */
export function nextWordStart(text: string, from: number): number | null {
  for (let i = snapOffset(text, from); i < text.length; i = nextCp(text, i)) if (isWordCp(text.codePointAt(i)!)) return i;
  return null;
}
/** Offset of the last word character before `before`, or null. */
export function prevWordChar(text: string, before: number): number | null {
  for (let i = snapOffset(text, before); i > 0;) { i = prevCp(text, i); if (isWordCp(text.codePointAt(i)!)) return i; }
  return null;
}

/** Drop the last character of the span (never below one character). */
export function shrinkSpan(text: string, s: Span): Span {
  const e = prevCp(text, s.end);
  return e > s.start ? { start: s.start, end: e } : s;
}
/** Add the next character after the span (stops at the end of the text). */
export function extendSpan(text: string, s: Span): Span {
  return s.end >= text.length ? s : { start: s.start, end: nextCp(text, s.end) };
}

/** [before, highlighted, after] for rendering one highlighted span; clamps and snaps to code points. */
export function splitHighlight(text: string, s: Span | null): [string, string, string] {
  if (!s) return [text, '', ''];
  const a = snapOffset(text, s.start), b = snapOffset(text, Math.max(s.end, a));
  return [text.slice(0, a), text.slice(a, b), text.slice(b)];
}

/** UTF-16 offset of DOM position (node, off) within `root`'s text content. */
export function textOffsetIn(root: Node, node: Node, off: number): number {
  const r = root.ownerDocument!.createRange();
  r.selectNodeContents(root);
  r.setEnd(node, off);
  return r.toString().length;
}

/** DOM Range covering UTF-16 offsets [start, end) of `root`'s text (null when out of range). */
export function rangeAt(root: Node, start: number, end: number): Range | null {
  const doc = root.ownerDocument!;
  const w = doc.createTreeWalker(root, 4 /* SHOW_TEXT */);
  const r = doc.createRange();
  let pos = 0, gotStart = false;
  for (let n = w.nextNode() as Text | null; n; n = w.nextNode() as Text | null) {
    const len = n.data.length;
    if (!gotStart && start <= pos + len && (start < pos + len || end <= start)) { r.setStart(n, start - pos); gotStart = true; }
    if (gotStart && end <= pos + len) { r.setEnd(n, end - pos); return r; }
    pos += len;
  }
  return null;
}
