import { useEffect, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { Hanja, Pos } from './common';
import { SaveButton } from './SaveButton';
import { entryPath, href, wordPath } from '../lib/router';
import { curriculumHint, curriculumLabel } from '../lib/home-day';
import type { HomeData, HomeIdiom } from '../db/types';
import { EntryInline } from '../views/Entry';
import type { ItemInput } from '../lib/lists';

export function CurriculumLevel({ level }: { level: number }) {
  return <span class="cur-lvl" title={curriculumHint(level)} aria-label={curriculumHint(level)}>{curriculumLabel(level)}</span>;
}

interface PageSpec {
  key: string;
  kind: 'proverb' | 'seongeo' | 'idiom' | 'word' | 'grammar';
  label: ComponentChildren;     // header strip, left: what this page is
  level?: number;               // curriculum level badge (words and grammar)
  save: ItemInput;
  title: string;
  hanja?: string | null;
  pos?: string | null;
  gloss?: string | null;
  entry: { source?: string; id?: number; hw?: string; word?: string };   // the entry shown in the card
  path: string;                 // the entry page (the headword links there)
}

const IDIOM_LABEL: Record<HomeIdiom['kind'], [string, string]> = { proverb: ['Proverb', '속담'], seongeo: ['Four-character idiom', '성어'], idiom: ['Idiom', '관용구'] };

function pageSpecs(h: HomeData): PageSpec[] {
  const out: PageSpec[] = [];
  for (const i of h.idioms ?? []) {
    const [en, ko] = IDIOM_LABEL[i.kind] ?? IDIOM_LABEL.idiom;
    out.push({
      key: `i${i.id}`, kind: i.kind,
      label: <>{en} <span class="card-kind-ko" lang="ko">{ko}</span></>,
      save: { kind: 'entry', source: i.source, headword: i.headword, homonym: i.homonym, pos: i.pos ?? i.kind, gloss: i.gloss },
      title: i.headword, gloss: i.gloss, entry: { source: i.source, id: i.id, hw: i.headword }, path: entryPath(i.source, i.id, i.headword),
    });
  }
  for (const w of h.words) {
    out.push({
      key: `w${w.id}`, kind: 'word', label: 'Word', level: w.level,
      save: { kind: 'entry', source: w.source, headword: w.headword, homonym: w.homonym, pos: w.pos, hanja: w.hanja, gloss: w.gloss },
      title: w.headword, hanja: w.hanja, pos: w.pos, gloss: w.gloss, entry: { source: w.source, id: w.id, hw: w.headword }, path: entryPath(w.source, w.id, w.headword),
    });
  }
  for (const g of h.grammar) {
    out.push({
      key: `g${g.id}`, kind: 'grammar',
      label: 'Grammar', level: g.level,
      save: { kind: 'grammar', pattern: g.pattern, summary: g.summary_en, level: g.level, ref: g.entry ? { source: g.entry.source, homonym: g.entry.homonym, pos: g.entry.pos } : null },
      title: g.pattern, gloss: g.summary_en,
      entry: g.entry ? { source: g.entry.source, id: g.entry.id, hw: g.pattern } : { word: g.pattern },
      path: g.entry ? entryPath(g.entry.source, g.entry.id, g.pattern) : wordPath(g.pattern),
    });
  }
  return out;
}

/** One page: a framed card. The header strip (kind · level · save) stays put; below it the card scrolls
 *  through the headword (hanja and part of speech under it), its English meaning and then the whole
 *  entry as the entry page's Dict tab shows it (every dictionary's senses, examples, related words,
 *  grammar lessons). The entry loads once the page is near the one on screen. */
function Page({ p, n, total, near }: { p: PageSpec; n: number; total: number; near: boolean }) {
  return (
    <div class="car-slide" role="group" aria-roledescription="slide" aria-label={`${n} of ${total}`}>
      <article class="today-card" data-kind={p.kind}>
        <header class="card-head">
          <span class="card-kind">{p.label}</span>
          {p.level != null && <CurriculumLevel level={p.level} />}
          <span class="card-spacer" />
          <SaveButton item={p.save} title={p.title} size={22} />
        </header>
        <div class="card-body" tabIndex={-1}>
          <div class="card-top">
            <h2 class="card-hw">
              <a class="card-link" href={href(p.path)}><span class="hangul" lang="ko">{p.title}</span></a>
            </h2>
            {(p.hanja || p.pos) && <div class="card-sub">{p.hanja && <Hanja text={p.hanja} />}<Pos pos={p.pos} /></div>}
            {p.gloss && <p class="card-gloss">{p.gloss}</p>}
          </div>
          {near && <EntryInline {...p.entry} />}
        </div>
      </article>
    </div>
  );
}

const STEP_KEYS: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };

