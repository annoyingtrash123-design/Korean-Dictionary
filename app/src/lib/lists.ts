// The user's own word lists, kept on the device (IndexedDB key `lists`, with a localStorage copy for
// when the browser clears IndexedDB). A list holds dictionary entries and grammar patterns; one item
// can be in several lists. Bookmarks saved by older versions become lists on first load.
import { get, set } from 'idb-keyval';
import { createStore } from './store';
import { userDb } from './idb';
import type { ErrEntry } from './errlog';
import { legacyKey, stableKey, upgradeLegacy, type EntryRef, type SavedRef } from './entry-key';
import { fromMirror, mirror } from './backup';
import { DEFAULT_FOLDER, sanitizeBookmarks, type BookmarkData } from './bookmarks';

export const MAX_IMPORT_BYTES = 5 * 1024 * 1024;
export const MAX_IMPORT_ITEMS = 50_000;
export const MAX_NAME = 80;

/** A dictionary entry, keyed like bookmarks were (`source|headword|homonym|pos`: entry ids change with every data rebuild). */
export interface EntryItem extends SavedRef { kind: 'entry'; key: string; added: number; legacyId?: number }
/** A grammar pattern (a row of the grammar table), keyed on its pattern text: row ids change with every data rebuild. */
export interface GrammarItem {
  kind: 'grammar'; key: string; pattern: string; summary: string | null; level: number | null; added: number;
  /** The dictionary entry the pattern opens (resolved by headword = pattern, like saved words). */
  ref: { source: string; homonym: number | null; pos: string | null } | null;
}
export type ListItem = EntryItem | GrammarItem;
export interface UserList { id: string; name: string; created: number; updated: number; items: ListItem[] }
export interface ListsData {
  version: 2;
  lists: UserList[];
  /** Where a tap on the save button puts things. */
  lastUsed: string;
  /** Old bookmarks have been moved into lists (so they are never merged in a second time). */
  migrated: boolean;
}

export type EntryInput = EntryRef & { hanja?: string | null; gloss?: string | null };
export type GrammarInput = { pattern: string; summary?: string | null; level?: number | null; ref?: GrammarItem['ref'] };
export type ItemInput = ({ kind: 'entry' } & EntryInput) | ({ kind: 'grammar' } & GrammarInput);

export const DEFAULT_LIST = { id: 'saved', name: 'Saved words' } as const;

export const grammarKey = (pattern: string) => `grammar|${pattern}`;
export const itemKey = (i: ItemInput) => (i.kind === 'grammar' ? grammarKey(i.pattern) : stableKey(i));

const newList = (id: string, name: string, now = Date.now()): UserList => ({ id, name, created: now, updated: now, items: [] });
export const emptyLists = (): ListsData => ({ version: 2, lists: [newList(DEFAULT_LIST.id, DEFAULT_LIST.name)], lastUsed: DEFAULT_LIST.id, migrated: false });
const cleanName = (s: string, fallback: string) => s.replace(/\s+/g, ' ').trim().slice(0, MAX_NAME) || fallback;
const newId = () => 'l' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);

export const lists = createStore<ListsData>(emptyLists());
let loaded: Promise<void> | undefined;

function makeItem(i: ItemInput, added = Date.now()): ListItem {
  if (i.kind === 'grammar') {
    return { kind: 'grammar', key: grammarKey(i.pattern), pattern: i.pattern, summary: i.summary ?? null, level: i.level ?? null, added, ref: i.ref ?? null };
  }
  const ref: SavedRef = { source: i.source, headword: i.headword, hanja: i.hanja ?? null, gloss: i.gloss ?? null, homonym: i.homonym ?? null, pos: i.pos ?? null };
  return { kind: 'entry', ...ref, key: stableKey(ref), added };
}

