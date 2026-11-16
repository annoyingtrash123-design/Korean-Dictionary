// Loading from storage, as the app does at start: old bookmarks move into lists once, and the
// localStorage copy brings everything back after the browser clears IndexedDB.
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { clear, get, set } from 'idb-keyval';
import { userDb } from './idb';

const oldBookmarks = {
  version: 1,
  folders: [{ id: 'saved', name: 'Saved' }, { id: 'fv', name: 'Verbs' }],
  items: [
    { key: 'krdict|학교||noun', source: 'krdict', headword: '학교', homonym: null, pos: 'noun', hanja: '學校', gloss: 'school', folder: 'saved', added: 2 },
    { key: 'krdict|가다||verb', source: 'krdict', headword: '가다', homonym: null, pos: 'verb', hanja: null, gloss: 'go', folder: 'fv', added: 1 },
  ],
};

/** A fresh copy of the module, as after an app restart. */
async function restart() {
  vi.resetModules();
  const m = await import('./lists');
  await m.loadLists();
  return m;
}
const summary = (m: Awaited<ReturnType<typeof restart>>) => m.lists.get().lists.map((l) => `${l.name}: ${l.items.map((i) => (i.kind === 'entry' ? i.headword : i.pattern)).join(',')}`);

describe('loading lists', () => {
  beforeEach(async () => { await clear(userDb()); localStorage.clear(); });

  it('moves stored bookmarks into lists once, and never again', async () => {
    await set('bookmarks', oldBookmarks, userDb());
    let m = await restart();
    expect(summary(m)).toEqual(['Saved words: 학교', 'Verbs: 가다']);
    expect(await get('lists', userDb())).toMatchObject({ migrated: true });
    expect(await get('bookmarks', userDb())).toEqual(oldBookmarks);   // the old data stays (nothing is lost)

    // removing a migrated word must stick: the next start does not bring it back from the old bookmarks
    await m.removeFromList('saved', 'krdict|학교||noun');
    m = await restart();
    expect(summary(m)).toEqual(['Saved words: ', 'Verbs: 가다']);
    m = await restart();
    expect(summary(m)).toEqual(['Saved words: ', 'Verbs: 가다']);
  });

  it('keeps the default "Saved words" when nothing was stored', async () => {
    const m = await restart();
    expect(summary(m)).toEqual(['Saved words: ']);
    await m.toggleSave({ kind: 'grammar', pattern: '-아서/어서' });
    expect(summary(await restart())).toEqual(['Saved words: -아서/어서']);
  });

  it('restores lists from the localStorage copy after IndexedDB was cleared', async () => {
    let m = await restart();
    const id = await m.createList('Topik 6');
    await m.addToList(id, { kind: 'entry', source: 'krdict', headword: '고찰', homonym: null, pos: 'noun' });
    expect(localStorage.getItem('kd.backup.lists')).toContain('고찰');
    await clear(userDb());
    m = await restart();
    expect(summary(m)).toEqual(['Saved words: ', 'Topik 6: 고찰']);
    expect(await get('lists', userDb())).toBeTruthy();   // written back
  });

  it('migrates the localStorage copy of bookmarks saved by an older version', async () => {
    localStorage.setItem('kd.backup.bookmarks', JSON.stringify(oldBookmarks));
    const m = await restart();
    expect(summary(m)).toEqual(['Saved words: 학교', 'Verbs: 가다']);
    expect(localStorage.getItem('kd.backup.lists')).toContain('학교');
  });
});