/** Home: full-page items to swipe through (idiom of the day, 3 words, 2 grammar patterns): scroll-snap track, mouse drag,
 *  arrow buttons and keys, page dots. Always opens on the first page. */
export function HomePager({ data }: { data: HomeData }) {
  const pages = pageSpecs(data);
  const n = pages.length;
  const track = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; left: number; moved: boolean } | null>(null);
  const [cur, setCur] = useState(0);
  // pages whose entry has loaded stay loaded (swiping back does not reload them)
  const [seen] = useState(() => new Set<number>());
  for (let i = cur - 1; i <= cur + 1; i++) seen.add(i);
  if (!n) return null;

  const nearest = () => {
    const t = track.current;
    return t && t.clientWidth ? Math.max(0, Math.min(n - 1, Math.round(t.scrollLeft / t.clientWidth))) : 0;
  };
  const goTo = (i: number) => {
    const t = track.current;
    if (!t) return;
    const smooth = !matchMedia('(prefers-reduced-motion: reduce)').matches;
    t.scrollTo({ left: Math.max(0, Math.min(n - 1, i)) * t.clientWidth, behavior: smooth ? 'smooth' : 'auto' });
  };
  const onScroll = () => setCur(nearest());
  useEffect(() => { track.current?.scrollTo({ left: 0 }); setCur(0); }, [n]);

  const onPointerDown = (e: PointerEvent) => {
    if (e.pointerType !== 'mouse' || e.button !== 0 || !track.current) return;
    if ((e.target as HTMLElement).closest('a, button, .chip')) return;
    drag.current = { x: e.clientX, left: track.current.scrollLeft, moved: false };
  };
  const onPointerMove = (e: PointerEvent) => {
    const d = drag.current, t = track.current;
    if (!d || !t) return;
    const dx = e.clientX - d.x;
    if (!d.moved && Math.abs(dx) > 5) { d.moved = true; t.classList.add('dragging'); t.setPointerCapture(e.pointerId); }
    if (d.moved) t.scrollLeft = d.left - dx;
  };
  const endDrag = () => {
    const d = drag.current, t = track.current;
    drag.current = null;
    if (!d?.moved || !t) return;
    t.classList.remove('dragging');
    goTo(nearest());
  };
  const onKeyDown = (e: KeyboardEvent) => {
    const step = STEP_KEYS[e.key];
    if (!step || (e.target as HTMLElement).closest('button, input, a')) return;
    e.preventDefault();
    goTo(cur + step);
  };

  return (
    <section class="today" role="region" aria-roledescription="carousel" aria-label="Today: idiom, words and grammar">
      <div class="car-track" ref={track} tabIndex={0} aria-label="Pages, swipe or use the arrow keys" onScroll={onScroll} onKeyDown={onKeyDown}
        onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={endDrag} onPointerCancel={endDrag}
        onClickCapture={(e) => { if (drag.current?.moved) { e.preventDefault(); e.stopPropagation(); } }}>
        {pages.map((p, i) => <Page key={p.key} p={p} n={i + 1} total={n} near={Math.abs(i - cur) <= 1 || seen.has(i)} />)}
      </div>
      {n > 1 && (
        <div class="car-nav">
          <button type="button" class="iconbtn car-arrow" aria-label="Previous page" disabled={cur <= 0} onClick={() => goTo(cur - 1)}><Icon name="back" size={22} /></button>
          <div class="car-dots" role="group" aria-label="Choose a page">
            {pages.map((p, i) => (
              <button key={p.key} type="button" class="car-dot" aria-label={`${i + 1} of ${n}`} aria-current={i === cur ? 'true' : undefined} onClick={() => goTo(i)} />
            ))}
          </div>
          <button type="button" class="iconbtn car-arrow" aria-label="Next page" disabled={cur >= n - 1} onClick={() => goTo(cur + 1)}><Icon name="chev" size={22} /></button>
        </div>
      )}
    </section>
  );
}