/** Defensive normalisation of one stored or imported item (items without `kind` are entries, as bookmarks were). */
function sanitizeItem(raw: unknown): ListItem | null {
  const b = raw as Partial<EntryItem> & Partial<GrammarItem> & { id?: unknown; kind?: unknown };
  if (!b || typeof b !== 'object') return null;
  const added = typeof b.added === 'number' ? b.added : Date.now();
  if (b.kind === 'grammar') {
    if (typeof b.pattern !== 'string' || !b.pattern) return null;
    const r = b.ref && typeof b.ref === 'object' && typeof b.ref.source === 'string'
      ? { source: b.ref.source, homonym: typeof b.ref.homonym === 'number' ? b.ref.homonym : null, pos: typeof b.ref.pos === 'string' ? b.ref.pos : null } : null;
    return makeItem({ kind: 'grammar', pattern: b.pattern, summary: typeof b.summary === 'string' ? b.summary : null, level: typeof b.level === 'number' ? b.level : null, ref: r }, added);
  }
  if (typeof b.source !== 'string' || typeof b.headword !== 'string') return null;
  // items saved as `source:id` by the oldest versions keep that id until upgradeLists() resolves them
  const legacyId = typeof b.legacyId === 'number' ? b.legacyId : !('pos' in b || 'homonym' in b) && typeof b.id === 'number' ? b.id : undefined;
  if (legacyId == null && !('pos' in b || 'homonym' in b)) return null;
  const it = makeItem({ kind: 'entry', source: b.source, headword: b.headword, hanja: b.hanja ?? null, gloss: b.gloss ?? null,
    homonym: typeof b.homonym === 'number' ? b.homonym : null, pos: typeof b.pos === 'string' ? b.pos : null }, added) as EntryItem;
  return legacyId != null ? { ...it, key: legacyKey(b.source, b.headword, legacyId), legacyId } : it;
}

function sanitizeItems(raw: unknown): ListItem[] {
  const out: ListItem[] = [];
  const seen = new Set<string>();
  for (const r of Array.isArray(raw) ? raw : []) {
    const it = sanitizeItem(r);
    if (it && !seen.has(it.key)) { seen.add(it.key); out.push(it); }
  }
  return out;
}

/** Defensive normalisation of stored or imported lists data. */
export function sanitizeLists(raw: unknown): ListsData {
  const r = (raw ?? {}) as Partial<ListsData>;
  const out: ListsData = { version: 2, lists: [], lastUsed: DEFAULT_LIST.id, migrated: r.migrated === true };
  for (const l of Array.isArray(r.lists) ? r.lists : []) {
    if (!l || typeof l.id !== 'string' || !l.id || typeof l.name !== 'string' || out.lists.some((x) => x.id === l.id)) continue;
    const now = Date.now();
    out.lists.push({
      id: l.id, name: cleanName(l.name, 'List'), items: sanitizeItems(l.items),
      created: typeof l.created === 'number' ? l.created : now, updated: typeof l.updated === 'number' ? l.updated : now,
    });
  }
  if (typeof r.lastUsed === 'string' && out.lists.some((l) => l.id === r.lastUsed)) out.lastUsed = r.lastUsed;
  return out;
}

/**
 * Move old bookmarks into lists: the default folder becomes "Saved words", every other folder a list of
 * the same name. Merges by list id and item key, so running it again changes nothing.
 */
export function migrateBookmarks(base: ListsData, bm: BookmarkData): ListsData {
  const out: ListsData = { ...base, lists: base.lists.map((l) => ({ ...l, items: [...l.items] })), migrated: true };
  for (const f of bm.folders) {
    const items = sanitizeItems(bm.items.filter((i) => i.folder === f.id));
    const isDefault = f.id === DEFAULT_FOLDER.id;
    let list = out.lists.find((l) => l.id === f.id);
    if (!list) {
      if (isDefault && !items.length) continue;
      const times = items.map((i) => i.added);
      list = { ...newList(f.id, isDefault ? DEFAULT_LIST.name : cleanName(f.name, 'List')), ...(times.length ? { created: Math.min(...times), updated: Math.max(...times) } : {}) };
      out.lists.push(list);
    }
    const have = new Set(list.items.map((i) => i.key));
    for (const it of items) if (!have.has(it.key)) { have.add(it.key); list.items.push(it); }
  }
  return out;
}

