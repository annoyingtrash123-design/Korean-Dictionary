import type { ColorKey, Settings } from './settings';

export const COLOR_VARS: Record<ColorKey, string> = {
  accent: '--accent', bg: '--bg', text: '--text', hangul: '--hangul', hanja: '--hanja',
};
export const THEME_COLOR_FALLBACK = { light: '#faf8f5', dark: '#16181a', sepia: '#f3e8d2' } as const;

export function resolveTheme(theme: Settings['theme'], prefersDark: boolean): 'light' | 'dark' | 'sepia' {
  return theme === 'system' ? (prefersDark ? 'dark' : 'light') : theme;
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
  const bg = s.colors.bg ?? THEME_COLOR_FALLBACK[t];
  doc.querySelector('meta[name=theme-color]')?.setAttribute('content', bg);
  return t;
}

/** Computed colour (hex) currently in effect for a variable, for colour pickers. */
export function currentColor(k: ColorKey, doc: Document = document): string {
  const v = getComputedStyle(doc.documentElement).getPropertyValue(COLOR_VARS[k]).trim();
  return /^#[0-9a-f]{6}$/i.test(v) ? v : '#000000';
}
