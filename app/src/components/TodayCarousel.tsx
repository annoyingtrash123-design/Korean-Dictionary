import { useEffect, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { Hanja, Pos } from './common';
import { SaveButton } from './SaveButton';
import { entryPath, href, searchPath, wordPath } from '../lib/router';
import { curriculumHint, curriculumLabel } from '../lib/home-day';
import { useWide } from '../lib/layout';
import type { HomeData, HomeExample, HomeRel } from '../db/types';
import type { ItemInput } from '../lib/lists';

export function CurriculumLevel({ level }: { level: number }) {
  return <span class="cur-lvl" title={curriculumHint(level)} aria-label={curriculumHint(level)}>{curriculumLabel(level)}</span>;
}

/** Example sentences: Korean, then English (with the AI chip when the English is an AI translation); a dialogue keeps one turn per line. */
export function CardExamples({ items }: { items: HomeExample[] }) {
  if (!items.length) return null;
  return (
    <ul class="plain card-exs" aria-label="Examples">
      {items.map((x, i) => (
        <li key={i} class="card-ex">
          {x.ko.split('\n').map((l, j) => <div key={j} class="ex-ko hangul" lang="ko">{l}</div>)}
          {x.en && x.en.split('\n').map((l, j) => <div key={j} class="ex-en">{x.en_ai && j === 0 && <span class="chip-ai ex-ai" title="AI translation, reviewed">AI</span>}{l}</div>)}
        </li>
      ))}
    </ul>
  );
}

const RELTYPE: Record<string, string> = { synonym: 'syn', antonym: 'ant', honorific: 'hon', humble: 'hum', 'see also': 'see', reference: 'ref', derived: 'der', variant: 'var', abbreviation: 'abbr' };
function Related({ rels }: { rels: HomeRel[] }) {
  const wide = useWide();
  if (!rels.length) return null;
  return (
    <section class="card-related" aria-label="Related words">
      <h4>Related</h4>
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
  tag: ComponentChildren;       // top-left: the level badge, or the idiom / proverb label
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
      tag: <span class="card-tag" lang="ko" title={i.kind === 'proverb' ? 'Proverb' : 'Idiom'}>{i.kind === 'proverb' ? '속담' : '관용구'}</span>,
      save: { kind: 'entry', source: i.source, headword: i.headword, homonym: i.homonym, pos: i.kind, gloss: i.gloss },
      title: i.headword, gloss: i.gloss, koDef: i.ko_def, examples: i.examples, related: i.related, path: entryPath(i.source, i.id, i.headword),
    });
  }
  for (const w of h.words) {
    out.push({
      key: `w${w.id}`, kind: 'word', tag: <CurriculumLevel level={w.level} />,
      save: { kind: 'entry', source: w.source, headword: w.headword, homonym: w.homonym, pos: w.pos, hanja: w.hanja, gloss: w.gloss },
      title: w.headword, hanja: w.hanja, pos: w.pos, gloss: w.gloss, koDef: w.ko_def, examples: w.examples, related: w.related, path: entryPath(w.source, w.id, w.headword),
    });
  }
  for (const g of h.grammar) {
    out.push({
      key: `g${g.id}`, kind: 'grammar',
      tag: <><span class="card-tag">Grammar</span><CurriculumLevel level={g.level} /></>,
      save: { kind: 'grammar', pattern: g.pattern, summary: g.summary_en, level: g.level, ref: g.entry ? { source: g.entry.source, homonym: g.entry.homonym, pos: g.entry.pos } : null },
      title: g.pattern, gloss: g.summary_en, koDef: g.ko_def, examples: g.examples, related: g.related,
      path: g.entry ? entryPath(g.entry.source, g.entry.id, g.pattern) : wordPath(g.pattern),
    });
  }
  return out;
}

/** One whole page: badge + star, the headword large, English, Korean definition, examples, related words, "Open entry". */
function Page({ p, n, total }: { p: PageSpec; n: number; total: number }) {
  return (
    <div class="car-slide" role="group" aria-roledescription="slide" aria-label={`${n} of ${total}`}>
      <article class="today-card" data-kind={p.kind}>
        <header class="card-head">
          {p.tag}
          <span class="card-spacer" />
          <SaveButton item={p.save} title={p.title} size={24} />
        </header>
        <h2 class="card-hw">
          <a class="card-link" href={href(p.path)}><span class="hangul" lang="ko">{p.title}</span></a>
          {p.hanja && <Hanja text={p.hanja} />}
          <Pos pos={p.pos} />
        </h2>
        {p.gloss && <p class="card-gloss">{p.gloss}</p>}
        {p.koDef && <p class="card-kodef hangul" lang="ko">{p.koDef}</p>}
        <CardExamples items={p.examples} />
        <Related rels={p.related} />
        <a class="btn primary card-open" href={href(p.path)}>Open entry</a>
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