function persist(d: ListsData) { mirror('lists', d); return set('lists', d, userDb()).catch(() => undefined); }
async function mutate(fn: (d: ListsData) => ListsData) {
  await loadLists();
  const next = fn(lists.get());
  lists.set(next);
  await persist(next);
}

export function loadLists(): Promise<void> {
  return (loaded ??= (async () => {
    const v = await get('lists', userDb()).catch(() => undefined);
    // IndexedDB cleared by the browser: restore from the localStorage copy
    const saved = v ?? fromMirror('lists');
    let d = saved ? sanitizeLists(saved) : undefined;
    if (!d?.migrated) {
      const bm = (await get('bookmarks', userDb()).catch(() => undefined)) ?? fromMirror('bookmarks');
      if (bm || d) d = migrateBookmarks(d ?? lists.get(), sanitizeBookmarks(bm));
    }
    if (!d) return;   // nothing stored yet: keep the default (one empty "Saved words")
    lists.set(d);
    if (!v || !(v as Partial<ListsData>).migrated) await persist(d);
    else mirror('lists', v);   // keep the copy current for data saved before it existed
  })());
}

/** Resolve legacy `source:id` items once the engine is up (ids still match the data they were saved from). */
export async function upgradeLists(lookup: Parameters<typeof upgradeLegacy>[1]): Promise<void> {
  await loadLists();
  const legacy = new Map<string, EntryItem>();
  for (const l of lists.get().lists) for (const i of l.items) if (i.kind === 'entry' && i.legacyId != null) legacy.set(i.key, i);
  if (!legacy.size) return;
  const done = new Map<string, EntryItem>();
  for (const [k, it] of legacy) {
    const r = await upgradeLegacy([it], lookup);
    if (!r.changed) return;   // engine unavailable: keep them as they are
    done.set(k, { ...r.items[0], kind: 'entry' });
  }
  await mutate((d) => ({
    ...d, lists: d.lists.map((l) => {
      const seen = new Set<string>();
      const items = l.items.map((i) => done.get(i.key) ?? i).filter((i) => (seen.has(i.key) ? false : (seen.add(i.key), true)));
      return { ...l, items };
    }),
  }));
}

// ---------- queries ----------
export const findList = (d: ListsData, id: string) => d.lists.find((l) => l.id === id);
export const inList = (l: UserList | undefined, key: string) => !!l?.items.some((i) => i.key === key);
export const listsWith = (d: ListsData, key: string) => d.lists.filter((l) => inList(l, key));
/** The list a tap on the save button uses: the last one picked, else "Saved words", else the first. */
export const targetList = (d: ListsData): UserList | undefined => findList(d, d.lastUsed) ?? findList(d, DEFAULT_LIST.id) ?? d.lists[0];
export const itemCount = (d: ListsData) => new Set(d.lists.flatMap((l) => l.items.map((i) => i.key))).size;

// ---------- changes ----------
const touch = (l: UserList, items: ListItem[]): UserList => ({ ...l, items, updated: Date.now() });

export async function createList(name: string): Promise<string> {
  const id = newId();
  await mutate((d) => ({ ...d, lists: [...d.lists, newList(id, cleanName(name, 'New list'))] }));
  return id;
}
export const renameList = (id: string, name: string) =>
  mutate((d) => ({ ...d, lists: d.lists.map((l) => (l.id === id ? { ...l, name: cleanName(name, l.name), updated: Date.now() } : l)) }));
