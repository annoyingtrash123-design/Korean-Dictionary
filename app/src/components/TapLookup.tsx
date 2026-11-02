import { useEffect, useLayoutEffect, useRef, useState } from 'preact/hooks';
import { db } from '../db/client';
import { groupResults } from '../lib/merge';
import { classOf, extendSpan, nextWordStart, offsetAtPoint, prevWordChar, rangeAt, runAt, shrinkSpan, snapOffset, type Span } from '../lib/reader-text';
import type { ResultRow, TextMatch } from '../lib/types';
import { LookupCard, type LookupState } from './LookupCard';

/** Korean text anywhere in the app marked `data-tap` (example sentences, Korean definitions)
 *  works like the Reader: tap a word for its dictionary card. Mounted once, next to the panes. */
export const TAP = { 'data-tap': '' };

const HL = 'tap-word';
type Sel = { el: HTMLElement; span: Span };
const NONE: LookupState = { match: null, loading: false, manual: false, text: '' };
const highlights = (): { set: (k: string, v: unknown) => void; delete: (k: string) => void } | undefined =>
  (globalThis as unknown as { CSS?: { highlights?: { set: (k: string, v: unknown) => void; delete: (k: string) => void } } }).CSS?.highlights;
const tapEls = () => [...document.querySelectorAll<HTMLElement>('[data-tap]')];

