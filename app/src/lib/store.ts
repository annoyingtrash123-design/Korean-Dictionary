import { useEffect, useState } from 'preact/hooks';

/** Minimal observable store. */
export interface Store<T> {
  get(): T;
  set(v: T | ((prev: T) => T)): void;
  subscribe(fn: (v: T) => void): () => void;
}
export function createStore<T>(initial: T): Store<T> {
  let value = initial;
  const subs = new Set<(v: T) => void>();
  return {
    get: () => value,
    set(v) {
      value = typeof v === 'function' ? (v as (p: T) => T)(value) : v;
      subs.forEach((f) => f(value));
    },
    subscribe(fn) { subs.add(fn); return () => subs.delete(fn); },
  };
}
export function useStore<T>(s: Store<T>): T {
  const [v, setV] = useState(s.get());
  useEffect(() => { setV(s.get()); return s.subscribe(setV); }, [s]);
  return v;
}
