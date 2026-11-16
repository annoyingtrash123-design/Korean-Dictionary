// Bookmarks as older versions stored them (IndexedDB key `bookmarks`, one folder per item). They are
// read once and turned into lists (see lists.ts: `migrateBookmarks`), and old backup files in this
// format can still be imported. Nothing writes this format any more.
import { legacyKey, stableKey, type SavedRef } from './entry-key';

export interface Bookmark extends SavedRef {
  key: string;            // stableKey(): `${source}|${headword}|${homonym}|${pos}` (entries.id is NOT stable across data rebuilds)
  folder: string;         // folder id
  added: number;
  legacyId?: number;      // only on items saved by older versions (as source:id) until upgradeLegacy() resolves them
}
export interface Folder { id: string; name: string }
export interface BookmarkData { version: 1; folders: Folder[]; items: Bookmark[] }
export const DEFAULT_FOLDER: Folder = { id: 'saved', name: 'Saved' };

export const emptyBookmarks = (): BookmarkData => ({ version: 1, folders: [{ ...DEFAULT_FOLDER }], items: [] });

/** Defensive normalisation of stored or imported bookmark data. */
export function sanitizeBookmarks(raw: unknown): BookmarkData {
  const out = emptyBookmarks();
  const r = (raw ?? {}) as Partial<BookmarkData>;
  const folders = Array.isArray(r.folders) ? r.folders.filter((f) => f && typeof f.id === 'string' && typeof f.name === 'string') : [];
  for (const f of folders) if (!out.folders.some((o) => o.id === f.id)) out.folders.push({ id: f.id, name: f.name });
  const ids = new Set(out.folders.map((f) => f.id));
  const items = Array.isArray(r.items) ? r.items : [];
  const seen = new Set<string>();
  for (const b of items as (Partial<Bookmark> & { id?: unknown })[]) {
    if (!b || typeof b.source !== 'string' || typeof b.headword !== 'string') continue;
    const modern = 'pos' in b || 'homonym' in b;
    const legacyId = !modern && typeof b.id === 'number' ? b.id : undefined;
    if (!modern && legacyId == null) continue;
    const ref: SavedRef = {
      source: b.source, headword: b.headword, hanja: b.hanja ?? null, gloss: b.gloss ?? null,
      homonym: typeof b.homonym === 'number' ? b.homonym : null, pos: typeof b.pos === 'string' ? b.pos : null,
    };
    const key = legacyId != null ? legacyKey(b.source, b.headword, legacyId) : stableKey(ref);
    if (seen.has(key)) continue;
    seen.add(key);
    out.items.push({
      ...ref, key, ...(legacyId != null ? { legacyId } : {}),
      folder: typeof b.folder === 'string' && ids.has(b.folder) ? b.folder : DEFAULT_FOLDER.id, added: typeof b.added === 'number' ? b.added : Date.now(),
    });
  }
  return out;
}
