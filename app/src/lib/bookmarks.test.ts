import { beforeEach, describe, expect, it } from 'vitest';
import {
  addBookmark, bookmarks, createFolder, deleteFolder, emptyData, exportBookmarks, importBookmarks, isBookmarked,
  moveBookmark, removeBookmark, renameFolder, sanitize,
} from './bookmarks';

const bm = (id: number, extra = {}) => ({ source: 'krdict', id, headword: 'w' + id, hanja: null, gloss: 'g', ...extra });

describe('bookmark store', () => {
  beforeEach(() => bookmarks.set(emptyData()));

  it('adds, dedupes and removes', async () => {
    await addBookmark(bm(1));
    await addBookmark(bm(1));
    await addBookmark(bm(2));
    expect(bookmarks.get().items.map((i) => i.id)).toEqual([2, 1]);
    expect(isBookmarked(bookmarks.get(), 'krdict', 1)).toBe(true);
    await removeBookmark('krdict', 1);
    expect(isBookmarked(bookmarks.get(), 'krdict', 1)).toBe(false);
  });
  it('manages folders and moves items', async () => {
    await createFolder('Verbs');
    const f = bookmarks.get().folders.find((x) => x.name === 'Verbs')!;
    await addBookmark(bm(1));
    await moveBookmark('krdict:1', f.id);
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
    await addBookmark(bm(1, { folder: fid }));
    const json = exportBookmarks();
    bookmarks.set(emptyData());
    const r = await importBookmarks(json);
    expect(r).toEqual({ folders: 1, items: 1 });
    expect(bookmarks.get().items[0].folder).toBe(fid);
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
});
