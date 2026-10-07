import { useEffect, useState } from 'preact/hooks';

export interface Async<T> { data: T | undefined; loading: boolean; error: string | undefined }
/** Run an async function when deps change; stale results are dropped. */
export function useAsync<T>(fn: () => Promise<T>, deps: unknown[]): Async<T> {
  const [s, setS] = useState<Async<T>>({ data: undefined, loading: true, error: undefined });
  useEffect(() => {
    let live = true;
    setS((p) => ({ data: p.data, loading: true, error: undefined }));
    fn().then((data) => live && setS({ data, loading: false, error: undefined }),
      (e) => live && setS({ data: undefined, loading: false, error: String(e?.message ?? e) }));
    return () => { live = false; };
  }, deps);
  return s;
}
