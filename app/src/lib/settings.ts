import { createStore } from './store';

export type ThemeName = 'light' | 'dark' | 'sepia' | 'system';
export type ColorKey = 'accent' | 'bg' | 'text' | 'hangul' | 'hanja';
export interface Settings {
  theme: ThemeName;
  colors: Partial<Record<ColorKey, string>>;
  fontSize: number;
  showKoDef: boolean;
  stdict: boolean;
}
export const SETTINGS_KEY = 'kd.settings';
export const DEFAULT_SETTINGS: Settings = { theme: 'system', colors: {}, fontSize: 17, showKoDef: true, stdict: true };

export function loadSettings(): Settings {
  try {
    const raw = JSON.parse(localStorage.getItem(SETTINGS_KEY) || '{}');
    return { ...DEFAULT_SETTINGS, ...raw, colors: { ...(raw.colors ?? {}) } };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}
export function saveSettings(s: Settings): void {
  try { localStorage.setItem(SETTINGS_KEY, JSON.stringify(s)); } catch { /* storage unavailable */ }
}

export const settings = createStore<Settings>(loadSettings());
settings.subscribe(saveSettings);
export const updateSettings = (patch: Partial<Settings>) => settings.set((s) => ({ ...s, ...patch }));
