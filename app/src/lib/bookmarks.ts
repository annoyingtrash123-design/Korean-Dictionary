import { createStore } from './store';
import { get, set } from 'idb-keyval';
import { userDb } from './idb';
import { legacyKey, stableKey, upgradeLegacy, type EntryRef, type SavedRef } from './entry-key';

export const MAX_IMPORT_BYTES = 5 * 1024 * 1024;
export const MAX_IMPORT_ITEMS = 50_000;

export interface Bookmark extends SavedRef {
  key: string;            // stableKey(): `${source}|${headword}|${homonym}|${pos}` (entries.id is NOT stable across data rebuilds)
  folder: string;         // folder id
  added: number;
  legacyId?: number;      // only on items saved by older versions (as source:id) until upgradeLegacy() resolves them
}
export interface Folder { id: string; name: string }
export interface BookmarkData { version: 1; folders: Folder[]; items: Bookmark[] }
export const DEFAULT_FOLDER: Folder = { id: 'saved', name: 'Saved' };

export const emptyData = (): BookmarkData => ({ version: 1, folders: [{ ...DEFAULT_FOLDER }], items: [] });

export const bookmarks = createStore<BookmarkData>(emptyData());
let loaded: Promise<void> | undefined;

/** Defensive normalisation (also used for imports). */
export function sanitize(raw: unknown): BookmarkData {
  const out = emptyData();
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

function persist(d: BookmarkData) { return set('bookmarks', d, userDb()).catch(() => undefined); }
async function mutate(fn: (d: BookmarkData) => BookmarkData) {
  await loadBookmarks();
  const next = fn(bookmarks.get());
  bookmarks.set(next);
  await persist(next);
}

export function loadBookmarks(): Promise<void> {
  return (loaded ??= get('bookmarks', userDb())
    .catch(() => undefined)
    .then((v) => { if (v) bookmarks.set(sanitize(v)); }));
}
/** Resolve legacy `source:id` bookmarks once the engine is up (ids still match the data they were saved from). */
export async function upgradeBookmarks(lookup: Parameters<typeof upgradeLegacy>[1]): Promise<void> {
  await loadBookmarks();
  const r = await upgradeLegacy(bookmarks.get().items, lookup);
  if (!r.changed) return;
  await mutate((d) => ({ ...d, items: upgradeMerge(d.items, r.items) }));
}
/** Apply upgraded items onto the current list (which may have changed while the lookup ran). */
function upgradeMerge(cur: Bookmark[], upgraded: Bookmark[]): Bookmark[] {
  const up = new Map(upgraded.map((i) => [i.added + '|' + i.source + '|' + i.headword, i]));
  const seen = new Set<string>();
  return cur.map((i) => (i.legacyId != null ? up.get(i.added + '|' + i.source + '|' + i.headword) ?? i : i))
    .filter((i) => (seen.has(i.key) ? false : (seen.add(i.key), true)));
}

export const isBookmarked = (d: BookmarkData, e: EntryRef) => d.items.some((b) => b.key === stableKey(e));
export const findBookmark = (d: BookmarkData, e: EntryRef) => d.items.find((b) => b.key === stableKey(e));

export const addBookmark = (b: EntryRef & { hanja?: string | null; gloss?: string | null; folder?: string }) =>
  mutate((d) => {
    const ref: SavedRef = { source: b.source, headword: b.headword, hanja: b.hanja ?? null, gloss: b.gloss ?? null, homonym: b.homonym ?? null, pos: b.pos ?? null };
    const key = stableKey(ref);
    const folder = d.folders.some((f) => f.id === b.folder) ? b.folder! : DEFAULT_FOLDER.id;
    const rest = d.items.filter((i) => i.key !== key);
    return { ...d, items: [{ ...ref, key, folder, added: Date.now() }, ...rest] };
  });
export const removeBookmark = (key: string) =>
  mutate((d) => ({ ...d, items: d.items.filter((i) => i.key !== key) }));
export const moveBookmark = (key: string, folder: string) =>
  mutate((d) => ({ ...d, items: d.items.map((i) => (i.key === key ? { ...i, folder } : i)) }));
export const createFolder = (name: string) =>
  mutate((d) => ({ ...d, folders: [...d.folders, { id: 'f' + Date.now().toString(36) + Math.random().toString(36).slice(2, 5), name: name.trim() || 'Folder' }] }));
export const renameFolder = (id: string, name: string) =>
  mutate((d) => ({ ...d, folders: d.folders.map((f) => (f.id === id ? { ...f, name: name.trim() || f.name } : f)) }));
/** Deleting a folder moves its items to the default folder. */
export const deleteFolder = (id: string) =>
  mutate((d) => (id === DEFAULT_FOLDER.id ? d : {
    ...d, folders: d.folders.filter((f) => f.id !== id),
    items: d.items.map((i) => (i.folder === id ? { ...i, folder: DEFAULT_FOLDER.id } : i)),
  }));

export function exportBookmarks(d: BookmarkData = bookmarks.get()): string {
  return JSON.stringify({ app: 'korean-dictionary', exportedAt: new Date().toISOString(), ...d }, null, 2);
}
export class ImportError extends Error {}
/** Merge an imported backup into the existing data (folders by id, items by key). */
export async function importBookmarks(json: string): Promise<{ folders: number; items: number }> {
  if (json.length > MAX_IMPORT_BYTES) throw new ImportError('That file is too large (limit 5 MB).');
  const raw = JSON.parse(json);
  if (Array.isArray(raw?.items) && raw.items.length > MAX_IMPORT_ITEMS) throw new ImportError(`That file has too many bookmarks (limit ${MAX_IMPORT_ITEMS.toLocaleString('en-US')}).`);
  const inc = sanitize(raw);
  const added = { folders: 0, items: 0 };
  await mutate((d) => {
    const folders = [...d.folders];
    for (const f of inc.folders) if (!folders.some((x) => x.id === f.id)) { folders.push(f); added.folders++; }
    const items = [...d.items];
    for (const b of inc.items) if (!items.some((x) => x.key === b.key)) { items.push(b); added.items++; }
    return { ...d, folders, items };
  });
  return added;
}
