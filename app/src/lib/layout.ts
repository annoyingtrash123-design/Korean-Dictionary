import { useEffect, useState } from 'preact/hooks';
import type { RefObject } from 'preact';

/** Two-pane layout kicks in at this viewport width (iPad landscape, iPad Pro portrait, desktop). */
export const WIDE_QUERY = '(min-width: 900px)';
const mq = () => (typeof matchMedia === 'function' ? matchMedia(WIDE_QUERY) : undefined);

/** Live `min-width: 900px` flag; follows rotation, Split View and Stage Manager resizing. */
export function useWide(): boolean {
  const [wide, setWide] = useState(() => mq()?.matches ?? false);
  useEffect(() => {
    const m = mq(); if (!m) return;
    const f = () => setWide(m.matches);
    f();
    m.addEventListener('change', f);
    return () => m.removeEventListener('change', f);
  }, []);
  return wide;
}

const scrollMemo = new Map<string, number>();
/** Remember a scroll container's position per key so Back restores it (content is cached, so it paints at once). */
export function useScrollMemo(ref: RefObject<HTMLElement | null>, key: string) {
  useEffect(() => {
    const el = ref.current; if (!el) return;
    const saved = scrollMemo.get(key) ?? 0;
    el.scrollTop = saved;
    if (saved) requestAnimationFrame(() => { el.scrollTop = saved; });
    const on = () => scrollMemo.set(key, el.scrollTop);
    el.addEventListener('scroll', on, { passive: true });
    return () => el.removeEventListener('scroll', on);
  }, [key]);
}
