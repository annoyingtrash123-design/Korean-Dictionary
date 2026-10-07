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
export function navigate(path: string, replace = false): void {
  const h = '#' + path;
  if (location.hash === h) return;
  if (replace) {
    history.replaceState(history.state, '', h);
    window.dispatchEvent(new Event(EVT));
  } else location.hash = h;
}
export function useRoute(): Route {
  const [r, setR] = useState(currentRoute);
  useEffect(() => {
    const f = () => setR(currentRoute());
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
