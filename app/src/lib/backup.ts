// A copy of the user's own data (bookmarks, history, game progress) in localStorage.
// When Chrome is short of space it clears a site's IndexedDB and file storage but keeps its
// localStorage, so this copy brings the data back after such a clean-up.

const PREFIX = 'kd.backup.';

export function mirror(key: string, value: unknown): void {
  try { localStorage.setItem(PREFIX + key, JSON.stringify(value)); } catch { /* full or unavailable: the main copy still exists */ }
}

export function fromMirror<T>(key: string): T | undefined {
  try { const s = localStorage.getItem(PREFIX + key); return s ? (JSON.parse(s) as T) : undefined; } catch { return undefined; }
}
