import { createStore } from './store';

export type ThemeName = 'light' | 'dark' | 'sepia' | 'system';
export type ColorKey = 'accent' | 'bg' | 'text' | 'hangul' | 'hanja';
export interface Settings {
  theme: ThemeName;
  colors: Partial<Record<ColorKey, string>>;
  fontSize: number;
  showKoDef: boolean;
  /** optional dictionary packs enabled for search, by pack id */
  packs: Record<string, boolean>;
}
export const SETTINGS_KEY = 'kd.settings';
export const DEFAULT_SETTINGS: Settings = { theme: 'system', colors: {}, fontSize: 17, showKoDef: true, packs: { stdict: true, opendict: false } };

/** Fill defaults; migrate the old boolean `stdict` into `packs.stdict`. */
export function migrateSettings(raw: any): Settings {
  const packs = { ...DEFAULT_SETTINGS.packs, ...(raw.packs ?? {}) };
  if (typeof raw.stdict === 'boolean' && !raw.packs) packs.stdict = raw.stdict;
  const { stdict: _old, ...rest } = raw;
  return { ...DEFAULT_SETTINGS, ...rest, packs, colors: { ...(rest.colors ?? {}) } };
}

export function loadSettings(): Settings {
  try {
    return migrateSettings(JSON.parse(localStorage.getItem(SETTINGS_KEY) || '{}'));
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
