import { useEffect, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { Hanja, Pos } from './common';
import { SaveButton } from './SaveButton';
import { entryPath, href, searchPath, wordPath } from '../lib/router';
import { curriculumHint, curriculumLabel } from '../lib/home-day';
import { useWide } from '../lib/layout';
import type { HomeData, HomeExample, HomeRel } from '../db/types';
import { ExampleList } from './Examples';
import type { ItemInput } from '../lib/lists';

export function CurriculumLevel({ level }: { level: number }) {
  return <span class="cur-lvl" title={curriculumHint(level)} aria-label={curriculumHint(level)}>{curriculumLabel(level)}</span>;
}

const RELTYPE: Record<string, string> = { synonym: 'syn', antonym: 'ant', honorific: 'hon', humble: 'hum', 'see also': 'see', reference: 'ref', derived: 'der', variant: 'var', abbreviation: 'abbr' };
function Related({ rels }: { rels: HomeRel[] }) {
  const wide = useWide();
  if (!rels.length) return null;
  return (
    <section class="card-sec card-related" aria-label="Related words">
      <h3 class="card-label">Related</h3>
      <div class="chips">
        {rels.map((r, i) => (
          <a key={i} class="chip" href={href(wide ? wordPath(r.word) : searchPath(r.word))}><span class="chip-type">{RELTYPE[r.type] ?? r.type}</span> <span class="hangul" lang="ko">{r.word}</span></a>
        ))}
      </div>
    </section>
  );
}

interface PageSpec {
  key: string;
  kind: 'idiom' | 'word' | 'grammar';
  label: ComponentChildren;     // header strip, left: what this page is (Idiom / Proverb, Word, Grammar)
  level?: number;               // curriculum level badge (words and grammar)
  save: ItemInput;
  title: string;
  hanja?: string | null;
  pos?: string | null;
  gloss?: string | null;
  koDef?: string | null;
  examples: HomeExample[];
  related: HomeRel[];
  path: string;                 // where "Open entry" goes
}

function pageSpecs(h: HomeData): PageSpec[] {
  const out: PageSpec[] = [];
  if (h.idiom) {
    const i = h.idiom;
    out.push({
      key: `i${i.id}`, kind: 'idiom',
      label: <>{i.kind === 'proverb' ? 'Proverb' : 'Idiom'} <span class="card-kind-ko" lang="ko">{i.kind === 'proverb' ? '속담' : '관용구'}</span></>,
      save: { kind: 'entry', source: i.source, headword: i.headword, homonym: i.homonym, pos: i.kind, gloss: i.gloss },
      title: i.headword, gloss: i.gloss, koDef: i.ko_def, examples: i.examples, related: i.related, path: entryPath(i.source, i.id, i.headword),
    });
  }
  for (const w of h.words) {
    out.push({
      key: `w${w.id}`, kind: 'word', label: 'Word', level: w.level,
      save: { kind: 'entry', source: w.source, headword: w.headword, homonym: w.homonym, pos: w.pos, hanja: w.hanja, gloss: w.gloss },
      title: w.headword, hanja: w.hanja, pos: w.pos, gloss: w.gloss, koDef: w.ko_def, examples: w.examples, related: w.related, path: entryPath(w.source, w.id, w.headword),
    });
  }
  for (const g of h.grammar) {
    out.push({
      key: `g${g.id}`, kind: 'grammar',
      label: 'Grammar', level: g.level,
      save: { kind: 'grammar', pattern: g.pattern, summary: g.summary_en, level: g.level, ref: g.entry ? { source: g.entry.source, homonym: g.entry.homonym, pos: g.entry.pos } : null },
      title: g.pattern, gloss: g.summary_en, koDef: g.ko_def, examples: g.examples, related: g.related,
      path: g.entry ? entryPath(g.entry.source, g.entry.id, g.pattern) : wordPath(g.pattern),
    });
  }
  return out;
}

const MAX_EXAMPLES = 3;

/** One page: a framed card with fixed sections in a fixed order. Header strip (kind · level · save), the headword
 *  (hanja and part of speech under it) and its English meaning, then labelled blocks (Definition, Examples,
 *  Related; a block without content is left out) and "Open entry". The middle scrolls inside the card when long. */
function Page({ p, n, total }: { p: PageSpec; n: number; total: number }) {
  const exs = p.examples.slice(0, MAX_EXAMPLES);
  return (
    <div class="car-slide" role="group" aria-roledescription="slide" aria-label={`${n} of ${total}`}>
      <article class="today-card" data-kind={p.kind}>
        <header class="card-head">
          <span class="card-kind">{p.label}</span>
          {p.level != null && <CurriculumLevel level={p.level} />}
          <span class="card-spacer" />
          <SaveButton item={p.save} title={p.title} size={22} />
        </header>
        <div class="card-body">
          <div class="card-top">
            <h2 class="card-hw">
              <a class="card-link" href={href(p.path)}><span class="hangul" lang="ko">{p.title}</span></a>
            </h2>
            {(p.hanja || p.pos) && <div class="card-sub">{p.hanja && <Hanja text={p.hanja} />}<Pos pos={p.pos} /></div>}
            {p.gloss && <p class="card-gloss">{p.gloss}</p>}
          </div>
          {p.koDef && (
            <section class="card-sec" aria-label="Definition">
              <h3 class="card-label">Definition</h3>
              <p class="card-kodef hangul" lang="ko">{p.koDef}</p>
            </section>
          )}
          {exs.length > 0 && (
            <section class="card-sec card-exs" aria-label="Examples">
              <h3 class="card-label">Examples</h3>
              <ExampleList items={exs} />
            </section>
          )}
          <Related rels={p.related} />
        </div>
        <footer class="card-foot">
          <a class="btn primary card-open" href={href(p.path)}>Open entry</a>
        </footer>
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
        {pages.map((p, i) => <Page key={p.key} p={p} n={i + 1} total={n} />)}
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
