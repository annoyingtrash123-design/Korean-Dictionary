import { useEffect, useRef, useState } from 'preact/hooks';
import type { RefObject } from 'preact';

/** A draggable scroll bar for long scrolling panes (phone browsers' own scroll bars cannot be
 *  dragged). The pane scrolls live while the thumb moves; a bubble names the position. Shown only
 *  when the content is at least `minScreens` screens long. */
export function FastScroll({ target, label, minScreens = 2.5 }: {
  target: RefObject<HTMLElement | null>;
  /** Text for the bubble while dragging (e.g. the current chapter); default: percentage. */
  label?: (frac: number) => string;
  minScreens?: number;
}) {
  const [box, setBox] = useState<{ top: number; height: number } | null>(null);
  const [frac, setFrac] = useState(0);
  const [drag, setDrag] = useState(false);
  const [bubble, setBubble] = useState('');
  const grab = useRef(0);
  const track = useRef<HTMLDivElement>(null);
  const THUMB = 44;

  useEffect(() => {
    const el = target.current; if (!el) return;
    let raf = 0;
    const measure = () => {
      raf = 0;
      const long = el.scrollHeight > el.clientHeight * minScreens;
      setBox(long ? { top: el.offsetTop, height: el.clientHeight } : null);
      const max = el.scrollHeight - el.clientHeight;
      setFrac(max > 0 ? el.scrollTop / max : 0);
    };
    const kick = () => { if (!raf) raf = requestAnimationFrame(measure); };
    measure();
    el.addEventListener('scroll', kick, { passive: true });
    const ro = new ResizeObserver(kick);
    ro.observe(el);
    const mo = new MutationObserver(kick);   // content folding/unfolding changes the length
    mo.observe(el, { childList: true, subtree: true });
    return () => { el.removeEventListener('scroll', kick); ro.disconnect(); mo.disconnect(); if (raf) cancelAnimationFrame(raf); };
  }, [target.current, minScreens]);

  if (!box) return null;
  const travel = box.height - THUMB - 8;
  const thumbTop = 4 + frac * travel;

  const scrollTo = (clientY: number) => {
    const el = target.current, t = track.current; if (!el || !t) return;
    const y = clientY - t.getBoundingClientRect().top - grab.current - 4;
    const f = Math.min(1, Math.max(0, y / travel));
    el.scrollTop = f * (el.scrollHeight - el.clientHeight);
    setFrac(f);
    setBubble(label ? label(f) : `${Math.round(f * 100)}%`);
  };
  const onDown = (e: PointerEvent) => {
    e.preventDefault(); e.stopPropagation();
    const onThumb = (e.target as HTMLElement).classList.contains('fs-thumb');
    const t = track.current!.getBoundingClientRect();
    grab.current = onThumb ? e.clientY - t.top - thumbTop : THUMB / 2;   // a tap on the track jumps there
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    setDrag(true);
    scrollTo(e.clientY);
  };
  const onMove = (e: PointerEvent) => { if (drag) { e.preventDefault(); scrollTo(e.clientY); } };
  const onUp = () => setDrag(false);

  return (
    <div ref={track} class={`fs-track${drag ? ' drag' : ''}`} style={{ top: `${box.top}px`, height: `${box.height}px` }}
      onPointerDown={onDown} onPointerMove={onMove} onPointerUp={onUp} onPointerCancel={onUp}
      role="scrollbar" aria-orientation="vertical" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(frac * 100)} aria-label="Scroll through the text">
      <div class="fs-thumb" style={{ top: `${thumbTop}px`, height: `${THUMB}px` }} />
      {drag && bubble && <div class="fs-bubble" style={{ top: `${thumbTop + THUMB / 2}px` }}>{bubble}</div>}
    </div>
  );
}
