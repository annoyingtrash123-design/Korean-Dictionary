import { useEffect, useState } from 'preact/hooks';

export interface Async<T> { data: T | undefined; loading: boolean; error: string | undefined }
/**
 * Run an async function when deps change; stale results are dropped.
 * `peek` may return an already-known value for the current deps so cached views paint synchronously.
 */
export function useAsync<T>(fn: () => Promise<T>, deps: unknown[], peek?: () => T | undefined): Async<T> {
  const init = (): Async<T> => { const v = peek?.(); return { data: v, loading: v === undefined, error: undefined }; };
  const [s, setS] = useState<Async<T>>(init);
  const [key, setKey] = useState(JSON.stringify(deps));
  const k = JSON.stringify(deps);
  let cur = s;
  if (k !== key) { // deps changed during render: swap in the cached value (if any) before paint
    cur = init(); setKey(k); setS(cur);
  }
  useEffect(() => {
    let live = true;
    const v = peek?.();
    if (v !== undefined) { setS({ data: v, loading: false, error: undefined }); return; }
    setS((p) => ({ data: p.data, loading: true, error: undefined }));
    fn().then((data) => live && setS({ data, loading: false, error: undefined }),
      (e) => live && setS({ data: undefined, loading: false, error: String(e?.message ?? e) }));
    return () => { live = false; };
  }, [k]);
  return cur;
}

/** True only once `on` has been true for `ms` (avoids spinner flashes for fast operations). */
export function useDelayed(on: boolean, ms = 150): boolean {
  const [v, setV] = useState(false);
  useEffect(() => {
    if (!on) { setV(false); return; }
    const t = setTimeout(() => setV(true), ms);
    return () => clearTimeout(t);
  }, [on]);
  return v;
}