export function TapLookup() {
  const [sel, setSel] = useState<Sel | null>(null);
  const [lk, setLk] = useState<LookupState>(NONE);
  const [box, setBox] = useState<DOMRect | null>(null);
  const [cardH, setCardH] = useState(0);
  const seq = useRef(0);
  const selRef = useRef<Sel | null>(null);
  selRef.current = sel;

  const clear = () => { seq.current++; setSel(null); setLk(NONE); };

  async function selectAt(el: HTMLElement, offset: number, clampEndTo?: number) {
    const text = el.textContent ?? '';
    const run = runAt(text, offset);
    if (!run) { clear(); return; }
    const my = ++seq.current;
    setSel({ el, span: run });
    setLk({ match: null, loading: true, manual: false, text });
    let m: TextMatch;
    try { m = await db.lookupInText(text, offset); } catch { m = { match: text.slice(run.start, run.end), start: run.start, end: run.end, rows: [] }; }
    if (my !== seq.current) return;
    const o = snapOffset(text, offset);
    let span: Span = m.rows.length || m.hanja ? { start: m.start, end: m.end }
      : classOf(text.codePointAt(o)!) === 'han' ? { start: o, end: o + (text.codePointAt(o)! > 0xffff ? 2 : 1) } : run;
    if (clampEndTo != null && span.end > clampEndTo && span.start < clampEndTo) span = { start: span.start, end: clampEndTo };
    setSel({ el, span });
    setLk({ match: { ...m, match: text.slice(span.start, span.end), start: span.start, end: span.end }, loading: false, manual: false, text });
  }

  async function selectSpan(el: HTMLElement, span: Span) {
    const text = el.textContent ?? '';
    const sub = text.slice(span.start, span.end);
    const my = ++seq.current;
    setSel({ el, span });
    setLk((l) => ({ ...l, loading: true, manual: true, text }));
    let m: TextMatch | null = null;
    try { m = await db.lookupInText(sub, 0); } catch { /* fall through to plain search */ }
    if (!m || (!m.rows.length && !m.hanja)) {
      try { const s = await db.search(sub, { limit: 12 }); if (s.rows.length) m = { match: sub, start: 0, end: sub.length, rows: s.rows, hanja: s.hanja?.[0] }; } catch { /* none */ }
    }
    if (my !== seq.current) return;
    setLk({ match: { ...(m ?? { rows: [] }), match: sub, start: span.start, end: span.end, rows: m?.rows ?? [] }, loading: false, manual: true, text });
  }

  /** ◀ ▶ move word by word, on into the neighbouring tappable text; ⇤ ⇥ resize. */
  function step(what: 'prev' | 'next' | 'shrink' | 'extend') {
    if (!sel) return;
    const { el, span } = sel;
    const text = el.textContent ?? '';
    if (what === 'shrink') return void selectSpan(el, shrinkSpan(text, span));
    if (what === 'extend') return void selectSpan(el, extendSpan(text, span));
    const all = tapEls();
    let i = all.indexOf(el);
    if (what === 'next') {
      let cur = el, o = nextWordStart(text, span.end);
      while (o == null && i >= 0 && i < all.length - 1) { cur = all[++i]; o = nextWordStart(cur.textContent ?? '', 0); }
      if (o == null) return;
      const run = runAt(cur.textContent ?? '', o);
      if (run && run.start < o) return void selectSpan(cur, { start: o, end: run.end });
      return void selectAt(cur, o);
    }
    let cur = el, o = prevWordChar(text, span.start);
    while (o == null && i > 0) { cur = all[--i]; const t = cur.textContent ?? ''; o = prevWordChar(t, t.length); }
    if (o == null) return;
    void selectAt(cur, o, cur === el ? span.start : undefined);
  }

  const pick = (i: number) => setLk((l) => {
    if (!l.match) return l;
    const g = groupResults(l.match.rows);
    if (!g[i]) return l;
    const first = new Set(g[i].rows);
    return { ...l, match: { ...l.match, rows: [...g[i].rows, ...l.match.rows.filter((x: ResultRow) => !first.has(x))] } };
  });

  // taps anywhere in the document
  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      const t = e.target as HTMLElement;
      if (t.closest('.tl-host .lk')) return;
      const el = t.closest<HTMLElement>('[data-tap]');
      const s = window.getSelection();
      if (s && !s.isCollapsed && s.toString().trim()) return;   // selecting text, not tapping
      const off = el ? offsetAtPoint(el, e.clientX, e.clientY) : null;
      if (!el || off == null) { if (selRef.current) clear(); return; }
      // the card covers the pane the text scrolls in
      const pane = el.closest<HTMLElement>('main') ?? document.body;
      setBox(pane.getBoundingClientRect());
      void selectAt(el, off);
    };
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape' && selRef.current) clear(); };
    const onNav = () => clear();
    document.addEventListener('click', onClick);
    window.addEventListener('keydown', onKey);
    window.addEventListener('hashchange', onNav);
    return () => { document.removeEventListener('click', onClick); window.removeEventListener('keydown', onKey); window.removeEventListener('hashchange', onNav); };
  }, []);

  // highlight the selected word (CSS Custom Highlight API; no highlight where unsupported)
  useLayoutEffect(() => {
    const h = highlights();
    const Hl = (globalThis as unknown as { Highlight?: new (...r: Range[]) => unknown }).Highlight;
    if (!h || !Hl) return;
    const r = sel && sel.el.isConnected ? rangeAt(sel.el, sel.span.start, sel.span.end) : null;
    if (r) h.set(HL, new Hl(r)); else h.delete(HL);
  }, [sel?.el, sel?.span.start, sel?.span.end]);
  useEffect(() => () => highlights()?.delete(HL), []);

  // keep the selected word clear of the card
  useLayoutEffect(() => {
    if (!sel || !sel.el.isConnected) return;
    const sc = sel.el.closest<HTMLElement>('main'); if (!sc) return;
    const r = rangeAt(sel.el, sel.span.start, sel.span.end)?.getBoundingClientRect(); if (!r) return;
    const sr = sc.getBoundingClientRect();
    const limit = sr.bottom - cardH - 28;
    if (r.bottom > limit) sc.scrollBy({ top: r.bottom - limit });
    else if (r.top < sr.top + 8) sc.scrollBy({ top: r.top - sr.top - 80 });
  }, [sel?.el, sel?.span.start, sel?.span.end, cardH]);

  if (!sel || !box) return null;
  return (
    <div class="tl-host" style={{ left: `${box.left}px`, top: `${box.top}px`, width: `${box.width}px`, height: `${box.height}px` }}>
      <LookupCard st={lk} onClose={clear} onStep={step} onResize={setCardH} onPick={pick} />
    </div>
  );
}
