import { createStore } from './store';
import { get, set } from 'idb-keyval';
import { userDb } from './idb';

export interface Bookmark {
  key: string;            // `${source}:${id}`
  source: string; id: number;
  headword: string; hanja: string | null; gloss: string | null;
  folder: string;         // folder id
  added: number;
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
  for (const b of items) {
    if (!b || typeof b.source !== 'string' || typeof b.id !== 'number' || typeof b.headword !== 'string') continue;
    out.items.push({
      key: `${b.source}:${b.id}`, source: b.source, id: b.id, headword: b.headword,
      hanja: b.hanja ?? null, gloss: b.gloss ?? null,
      folder: ids.has(b.folder) ? b.folder : DEFAULT_FOLDER.id, added: typeof b.added === 'number' ? b.added : Date.now(),
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
export const isBookmarked = (d: BookmarkData, source: string, id: number) => d.items.some((b) => b.key === `${source}:${id}`);

export const addBookmark = (b: Omit<Bookmark, 'key' | 'added' | 'folder' | 'hanja' | 'gloss'> & { hanja?: string | null; gloss?: string | null; folder?: string }) =>
  mutate((d) => {
    const key = `${b.source}:${b.id}`;
    const folder = d.folders.some((f) => f.id === b.folder) ? b.folder! : DEFAULT_FOLDER.id;
    const rest = d.items.filter((i) => i.key !== key);
    return { ...d, items: [{ ...b, hanja: b.hanja ?? null, gloss: b.gloss ?? null, key, folder, added: Date.now() }, ...rest] };
  });
export const removeBookmark = (source: string, id: number) =>
  mutate((d) => ({ ...d, items: d.items.filter((i) => i.key !== `${source}:${id}`) }));
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
/** Merge an imported backup into the existing data (folders by id, items by key). */
export async function importBookmarks(json: string): Promise<{ folders: number; items: number }> {
  const inc = sanitize(JSON.parse(json));
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
