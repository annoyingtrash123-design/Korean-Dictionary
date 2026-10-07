import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { db, packStatus$ } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { useStore } from '../lib/store';
import { useWide } from '../lib/layout';
import { readerPrefs, updatePrefs, getPos, setPos, type ReaderPrefs } from '../lib/reader-prefs';
import { classOf, extendSpan, nextWordStart, prevWordChar, rangeAt, runAt, shrinkSpan, snapOffset, splitHighlight, textOffsetIn, type Span } from '../lib/reader-text';
import { parseNotes, type Inline, type MdBlock } from '../lib/reader-markdown';
import { fmtChars, fmtYear, shelfLabel } from '../lib/reader-library';
import { groupResults } from '../lib/merge';
import { LookupCard, type LookupState } from '../components/LookupCard';
import { Icon } from '../components/Icons';
import { Empty, LevelBadge, Sheet } from '../components/common';
import { href, readerPath, wordPath } from '../lib/router';
import type { ResultRow, TextDoc, TextLabels, TextMatch, TextProvenance } from '../lib/types';

// ---------- safe markdown ----------
function Inl({ c }: { c: Inline[] }) {
  return <>{c.map((n, i) => n.k === 'text' ? n.s : n.k === 'b' ? <strong key={i}><Inl c={n.c} /></strong> : n.k === 'i' ? <em key={i}><Inl c={n.c} /></em>
    : n.k === 'code' ? <code key={i}><Inl c={n.c} /></code> : <a key={i} href={n.href} target="_blank" rel="noopener noreferrer"><Inl c={n.c} /></a>)}</>;
}
export function Notes({ blocks }: { blocks: MdBlock[] }) {
  return (
    <div class="md">
      {blocks.map((b, i) => b.t === 'h' ? <h4 key={i}><Inl c={b.c} /></h4>
        : b.t === 'p' ? <p key={i}><Inl c={b.c} /></p>
        : b.t === 'ul' ? <ul key={i}>{b.items.map((it, j) => <li key={j}><Inl c={it} /></li>)}</ul>
        : <ol key={i}>{b.items.map((it, j) => <li key={j}><Inl c={it} /></li>)}</ol>)}
    </div>
  );
}

// ---------- labels ----------
export function translationLabel(l: TextLabels, prov?: TextProvenance | null): string {
  const t = l.translation ?? 'ai';
  if (t === 'ai') return 'AI-generated translation — may contain errors';
  const ref = t.replace(/^pd:/, '');
  return `Translation: ${prov?.english_source || ref.replace(/[-_]/g, ' ')} (public domain)`;
}
export function modernLabel(l: TextLabels): string | null {
  return l.modern === 'ai' ? 'Modernised spelling: AI-written — may contain errors' : l.modern === 'wikisource' ? 'Modernised spelling: Wikisource modern edition' : null;
}

const hasRev = (p?: TextProvenance | null): p is TextProvenance => !!p && p.revision_id != null && String(p.revision_id) !== '';
const fmtDate = (s?: string) => (s ? s.slice(0, 10) : '');

// ---------- paragraph ----------
function Para({ n, text, hl, verse, hanmun, reading, en, showEn, showReading }: {
  n: number; text: string; hl: Span | null; verse: boolean; hanmun: boolean; reading?: string | null; en?: string | null; showEn: boolean; showReading: boolean;
}) {
  const [a, b, c] = splitHighlight(text, hl);
  return (
    <section class={`rd-para${verse ? ' verse' : ''}${hanmun ? ' hanmun' : ''}`} data-n={n}>
      <p class="rd-ko" lang="ko" data-ko={n}>{a}{b && <mark class="hl">{b}</mark>}{c}</p>
      {showReading && reading && <p class="rd-reading" lang="ko">{reading}</p>}
      {showEn && en && <p class="rd-en" lang="en">{en}</p>}
    </section>
  );
}

