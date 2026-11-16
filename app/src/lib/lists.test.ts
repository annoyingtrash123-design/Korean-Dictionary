import { beforeEach, describe, expect, it } from 'vitest';
import {
  addBookmark, bookmarks, createFolder, deleteFolder, emptyData, exportBookmarks, findBookmark, importBookmarks, isBookmarked,
  MAX_IMPORT_BYTES, MAX_IMPORT_ITEMS, moveBookmark, removeBookmark, renameFolder, sanitize, upgradeBookmarks,
} from './bookmarks';
import { sanitizeHistory } from './history';
import { refFromParams, resolveEntry, savedPath, stableKey } from './entry-key';

const ref = (n: number, extra = {}) => ({ source: 'krdict', headword: 'w' + n, homonym: null, pos: 'noun', hanja: null, gloss: 'g', ...extra });

describe('bookmark store', () => {
  beforeEach(() => bookmarks.set(emptyData()));

  it('keys bookmarks on source|headword|homonym|pos, not on ids', async () => {
    await addBookmark(ref(1, { homonym: 2 }));
    expect(bookmarks.get().items[0].key).toBe('krdict|w1|2|noun');
    expect(bookmarks.get().items[0]).not.toHaveProperty('id');
  });
  it('adds, dedupes and removes', async () => {
    await addBookmark(ref(1));
    await addBookmark(ref(1));
    await addBookmark(ref(2));
    expect(bookmarks.get().items.map((i) => i.headword)).toEqual(['w2', 'w1']);
    expect(isBookmarked(bookmarks.get(), ref(1))).toBe(true);
    await removeBookmark(stableKey(ref(1)));
    expect(isBookmarked(bookmarks.get(), ref(1))).toBe(false);
  });
  it('tells homonyms and parts of speech apart', async () => {
    await addBookmark(ref(1, { homonym: 1 }));
    await addBookmark(ref(1, { homonym: 2 }));
    await addBookmark(ref(1, { homonym: 2, pos: 'verb' }));
    expect(bookmarks.get().items).toHaveLength(3);
    expect(findBookmark(bookmarks.get(), ref(1, { homonym: 2, pos: 'verb' }))).toBeTruthy();
  });
  it('manages folders and moves items', async () => {
    await createFolder('Verbs');
    const f = bookmarks.get().folders.find((x) => x.name === 'Verbs')!;
    await addBookmark(ref(1));
    await moveBookmark(stableKey(ref(1)), f.id);
    expect(bookmarks.get().items[0].folder).toBe(f.id);
    await renameFolder(f.id, 'Verbs 2');
    expect(bookmarks.get().folders.find((x) => x.id === f.id)!.name).toBe('Verbs 2');
    await deleteFolder(f.id);
    expect(bookmarks.get().items[0].folder).toBe('saved');
    await deleteFolder('saved'); // default folder cannot be deleted
    expect(bookmarks.get().folders.some((x) => x.id === 'saved')).toBe(true);
  });
  it('round-trips export/import and merges', async () => {
    await createFolder('A');
    const fid = bookmarks.get().folders[1].id;
    await addBookmark(ref(1, { folder: fid, homonym: 3 }));
    const json = exportBookmarks();
    bookmarks.set(emptyData());
    expect(await importBookmarks(json)).toEqual({ folders: 1, items: 1 });
    expect(bookmarks.get().items[0].folder).toBe(fid);
    expect(bookmarks.get().items[0].homonym).toBe(3);
    expect(await importBookmarks(json)).toEqual({ folders: 0, items: 0 });
  });
  it('sanitises garbage', () => {
    const d = sanitize({ folders: [{ id: 'x' }, { id: 'ok', name: 'Ok' }], items: [{ source: 'a', id: 1, headword: 'h', folder: 'nope' }, { bad: 1 }] });
    expect(d.folders.map((f) => f.id)).toEqual(['saved', 'ok']);
    expect(d.items).toHaveLength(1);
    expect(d.items[0].folder).toBe('saved');
  });
  it('rejects invalid import JSON', async () => {
    await expect(importBookmarks('not json')).rejects.toThrow();
  });
  it('rejects oversized imports (bytes and item count)', async () => {
    await expect(importBookmarks(' '.repeat(MAX_IMPORT_BYTES + 1))).rejects.toThrow(/too large/);
    const many = JSON.stringify({ items: Array.from({ length: MAX_IMPORT_ITEMS + 1 }, (_, i) => ref(i)) });
    await expect(importBookmarks(many)).rejects.toThrow(/too many/);
    const ok = JSON.stringify({ items: Array.from({ length: 1000 }, (_, i) => ref(i)) });
    expect((await importBookmarks(ok)).items).toBe(1000);
  });
});

