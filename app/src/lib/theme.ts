import type { ColorKey, Settings } from './settings';

export const COLOR_VARS: Record<ColorKey, string> = {
  accent: '--accent', bg: '--bg', text: '--text', hangul: '--hangul', hanja: '--hanja',
};
/** Browser/OS status-bar colour: the forest menu-bar colour (`--bar`), so the top of the app is closed in. */
export const THEME_COLOR_FALLBACK = { light: '#294d38', dark: '#1f3a2a' } as const;

/** Light (parchment) or dark. A stored 'sepia' (an older theme) is now light. */
export function resolveTheme(theme: Settings['theme'], prefersDark: boolean): 'light' | 'dark' {
  if (theme === 'system') return prefersDark ? 'dark' : 'light';
  return theme === 'dark' ? 'dark' : 'light';
}

/** Apply settings to the document: data-theme, custom colour overrides, entry font size. */
export function applyTheme(s: Pick<Settings, 'theme' | 'colors' | 'fontSize'>, doc: Document = document, prefersDark?: boolean): string {
  const dark = prefersDark ?? (typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: dark)').matches);
  const t = resolveTheme(s.theme, dark);
  const root = doc.documentElement;
  root.setAttribute('data-theme', t);
  for (const k of Object.keys(COLOR_VARS) as ColorKey[]) {
    const v = s.colors[k];
    if (v) root.style.setProperty(COLOR_VARS[k], v);
    else root.style.removeProperty(COLOR_VARS[k]);
  }
  root.style.setProperty('--entry-font-size', `${s.fontSize}px`);
  doc.querySelector('meta[name=theme-color]')?.setAttribute('content', THEME_COLOR_FALLBACK[t]);
  return t;
}

/** Computed colour (hex) currently in effect for a variable, for colour pickers. */
export function currentColor(k: ColorKey, doc: Document = document): string {
  const v = getComputedStyle(doc.documentElement).getPropertyValue(COLOR_VARS[k]).trim();
  return /^#[0-9a-f]{6}$/i.test(v) ? v : '#000000';
}