// ---------- reading settings ----------
function ReadingSettings({ p, hasModern, hasReading, label, onClose }: { p: ReaderPrefs; hasModern: boolean; hasReading: boolean; label: string; onClose: () => void }) {
  const sw = (label: string, key: 'en' | 'modern' | 'reading', off?: boolean) => (
    <label class="field-row"><span>{label}</span>
      <input type="checkbox" role="switch" disabled={off} checked={p[key]} onChange={(e) => updatePrefs({ [key]: (e.currentTarget as HTMLInputElement).checked })} /></label>
  );
  return (
    <Sheet title="Reading settings" onClose={onClose}>
      <div class="rd-set">
        <label class="field-row"><span>Text size <span class="muted">{p.size}px</span></span>
          <input type="range" min={14} max={32} step={1} value={p.size} aria-label="Text size" onInput={(e) => updatePrefs({ size: +(e.currentTarget as HTMLInputElement).value })} /></label>
        <label class="field-row"><span>Line spacing <span class="muted">{p.lh.toFixed(2)}</span></span>
          <input type="range" min={1.4} max={2.6} step={0.05} value={p.lh} aria-label="Line spacing" onInput={(e) => updatePrefs({ lh: +(e.currentTarget as HTMLInputElement).value })} /></label>
        <div class="field-row"><span>Korean typeface</span>
          <div class="seg inline" role="radiogroup" aria-label="Korean typeface">
            <button type="button" role="radio" aria-checked={p.serif} class={p.serif ? 'on' : ''} onClick={() => updatePrefs({ serif: true })}>Serif</button>
            <button type="button" role="radio" aria-checked={!p.serif} class={!p.serif ? 'on' : ''} onClick={() => updatePrefs({ serif: false })}>Sans</button>
          </div></div>
        {sw('Show English translation', 'en')}
        {p.en && <div class="rd-labels small">{label}</div>}
        {sw(hasModern ? 'Modern spelling' : 'Modern spelling (not available for this text)', 'modern', !hasModern)}
        {sw(hasReading ? 'Hanmun reading line (음)' : 'Hanmun reading line (none for this text)', 'reading', !hasReading)}
        <div class="rd-sample" style={{ fontSize: p.size, lineHeight: p.lh }} data-serif={p.serif ? '1' : '0'} lang="ko">저는 한국어를 읽어요. <span class="hanja">學校</span></div>
      </div>
    </Sheet>
  );
}

// ---------- caret hit-testing ----------
function caretAt(x: number, y: number): { node: Node; off: number } | null {
  const d = document as Document & { caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null; caretRangeFromPoint?: (x: number, y: number) => Range | null };
  if (d.caretPositionFromPoint) { const p = d.caretPositionFromPoint(x, y); if (p) return { node: p.offsetNode, off: p.offset }; }
  if (d.caretRangeFromPoint) { const r = d.caretRangeFromPoint(x, y); if (r) return { node: r.startContainer, off: r.startOffset }; }
  return null;
}
const rectDist = (r: DOMRect, x: number, y: number) => Math.hypot(Math.max(r.left - x, 0, x - r.right), Math.max(r.top - y, 0, y - r.bottom));
/** UTF-16 offset of the character under (x, y) inside paragraph element `root`, or null when the tap is not near any text. */
export function offsetAtPoint(root: HTMLElement, x: number, y: number): number | null {
  const c = caretAt(x, y);
  if (!c || !root.contains(c.node)) return null;
  const text = root.textContent ?? '';
  const o = textOffsetIn(root, c.node, c.off);
  let best: { i: number; d: number } | null = null;
  for (const raw of [o, o - 1]) {
    if (raw < 0 || raw >= text.length) continue;
    const i = snapOffset(text, raw);
    const end = i + (text.codePointAt(i)! > 0xffff ? 2 : 1);
    const r = rangeAt(root, i, end)?.getBoundingClientRect();
    if (!r) continue;
    const d = rectDist(r, x, y);
    if (!best || d < best.d) best = { i, d };
  }
  return best && best.d <= 22 ? best.i : null;
}