describe('legacy source:id bookmarks', () => {
  beforeEach(() => bookmarks.set(emptyData()));
  const old = { version: 1, folders: [], items: [
    { key: 'krdict:10', source: 'krdict', id: 10, headword: '배', hanja: null, gloss: 'pear', folder: 'saved', added: 1 },
    { key: 'krdict:11', source: 'krdict', id: 11, headword: '배', hanja: null, gloss: 'ship', folder: 'saved', added: 2 },
    { key: 'wikt:5', source: 'wikt', id: 5, headword: '밥', hanja: null, gloss: 'rice', folder: 'saved', added: 3 },
  ] };
  it('keeps same-headword legacy items apart until migrated', () => {
    const d = sanitize(old);
    expect(d.items).toHaveLength(3);
    expect(new Set(d.items.map((i) => i.key)).size).toBe(3);
    expect(d.items.every((i) => i.legacyId != null)).toBe(true);
  });
  it('migrates to stable keys using the old ids', async () => {
    bookmarks.set(sanitize(old));
    const db: Record<string, { headword: string; homonym?: number; pos?: string }> = {
      'krdict:10': { headword: '배', homonym: 1, pos: 'noun' }, 'krdict:11': { headword: '배', homonym: 2, pos: 'noun' },
      'wikt:5': { headword: 'DIFFERENT', pos: 'noun' }, // id now points elsewhere (data rebuilt)
    };
    // loadBookmarks() reads IndexedDB (empty here) so the in-memory legacy items stay as set above
    await upgradeBookmarks(async (s, id) => db[`${s}:${id}`] ?? null);
    const keys = bookmarks.get().items.map((i) => i.key).sort();
    expect(keys).toEqual(['krdict|배|1|noun', 'krdict|배|2|noun', 'wikt|밥||']);
    expect(bookmarks.get().items.some((i) => i.legacyId != null)).toBe(false);
    expect(bookmarks.get().items.find((i) => i.headword === '밥')!.gloss).toBe('rice');
  });
  it('leaves legacy items untouched if the engine is unavailable', async () => {
    bookmarks.set(sanitize(old));
    await upgradeBookmarks(async () => { throw new Error('no engine'); });
    expect(bookmarks.get().items.every((i) => i.legacyId != null)).toBe(true);
  });
  it('migrates stored history the same way', () => {
    const h = sanitizeHistory([{ key: 'krdict:10', source: 'krdict', id: 10, headword: '배', hanja: null, gloss: 'pear', ts: 5 }, { junk: 1 }]);
    expect(h).toHaveLength(1);
    expect(h[0].legacyId).toBe(10);
  });
});

describe('opening a saved word', () => {
  const entries = [
    { source: 'krdict', homonym: 1, pos: 'noun', id: 901 },
    { source: 'krdict', homonym: 2, pos: 'noun', id: 902 },
    { source: 'krdict', homonym: 2, pos: 'verb', id: 903 },
    { source: 'stdict', homonym: 1, pos: 'noun', id: 904 },
  ];
  it('matches source+homonym+pos regardless of id', () => {
    expect(resolveEntry(entries, { source: 'krdict', homonym: 2, pos: 'verb' })!.id).toBe(903);
    expect(resolveEntry(entries, { source: 'stdict', homonym: 1, pos: 'noun' })!.id).toBe(904);
  });
  it('falls back to the same source, then any entry', () => {
    expect(resolveEntry(entries, { source: 'krdict', homonym: 9, pos: 'verb' })!.id).toBe(903);
    expect(resolveEntry(entries, { source: 'krdict', homonym: 9, pos: 'adverb' })!.id).toBe(901);
    expect(resolveEntry(entries, { source: 'wikt', homonym: 1, pos: 'noun' })!.id).toBe(901);
    expect(resolveEntry([], { source: 'krdict' })).toBeNull();
  });
  it('round-trips through the route', () => {
    const p = savedPath({ source: 'krdict', headword: '먹다', homonym: 2, pos: 'verb' });
    expect(p.startsWith('/word/' + encodeURIComponent('먹다') + '?')).toBe(true);
    expect(refFromParams(new URLSearchParams(p.split('?')[1]))).toEqual({ source: 'krdict', homonym: 2, pos: 'verb' });
  });
});