export const deleteList = (id: string) => mutate((d) => ({ ...d, lists: d.lists.filter((l) => l.id !== id) }));
export const setLastUsed = (id: string) => mutate((d) => (d.lastUsed === id ? d : { ...d, lastUsed: id }));

/** Add to a list (newest first); already there: unchanged. */
export const addToList = (id: string, input: ItemInput) =>
  mutate((d) => ({ ...d, lists: d.lists.map((l) => (l.id !== id || inList(l, itemKey(input)) ? l : touch(l, [makeItem(input), ...l.items]))) }));
export const removeFromList = (id: string, key: string) =>
  mutate((d) => ({ ...d, lists: d.lists.map((l) => (l.id === id && inList(l, key) ? touch(l, l.items.filter((i) => i.key !== key)) : l)) }));
/** Put a removed item back where it was (Undo). */
export const restoreItem = (id: string, item: ListItem, index: number) =>
  mutate((d) => ({ ...d, lists: d.lists.map((l) => {
    if (l.id !== id || inList(l, item.key)) return l;
    const items = [...l.items]; items.splice(Math.min(index, items.length), 0, item);
    return touch(l, items);
  }) }));

/** The save button's tap: toggle the item in the target list ("Saved words" is made again if it was deleted). */
export async function toggleSave(input: ItemInput): Promise<{ list: UserList; removed: boolean }> {
  await loadLists();
  if (!targetList(lists.get())) await mutate((d) => ({ ...d, lists: [newList(DEFAULT_LIST.id, DEFAULT_LIST.name)], lastUsed: DEFAULT_LIST.id }));
  const list = targetList(lists.get())!;
  const key = itemKey(input);
  const removed = inList(list, key);
  if (removed) await removeFromList(list.id, key); else await addToList(list.id, input);
  return { list: findList(lists.get(), list.id)!, removed };
}

// ---------- backup ----------
/** `errors` (the on-device error log) is an optional extra for diagnosis; import ignores it and works without it. */
export function exportBackup(d: ListsData = lists.get(), errors: ErrEntry[] = []): string {
  return JSON.stringify({ app: 'korean-dictionary', exportedAt: new Date().toISOString(), ...d, ...(errors.length ? { errors } : {}) }, null, 2);
}
export class ImportError extends Error {}

/** Merge a backup into the existing lists (lists by id, items by key). Old bookmark backups go into "Saved words" (and their folders' lists). */
export async function importBackup(json: string): Promise<{ lists: number; items: number }> {
  if (json.length > MAX_IMPORT_BYTES) throw new ImportError('That file is too large (limit 5 MB).');
  const raw = JSON.parse(json);
  const n = Array.isArray(raw?.lists) ? raw.lists.reduce((s: number, l: { items?: unknown }) => s + (Array.isArray(l?.items) ? l.items.length : 0), 0)
    : Array.isArray(raw?.items) ? raw.items.length : 0;
  if (n > MAX_IMPORT_ITEMS) throw new ImportError(`That file has too many saved items (limit ${MAX_IMPORT_ITEMS.toLocaleString('en-US')}).`);
  const inc = Array.isArray(raw?.lists) ? sanitizeLists(raw)
    : migrateBookmarks({ version: 2, lists: [], lastUsed: DEFAULT_LIST.id, migrated: true }, sanitizeBookmarks(raw));
  const added = { lists: 0, items: 0 };
  await mutate((d) => {
    const out = d.lists.map((l) => ({ ...l }));
    for (const l of inc.lists) {
      const cur = out.find((x) => x.id === l.id);
      if (!cur) { out.push(l); added.lists++; added.items += l.items.length; continue; }
      const have = new Set(cur.items.map((i) => i.key));
      const extra = l.items.filter((i) => !have.has(i.key));
      if (extra.length) { cur.items = [...cur.items, ...extra]; cur.updated = Date.now(); added.items += extra.length; }
    }
    return { ...d, lists: out };
  });
  return added;
}