// ---------- the view ----------
export function TextView({ id }: { id: string }) {
  const prefs = useStore(readerPrefs);
  const status = useStore(packStatus$);
  const wide = useWide();
  const r = useAsync(() => db.getText(id), [id]);
  const doc = r.data ?? undefined;
  const scroller = useRef<HTMLDivElement>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [sel, setSel] = useState<{ n: number; span: Span } | null>(null);
  const [lk, setLk] = useState<LookupState>({ match: null, loading: false, manual: false, text: '' });
  const [cardH, setCardH] = useState(0);
  const seq = useRef(0);

  const hasModern = !!doc?.paragraphs.some((p) => p.modern);
  const hasReading = !!doc?.paragraphs.some((p) => p.reading);
  const useModern = prefs.modern && hasModern;
  const shown = (n: number) => { const p = doc!.paragraphs[n]; return (useModern && p.modern) || p.orig; };

  // lookups run against the text as displayed (modern spelling when that toggle is on)
  const clear = () => { seq.current++; setSel(null); setLk({ match: null, loading: false, manual: false, text: '' }); };
  useEffect(() => { clear(); }, [id, useModern]);
  useEffect(() => {
    if (!sel) return;
    const f = (e: KeyboardEvent) => { if (e.key === 'Escape') clear(); };
    window.addEventListener('keydown', f);
    return () => window.removeEventListener('keydown', f);
  }, [!!sel]);

  /** Look up the word at `offset` of paragraph `n`. */
  async function selectAt(n: number, offset: number, clampEndTo?: number) {
    const text = shown(n);
    const run = runAt(text, offset);
    if (!run) { clear(); return; }
    const my = ++seq.current;
    setSel({ n, span: run });
    setLk({ match: null, loading: true, manual: false, text });
    let m: TextMatch;
    try { m = await db.lookupInText(text, offset); } catch { m = { match: text.slice(run.start, run.end), start: run.start, end: run.end, rows: [] }; }
    if (my !== seq.current) return;
    let span: Span = m.rows.length || m.hanja ? { start: m.start, end: m.end } : classOf(text.codePointAt(snapOffset(text, offset))!) === 'han' ? { start: snapOffset(text, offset), end: snapOffset(text, offset) + (text.codePointAt(snapOffset(text, offset))! > 0xffff ? 2 : 1) } : run;
    if (clampEndTo != null && span.end > clampEndTo && span.start < clampEndTo) { span = { start: span.start, end: clampEndTo }; }
    setSel({ n, span });
    setLk({ match: { ...m, match: text.slice(span.start, span.end), start: span.start, end: span.end }, loading: false, manual: false, text });
  }
  /** Look up exactly `span` (manual selection: ⇤ ⇥ and stepping into the middle of a word). */
  async function selectSpan(n: number, span: Span) {
    const text = shown(n);
    const sub = text.slice(span.start, span.end);
    const my = ++seq.current;
    setSel({ n, span });
    setLk((l) => ({ ...l, loading: true, manual: true, text }));
    let m: TextMatch | null = null;
    try { m = await db.lookupInText(sub, 0); } catch { /* fall through to plain search */ }
    if (!m || (!m.rows.length && !m.hanja)) {
      try { const s = await db.search(sub, { limit: 12 }); if (s.rows.length) m = { match: sub, start: 0, end: sub.length, rows: s.rows, hanja: s.hanja?.[0] }; } catch { /* none */ }
    }
    if (my !== seq.current) return;
    setLk({ match: { ...(m ?? { rows: [] }), match: sub, start: span.start, end: span.end, rows: m?.rows ?? [] }, loading: false, manual: true, text });
  }

  async function step(what: 'prev' | 'next' | 'shrink' | 'extend') {
    if (!sel || !doc) return;
    const { n, span } = sel;
    const text = shown(n);
    if (what === 'shrink') return void selectSpan(n, shrinkSpan(text, span));
    if (what === 'extend') return void selectSpan(n, extendSpan(text, span));
    if (what === 'next') {
      let para = n, o = nextWordStart(text, span.end);
      while (o == null && para + 1 < doc.paragraphs.length) { para++; o = nextWordStart(shown(para), 0); }
      if (o == null) return;
      const t = shown(para), run = runAt(t, o);
      if (run && run.start < o) return void selectSpan(para, { start: o, end: run.end });  // rest of the eojeol after a partial match
      return void selectAt(para, o);
    }
    let para = n, o = prevWordChar(text, span.start);
    while (o == null && para > 0) { para--; o = prevWordChar(shown(para), shown(para).length); }
    if (o == null) return;
    void selectAt(para, o, para === n ? span.start : undefined);
  }
  const pick = (i: number) => setLk((l) => {
    if (!l.match) return l;
    const g = groupResults(l.match.rows);
    if (!g[i]) return l;
    const first = new Set(g[i].rows);
    return { ...l, match: { ...l.match, rows: [...g[i].rows, ...l.match.rows.filter((x: ResultRow) => !first.has(x))] } };
  });

  // tap on the text
  const onClick = (e: MouseEvent) => {
    if (!doc) return;
    const t = e.target as HTMLElement;
    const ko = t.closest<HTMLElement>('[data-ko]') ?? (t.closest('.rd-para')?.querySelector<HTMLElement>('[data-ko]') ?? null);
    const s = window.getSelection();
    if (s && !s.isCollapsed && s.toString().trim()) return;   // the user is selecting text, not tapping
    if (!ko) { if (sel) clear(); return; }
    const off = offsetAtPoint(ko, e.clientX, e.clientY);
    if (off == null) { if (sel) clear(); return; }
    void selectAt(+ko.dataset.ko!, off);
  };

  // keep the highlighted word clear of the card
  useLayoutEffect(() => {
    const sc = scroller.current; if (!sc || !sel) return;
    const mark = sc.querySelector<HTMLElement>('mark.hl'); if (!mark) return;
    const sr = sc.getBoundingClientRect(), mr = mark.getBoundingClientRect();
    const limit = sr.bottom - cardH - 28;
    if (mr.bottom > limit) sc.scrollBy({ top: mr.bottom - limit });
    else if (mr.top < sr.top + 8) sc.scrollBy({ top: mr.top - sr.top - 80 });
  }, [sel?.n, sel?.span.start, sel?.span.end, cardH]);

  // reading position: remember (paragraph, fraction) and resume
  const anchor = useRef<{ n: number; f: number } | null>(null);
  const saveT = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const measure = () => {
    const sc = scroller.current; if (!sc || !doc) return;
    const top = sc.getBoundingClientRect().top;
    const els = sc.querySelectorAll<HTMLElement>('.rd-para');
    for (const el of els) {
      const b = el.getBoundingClientRect();
      if (b.bottom > top + 4) { const n = +el.dataset.n!; anchor.current = { n, f: Math.max(n === 0 ? -200 : 0, Math.min(1, (top - b.top) / Math.max(1, b.height))) }; return; }
    }
  };
  const flush = () => { if (anchor.current && doc) setPos(doc.id, anchor.current); };
  const onScroll = () => { measure(); clearTimeout(saveT.current); saveT.current = setTimeout(flush, 250); };
  const restore = (pos: { n: number; f: number } | null) => {
    const sc = scroller.current; if (!sc || !pos) return;
    const el = sc.querySelector<HTMLElement>(`.rd-para[data-n="${pos.n}"]`); if (!el) return;
    const b = el.getBoundingClientRect(), top = sc.getBoundingClientRect().top;
    sc.scrollTop += b.top - top + pos.f * b.height;
  };
  const restored = useRef<string | undefined>(undefined);
  useLayoutEffect(() => {
    if (!doc || restored.current === doc.id) return;
    restored.current = doc.id;
    anchor.current = null;
    const pos = getPos(doc.id);
    if (pos) { restore(pos); requestAnimationFrame(() => restore(pos)); anchor.current = pos; } else if (scroller.current) scroller.current.scrollTop = 0;
  }, [doc?.id]);
  // typography changes reflow the text: keep the same place
  const first = useRef(true);
  useLayoutEffect(() => { if (first.current) { first.current = false; return; } restore(anchor.current); }, [prefs.size, prefs.lh, prefs.serif, prefs.en, prefs.reading, useModern]);
  useEffect(() => {
    const onHide = () => flush();
    window.addEventListener('pagehide', onHide); document.addEventListener('visibilitychange', onHide);
    return () => { window.removeEventListener('pagehide', onHide); document.removeEventListener('visibilitychange', onHide); clearTimeout(saveT.current); flush(); };
  }, [doc?.id]);

  const [openQ, setOpenQ] = useState<Set<number>>(new Set());
  useEffect(() => setOpenQ(new Set()), [id]);

  if (r.loading && !doc) return <div class="rd-host"><div class="skeleton" /></div>;
  if (!doc) return <div class="rd-host"><Empty title="Text not found">{r.error ?? 'This text is not in the installed library.'} <a href={href(readerPath())}>Back to the library</a></Empty></div>;

  const m = doc.meta;
  const prov = doc.provenance;
  const verse = ['verse', 'modern-poetry'].includes(m.shelf ?? '') || doc.paragraphs.some((p) => p.orig.includes('\n'));
  const hanmun = m.script === 'hanmun';
  const hasEn = doc.paragraphs.some((p) => p.en);
  const total = doc.paragraphs.reduce((s, p) => s + p.orig.length, 0);
  const notesKo = doc.notes?.ko ? parseNotes(doc.notes.ko) : [];
  const notesEn = doc.notes?.en ? parseNotes(doc.notes.en) : [];
  const style = { '--rd-size': `${prefs.size}px`, '--rd-lh': String(prefs.lh) } as Record<string, string>;
  const cedict = !!status?.packs.cedict?.installed;

  return (
    <div class="rd-host" data-serif={prefs.serif ? '1' : '0'} style={style}>
      <div class="rd-bar">
        {!wide && <a class="iconbtn" href={href(readerPath())} aria-label="Back to library"><Icon name="back" /></a>}
        <div class="rd-bar-title" lang="ko">{m.title_ko}</div>
        <button type="button" class="iconbtn" aria-label="Reading settings" onClick={() => setSettingsOpen(true)}><Icon name="aa" /></button>
      </div>
      <div class="rd-scroll" ref={scroller} onScroll={onScroll} onClick={onClick}>
        <article class="rd-article">
          <header class="rd-card">
            <div class="eyebrow">{shelfLabel(m.shelf ?? '')}{m.excerpt ? ' · excerpt' : ''}</div>
            <h1 class="rd-title" lang="ko">{m.title_ko}</h1>
            {m.title_en && <div class="rd-title-en">{m.title_en}</div>}
            <div class="rd-by">
              {(m.author_ko || m.author_en) && <span><span lang="ko">{m.author_ko}</span>{m.author_en && m.author_ko !== m.author_en ? <span class="muted"> · {m.author_en}</span> : null}{m.author_dates ? <span class="muted"> ({m.author_dates})</span> : null}</span>}
              {(m.date || m.year != null) && <span class="muted">{m.date || fmtYear(m.year)}</span>}
            </div>
            <div class="rd-badges">
              {doc.card.level && <span class="chip-lvl">{doc.card.level}</span>}
              <span class="chip-quiet">{fmtChars(total)}</span>
              {doc.labels.text === 'ai' && <span class="chip-ai">AI-written</span>}
              {m.excerpt && <span class="chip-quiet">excerpt</span>}
            </div>
            <details class="rd-about">
              <summary><span>About this text</span><span class="rd-i" aria-hidden="true">i</span></summary>
              <div class="rd-about-body">
                {(doc.card.summary_en || doc.card.summary_ko) && <div class="rd-summary"><p lang="ko">{doc.card.summary_ko}</p><p class="muted">{doc.card.summary_en}</p></div>}
                <dl class="rd-prov">
                  <dt>Shelf</dt><dd>{shelfLabel(m.shelf ?? '')}{m.script ? ` · ${m.script === 'hanmun' ? 'hanmun' : m.script === 'mixed' ? 'hangul + hanja' : 'hangul'}` : ''}</dd>
                  {(doc.card.edition_en || doc.card.edition_ko || prov?.edition) && <><dt>Edition</dt><dd>{doc.card.edition_en || prov?.edition}{doc.card.edition_ko && doc.card.edition_en ? <span class="muted" lang="ko"> · {doc.card.edition_ko}</span> : null}</dd></>}
                  {prov?.url && <><dt>Source</dt><dd><a href={/^https?:\/\//.test(prov.url) ? prov.url : undefined} target="_blank" rel="noopener noreferrer">{prov.page_title || prov.source || prov.url}</a>{hasRev(prov) && <span class="muted"> · revision {String(prov.revision_id)}{prov.revision_timestamp ? ` (${fmtDate(prov.revision_timestamp)})` : ''}</span>}</dd></>}
                  {!prov?.url && hasRev(prov) && <><dt>Revision</dt><dd>revision {String(prov.revision_id)}{prov.revision_timestamp ? ` (${fmtDate(prov.revision_timestamp)})` : ''}</dd></>}
                  {(prov?.licence || m.pd_basis) && <><dt>Licence</dt><dd>{prov?.licence || m.pd_basis}</dd></>}
                  {hasEn && <><dt>English</dt><dd>{translationLabel(doc.labels, prov)}</dd></>}
                  {hasModern && modernLabel(doc.labels) && <><dt>Modern</dt><dd>{modernLabel(doc.labels)}</dd></>}
                </dl>
                {(notesKo.length > 0 || notesEn.length > 0) && (
                  <div class="rd-notes">
                    <h3>Notes <span lang="ko">· 해설</span>{doc.labels.notes === 'ai' && <span class="chip-ai">AI-written</span>}</h3>
                    {notesKo.length > 0 && <div lang="ko" class="rd-notes-ko"><Notes blocks={notesKo} /></div>}
                    {notesEn.length > 0 && <div lang="en"><Notes blocks={notesEn} /></div>}
                  </div>
                )}
              </div>
            </details>
          </header>

          <div class="rd-ctl" role="group" aria-label="Display">
            {hasModern && (
              <div class="seg inline" role="radiogroup" aria-label="Spelling">
                <button type="button" role="radio" aria-checked={!useModern} class={!useModern ? 'on' : ''} onClick={() => updatePrefs({ modern: false })}>Original</button>
                <button type="button" role="radio" aria-checked={useModern} class={useModern ? 'on' : ''} onClick={() => updatePrefs({ modern: true })}>Modern</button>
              </div>
            )}
            {hasReading && <button type="button" class={`pill${prefs.reading ? ' on' : ''}`} aria-pressed={prefs.reading} onClick={() => updatePrefs({ reading: !prefs.reading })}>음 reading</button>}
          </div>
          {(hasEn && prefs.en) || useModern ? (
            <div class="rd-labels small">
              {hasEn && prefs.en && <div class="rd-trl">{(doc.labels.translation ?? 'ai') === 'ai' ? <><span class="chip-ai">AI translation</span> may contain errors</> : translationLabel(doc.labels, prov)}</div>}
              {useModern && modernLabel(doc.labels) && <div>{modernLabel(doc.labels)}</div>}
            </div>
          ) : null}
          {(hanmun || m.script === 'mixed') && !cedict && <div class="rd-hint small">Tip: install CC-CEDICT in Settings → Dictionaries for fuller hanja lookups.</div>}

          <div class="rd-body" lang="ko">
            {doc.paragraphs.map((p, i) => (
              <Para key={p.n} n={i} text={shown(i)} hl={sel && sel.n === i ? sel.span : null} verse={verse} hanmun={hanmun}
                reading={p.reading} en={p.en} showEn={prefs.en} showReading={prefs.reading} />
            ))}
          </div>

          {!!doc.vocab?.length && (
            <section class="rd-extra" aria-label="Vocabulary">
              <h2>Vocabulary <span lang="ko">· 어휘</span></h2>
              <ul class="plain rd-vocab">
                {doc.vocab.map((v, i) => (
                  <li key={i}><a class="hangul" lang="ko" href={href(wordPath(v.word))}>{v.word}</a><span class="rd-vgloss">{v.gloss_en}</span><LevelBadge level={v.level} /></li>
                ))}
              </ul>
            </section>
          )}
          {!!doc.questions?.length && (
            <section class="rd-extra" aria-label="Comprehension questions">
              <h2>Questions <span lang="ko">· 이해 확인</span></h2>
              <ol class="plain rd-q">
                {doc.questions.map((q, i) => {
                  const open = openQ.has(i);
                  return (
                    <li key={i}>
                      <div class="rd-q-ko" lang="ko">{q.q_ko}</div>
                      {q.q_en && <div class="rd-q-en">{q.q_en}</div>}
                      {q.answer_en && (open
                        ? <div class="rd-ans" role="status">{q.answer_en}</div>
                        : <button type="button" class="link" onClick={() => setOpenQ(new Set([...openQ, i]))}>Show answer</button>)}
                    </li>
                  );
                })}
              </ol>
            </section>
          )}
          <div class="rd-end" aria-hidden="true" />
        </article>
      </div>
      {sel && <LookupCard st={lk} onClose={clear} onStep={step} onResize={setCardH} onPick={pick} />}
      {settingsOpen && <ReadingSettings p={prefs} hasModern={hasModern} hasReading={hasReading} label={translationLabel(doc.labels, prov)} onClose={() => setSettingsOpen(false)} />}
    </div>
  );
}
