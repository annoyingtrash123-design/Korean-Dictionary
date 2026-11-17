import { useEffect, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { Hanja, Pos } from './common';
import { SaveButton } from './SaveButton';
import { entryPath, href, wordPath } from '../lib/router';
import { curriculumHint, curriculumLabel } from '../lib/home-day';
import type { HomeExample, HomeGrammar, HomeIdiom, HomeWord } from '../db/types';

export function CurriculumLevel({ level }: { level: number }) {
  return <span class="cur-lvl" title={curriculumHint(level)} aria-label={curriculumHint(level)}>{curriculumLabel(level)}</span>;
}

/** Example sentences of a card: Korean, then English (with the AI chip when the English is an AI translation). */
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

function WordCard({ w }: { w: HomeWord }) {
  const path = entryPath(w.source, w.id, w.headword);
  return (
    <article class="today-card" data-kind="word">
      <header class="card-head">
        <CurriculumLevel level={w.level} />
        <SaveButton item={{ kind: 'entry', source: w.source, headword: w.headword, homonym: w.homonym, pos: w.pos, hanja: w.hanja, gloss: w.gloss }} title={w.headword} size={22} />
      </header>
      <h3 class="card-hw">
        <a class="card-link" href={href(path)}><span class="hangul" lang="ko">{w.headword}</span></a>
        {w.hanja && <Hanja text={w.hanja} />}
        <Pos pos={w.pos} />
      </h3>
      <p class="card-gloss">{w.gloss}</p>
      <CardExamples items={w.examples} />
    </article>
  );
}

function GrammarCard({ g }: { g: HomeGrammar }) {
  const path = g.entry ? entryPath(g.entry.source, g.entry.id, g.pattern) : wordPath(g.pattern);
  return (
    <article class="today-card" data-kind="grammar">
      <header class="card-head">
        <span class="card-tag">Grammar</span>
        <CurriculumLevel level={g.level} />
        <span class="card-spacer" />
        <SaveButton item={{ kind: 'grammar', pattern: g.pattern, summary: g.summary_en, level: g.level, ref: g.entry ? { source: g.entry.source, homonym: g.entry.homonym, pos: g.entry.pos } : null }} title={g.pattern} size={22} />
      </header>
      <h3 class="card-hw"><a class="card-link" href={href(path)}><span class="hangul" lang="ko">{g.pattern}</span></a></h3>
      {g.summary_en && <p class="card-gloss">{g.summary_en}</p>}
      <CardExamples items={g.examples} />
    </article>
  );
}

const STEP_KEYS: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };

/** Today's five cards (3 words, 2 grammar patterns): CSS scroll-snap track, drag with a mouse, arrow buttons and keys, dots. */
export function TodayCarousel({ words, grammar }: { words: HomeWord[]; grammar: HomeGrammar[] }) {
  const cards: ComponentChildren[] = [
    ...words.map((w) => <WordCard key={`w${w.id}`} w={w} />),
    ...grammar.map((g) => <GrammarCard key={`g${g.id}`} g={g} />),
  ];
  const n = cards.length;
  const track = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; left: number; moved: boolean } | null>(null);
  const [cur, setCur] = useState(0);
  if (!n) return null;

  const slide = (i: number) => track.current?.children[i] as HTMLElement | undefined;
  const goTo = (i: number) => {
    const t = track.current, s = slide(Math.max(0, Math.min(n - 1, i)));
    if (!t || !s) return;
    const smooth = !matchMedia('(prefers-reduced-motion: reduce)').matches;
    t.scrollTo({ left: s.offsetLeft - (t.clientWidth - s.offsetWidth) / 2, behavior: smooth ? 'smooth' : 'auto' });
  };
  // which card is centred (nearest to the middle of the viewport of the track)
  const onScroll = () => {
    const t = track.current;
    if (!t) return;
    const mid = t.scrollLeft + t.clientWidth / 2;
    let best = 0, bd = Infinity;
    for (let i = 0; i < n; i++) {
      const s = slide(i)!;
      const d = Math.abs(s.offsetLeft + s.offsetWidth / 2 - mid);
      if (d < bd) { bd = d; best = i; }
    }
    setCur(best);
  };
  useEffect(onScroll, [n]);

  const onPointerDown = (e: PointerEvent) => {
    if (e.pointerType !== 'mouse' || e.button !== 0 || !track.current) return;
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
    onScroll();
    // let scroll-snap settle on the nearest card
    const mid = t.scrollLeft + t.clientWidth / 2;
    let best = 0, bd = Infinity;
    for (let i = 0; i < n; i++) { const s = slide(i)!; const c = Math.abs(s.offsetLeft + s.offsetWidth / 2 - mid); if (c < bd) { bd = c; best = i; } }
    goTo(best);
  };
  const onKeyDown = (e: KeyboardEvent) => {
    const step = STEP_KEYS[e.key];
    if (!step || (e.target as HTMLElement).closest('button, input')) return;
    e.preventDefault();
    goTo(cur + step);
  };

  return (
    <section class="today" role="region" aria-roledescription="carousel" aria-label="Today: words and grammar">
      <div class="section-head">
        <h2>Today</h2>
        <div class="car-arrows">
          <button type="button" class="iconbtn car-arrow" aria-label="Previous card" disabled={cur <= 0} onClick={() => goTo(cur - 1)}><Icon name="back" size={22} /></button>
          <button type="button" class="iconbtn car-arrow" aria-label="Next card" disabled={cur >= n - 1} onClick={() => goTo(cur + 1)}><Icon name="chev" size={22} /></button>
        </div>
      </div>
      <div class="car-track" ref={track} tabIndex={0} aria-label="Cards, swipe or use the arrow keys" onScroll={onScroll} onKeyDown={onKeyDown}
        onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={endDrag} onPointerCancel={endDrag}
        onClickCapture={(e) => { if (drag.current?.moved) { e.preventDefault(); e.stopPropagation(); } }}>
        {cards.map((c, i) => (
          <div key={i} class="car-slide" role="group" aria-roledescription="slide" aria-label={`${i + 1} of ${n}`}>{c}</div>
        ))}
      </div>
      <div class="car-dots" role="group" aria-label="Choose a card">
        {cards.map((_, i) => (
          <button key={i} type="button" class="car-dot" aria-label={`${i + 1} of ${n}`} aria-current={i === cur ? 'true' : undefined} onClick={() => goTo(i)} />
        ))}
      </div>
    </section>
  );
}

export function IdiomCard({ i }: { i: HomeIdiom }) {
  const label = i.kind === 'proverb' ? '속담' : '관용구';
  return (
    <section class="idiom" aria-label="Idiom of the day">
      <div class="section-head"><h2>Idiom of the day</h2></div>
      <article class="today-card idiom-card" data-kind={i.kind}>
        <header class="card-head">
          <span class="card-tag" lang="ko" title={i.kind === 'proverb' ? 'Proverb' : 'Idiom'}>{label}</span>
          <span class="card-spacer" />
          <SaveButton item={{ kind: 'entry', source: i.source, headword: i.headword, homonym: i.homonym, pos: i.kind, gloss: i.gloss }} title={i.headword} size={22} />
        </header>
        <h3 class="card-hw"><a class="card-link" href={href(entryPath(i.source, i.id, i.headword))}><span class="hangul" lang="ko">{i.headword}</span></a></h3>
        <p class="card-gloss">{i.gloss}</p>
        {i.ko_def && <p class="card-kodef hangul" lang="ko">{i.ko_def}</p>}
        <CardExamples items={[i.example]} />
      </article>
    </section>
  );
}
