import { useEffect, useState } from 'preact/hooks';

export interface Route { name: string; parts: string[]; params: URLSearchParams; raw: string }

export function parseHash(hash: string): Route {
  const raw = hash.replace(/^#/, '') || '/';
  const [path, qs = ''] = raw.split('?');
  const parts = path.split('/').filter(Boolean).map((p) => { try { return decodeURIComponent(p); } catch { return p; } });
  return { name: parts[0] ?? 'home', parts: parts.slice(1), params: new URLSearchParams(qs), raw };
}
export const currentRoute = (): Route => parseHash(location.hash);

const EVT = 'kd-route';

// Back while typing (keyboard up) should only close the keyboard: `undoBack` steps forward again
// and the router ignores that round trip (hashchanges until the hash is back where it was).
let shown = typeof location === 'undefined' ? '' : location.hash;   // hash the app shows
let holdAt: string | null = null;
// popstate also fires for new hash navigations (links, `navigate`); only a traversal (Back/Forward)
// leaves history.length unchanged.
let histLen = typeof history === 'undefined' ? 0 : history.length;
let traversal = false;
if (typeof window !== 'undefined') {
  window.addEventListener('popstate', () => { traversal = history.length === histLen; histLen = history.length; });
  window.addEventListener('hashchange', () => {
    if (holdAt === null) shown = location.hash;
    else if (location.hash === holdAt) holdAt = null;
  });
}
/** In a popstate handler: was this Back/Forward (not a link or `navigate`)? */
export const isTraversal = () => traversal;
/** Call from a popstate handler: undo the back step that just happened, without a route change. */
export function undoBack(): void {
  if (location.hash === shown) return;   // same URL: nothing for the router to see
  holdAt = shown;
  history.go(1);
}

export function navigate(path: string, replace = false): void {
  const h = '#' + path;
  if (location.hash === h) return;
  if (replace) {
    history.replaceState(history.state, '', h);
    shown = h;
    window.dispatchEvent(new Event(EVT));
  } else location.hash = h;
}
export function useRoute(): Route {
  const [r, setR] = useState(currentRoute);
  useEffect(() => {
    const f = () => { if (holdAt === null) setR(currentRoute()); };
    window.addEventListener('hashchange', f);
    window.addEventListener(EVT, f);
    return () => { window.removeEventListener('hashchange', f); window.removeEventListener(EVT, f); };
  }, []);
  return r;
}

/** Route hash without the ?tab= param (an entry is the same entry whichever tab shows). */
export const withoutTab = (raw: string) => raw.replace(/([?&])tab=[^&]*&?/, '$1').replace(/[?&]$/, '');
export const withTab = (raw: string, tab: string) => { const b = withoutTab(raw); return `${b}${b.includes('?') ? '&' : '?'}tab=${tab}`; };

export const searchPath = (q: string) => `/search?q=${encodeURIComponent(q)}`;
export const entryPath = (source: string, id: number, hw?: string) =>
  `/entry/${source}/${id}${hw ? `?hw=${encodeURIComponent(hw)}` : ''}`;
export const wordPath = (hw: string) => `/word/${encodeURIComponent(hw)}`;
export const hanjaPath = (ch: string) => `/hanja/${encodeURIComponent(ch)}`;
export const href = (path: string) => '#' + path;
export const readerPath = (id?: string) => (id ? `/reader/${encodeURIComponent(id)}` : '/reader');
