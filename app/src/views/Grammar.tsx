import { useEffect, useMemo, useRef, useState } from 'preact/hooks';
import { db } from '../db/client';
import type { GrammarRow } from '../db/types';
import { useAsync } from '../lib/useAsync';
import { Empty, LevelBadge } from '../components/common';
import { entryPath, href } from '../lib/router';

let grammarCache: GrammarRow[] | undefined;

export const prefetchGrammar = () => { if (!grammarCache) db.grammarList().then((g) => { if (g.length) grammarCache = g; }, () => undefined); };

export function GrammarView({ initialQ }: { initialQ: string }) {
  const r = useAsync(async () => { const g = await db.grammarList(); if (g.length) grammarCache = g; return g; }, [], () => grammarCache);
  const [cat, setCat] = useState('All');
  const [lvl, setLvl] = useState(0);
  const [q, setQ] = useState(initialQ);
  const cats = useMemo(() => ['All', ...[...new Set((r.data ?? []).map((g) => g.category))]], [r.data]);
  const [shownN, setShownN] = useState(60);
  const matched = (r.data ?? []).filter((g) =>
    (cat === 'All' || g.category === cat) && (!lvl || g.level === lvl) &&
    (!q.trim() || (g.pattern + ' ' + (g.summary_en ?? '')).toLowerCase().includes(q.trim().toLowerCase())));
  const list = matched.slice(0, shownN);
  useEffect(() => setShownN(60), [cat, lvl, q]);
  const sentinel = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = sentinel.current; if (!el) return;
    const io = new IntersectionObserver((e) => { if (e[0].isIntersecting) setShownN((n) => n + 100); }, { rootMargin: '600px' });
    io.observe(el); return () => io.disconnect();
  }, [matched.length, shownN]);
  return (
    <div class="page">
      <div class="grammar-filter">
        <input type="search" class="field" placeholder="Filter grammar…" aria-label="Filter grammar" value={q} onInput={(e) => setQ((e.currentTarget as HTMLInputElement).value)} />
        <div class="seg scroll" role="tablist" aria-label="Category">
          {cats.map((c) => <button key={c} type="button" role="tab" aria-selected={cat === c} class={cat === c ? 'on' : ''} onClick={() => setCat(c)}>{c}</button>)}
        </div>
        <div class="seg scroll" role="tablist" aria-label="Level">
          {[[0, 'All levels'], [1, '초 Beginner'], [2, '중 Interm.'], [3, '고 Advanced']].map(([v, l]) => (
            <button key={v} type="button" role="tab" aria-selected={lvl === v} class={lvl === v ? 'on' : ''} onClick={() => setLvl(v as number)}>{l}</button>
          ))}
        </div>
      </div>
      {r.loading && !r.data && <div class="skeleton" />}
      {r.data && matched.length === 0 && <Empty title="No grammar patterns match" />}
      <ul class="plain list">
        {list.map((g) => (
          <li key={g.id}>
            <a class="row" href={href(g.entry_id != null ? entryPath('krdict', g.entry_id, g.pattern) : `/search?q=${encodeURIComponent(g.pattern)}`)}>
              <div class="row-main">
                <div class="row-head"><span class="hangul hw" lang="ko">{g.pattern}</span><LevelBadge level={g.level} /></div>
                <div class="row-gloss">{g.summary_en}</div>
                <div class="row-src">{g.category}</div>
              </div>
            </a>
          </li>
        ))}
      </ul>
      {matched.length > shownN && <div ref={sentinel} class="sentinel" aria-hidden="true" />}
    </div>
  );
}
