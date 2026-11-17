import { useEffect, useLayoutEffect, useRef, useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { FastScroll } from './FastScroll';

const OVERSCAN = 6;

/**
 * A long list of equally tall rows (thousands of them) with only the visible ones in the page. Rows are
 * fetched `pageSize` at a time as they come into view; rows not loaded yet show a placeholder, and the
 * scroll bar and the draggable fast-scroll thumb cover the whole list. Fill the space below a header
 * (the parent is a flex column); remount with a new `key` to change the list.
 */
export function VirtualList<T>({ count, rowHeight, pageSize, fetchPage, renderRow, startRow = 0, onRow, label }: {
  count: number;
  rowHeight: number;
  pageSize: number;
  fetchPage: (offset: number, limit: number) => Promise<T[]>;
  renderRow: (item: T, index: number) => ComponentChildren;
  /** First row to show (restoring a remembered position). */
  startRow?: number;
  /** Called (throttled) with the first visible row as the list scrolls. */
  onRow?: (row: number) => void;
  /** Accessible name of the list. */
  label: string;
}) {
  const scroller = useRef<HTMLDivElement>(null);
  const pages = useRef(new Map<number, T[]>());
  const busy = useRef(new Set<number>());
  const [, bump] = useState(0);
  const [error, setError] = useState(false);
  const first0 = Math.min(Math.max(0, startRow), Math.max(0, count - 1));
  const [win, setWin] = useState({ first: Math.max(0, first0 - OVERSCAN), last: first0 + 14 });
  const onRowRef = useRef(onRow);
  onRowRef.current = onRow;

  // start at the remembered row (the content is as tall as the whole list from the first render)
  useLayoutEffect(() => { if (scroller.current && first0 > 0) scroller.current.scrollTop = first0 * rowHeight; }, []);

  useEffect(() => {
    const el = scroller.current; if (!el) return;
    let raf = 0, saved = 0;
    const measure = () => {
      raf = 0;
      const row = Math.floor(el.scrollTop / rowHeight);
      const first = Math.max(0, row - OVERSCAN);
      const last = Math.min(count - 1, Math.ceil((el.scrollTop + el.clientHeight) / rowHeight) + OVERSCAN);
      setWin((w) => (w.first === first && w.last === last ? w : { first, last }));
      if (onRowRef.current) { clearTimeout(saved); saved = window.setTimeout(() => onRowRef.current?.(row), 250); }
    };
    const kick = () => { if (!raf) raf = requestAnimationFrame(measure); };
    measure();
    el.addEventListener('scroll', kick, { passive: true });
    const ro = new ResizeObserver(kick);
    ro.observe(el);
    return () => { el.removeEventListener('scroll', kick); ro.disconnect(); if (raf) cancelAnimationFrame(raf); clearTimeout(saved); };
  }, [count, rowHeight]);

  // load the pages the window touches (a moment after the window settles: dragging the thumb passes many pages)
  useEffect(() => {
    const need: number[] = [];
    for (let p = Math.floor(win.first / pageSize); p <= Math.floor(Math.max(win.last, 0) / pageSize); p++) if (!pages.current.has(p) && !busy.current.has(p)) need.push(p);
    if (!need.length) return;
    const t = setTimeout(() => {
      for (const p of need) {
        if (busy.current.has(p)) continue;
        busy.current.add(p);
        fetchPage(p * pageSize, pageSize).then((rows) => { pages.current.set(p, rows); setError(false); }, () => setError(true)).finally(() => { busy.current.delete(p); bump((n) => n + 1); });
      }
    }, pages.current.size === 0 ? 0 : 40);
    return () => clearTimeout(t);
  }, [win.first, win.last, pageSize, error]);

  const rows: ComponentChildren[] = [];
  for (let i = win.first; i <= Math.min(win.last, count - 1); i++) {
    const item = pages.current.get(Math.floor(i / pageSize))?.[i % pageSize];
    rows.push(item === undefined
      ? <li key={`p${i}`} class="vl-row vl-skel" style={{ height: `${rowHeight}px` }} aria-hidden="true"><span class="skel-bar" /></li>
      : <li key={i} class="vl-row" style={{ height: `${rowHeight}px` }} aria-posinset={i + 1} aria-setsize={count}>{renderRow(item, i)}</li>);
  }
  return (
    <>
      <div class="vl-scroll" ref={scroller}>
        <div class="vl-space" style={{ height: `${count * rowHeight}px` }}>
          <ul class="plain vl-rows" aria-label={label} style={{ top: `${win.first * rowHeight}px` }}>{rows}</ul>
        </div>
        {error && <div class="vl-error" role="alert">Could not load the list. <button type="button" class="link" onClick={() => setError(false)}>Retry</button></div>}
      </div>
      <FastScroll target={scroller} minScreens={2.5} label={(f) => `${(Math.round(f * (count - 1)) + 1).toLocaleString('en-US')} / ${count.toLocaleString('en-US')}`} />
    </>
  );
}
