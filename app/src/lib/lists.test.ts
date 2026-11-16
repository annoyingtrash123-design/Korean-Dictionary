import { beforeEach, describe, expect, it } from 'vitest';
import {
  addToList, createList, deleteList, DEFAULT_LIST, emptyLists, exportBackup, findList, grammarKey, importBackup, inList, itemCount, lists, listsWith,
  MAX_IMPORT_BYTES, MAX_IMPORT_ITEMS, migrateBookmarks, removeFromList, renameList, restoreItem, sanitizeLists, setLastUsed, targetList, toggleSave,
  upgradeLists, type ItemInput,
} from './lists';
import { sanitizeBookmarks } from './bookmarks';
import { sanitizeHistory } from './history';
import { refFromParams, resolveEntry, savedPath, stableKey } from './entry-key';

const word = (n: number, extra = {}): ItemInput & { kind: 'entry' } => ({ kind: 'entry', source: 'krdict', headword: 'w' + n, homonym: null, pos: 'noun', hanja: null, gloss: 'g', ...extra });
const gram = (pattern: string): ItemInput => ({ kind: 'grammar', pattern, summary: 'because', level: 1, ref: { source: 'krdict', homonym: null, pos: null } });
const saved = () => findList(lists.get(), DEFAULT_LIST.id)!;
const keys = (id: string) => findList(lists.get(), id)!.items.map((i) => i.key);

describe('list store', () => {
  beforeEach(() => lists.set(emptyLists()));

  it('starts with an empty "Saved words" that taps go to', () => {
    expect(lists.get().lists.map((l) => l.name)).toEqual(['Saved words']);
    expect(targetList(lists.get())!.id).toBe('saved');
  });
  it('keys entries on source|headword|homonym|pos and grammar on the pattern text', async () => {
    await addToList('saved', word(1, { homonym: 2 }));
    await addToList('saved', gram('-아서/어서'));
    expect(keys('saved')).toEqual([grammarKey('-아서/어서'), 'krdict|w1|2|noun']);
    expect(saved().items[0]).toMatchObject({ kind: 'grammar', pattern: '-아서/어서', summary: 'because', level: 1 });
    expect(saved().items[1]).not.toHaveProperty('id');
  });
  it('adds newest first, dedupes, removes', async () => {
    await addToList('saved', word(1));
    await addToList('saved', word(1));
    await addToList('saved', word(2));
    expect(saved().items.map((i) => i.kind === 'entry' && i.headword)).toEqual(['w2', 'w1']);
    await removeFromList('saved', stableKey(word(1)));
    expect(keys('saved')).toEqual([stableKey(word(2))]);
  });
  it('keeps one item in several lists', async () => {
    const verbs = await createList('  Verbs   to   learn ');
    expect(findList(lists.get(), verbs)!.name).toBe('Verbs to learn');
    await addToList('saved', word(1));
    await addToList(verbs, word(1));
    expect(listsWith(lists.get(), stableKey(word(1))).map((l) => l.id)).toEqual(['saved', verbs]);
    expect(itemCount(lists.get())).toBe(1);
    await removeFromList(verbs, stableKey(word(1)));
    expect(inList(saved(), stableKey(word(1)))).toBe(true);
  });
  it('renames and deletes lists', async () => {
    const id = await createList('A');
    await renameList(id, 'B');
    expect(findList(lists.get(), id)!.name).toBe('B');
    await renameList(id, '   ');
    expect(findList(lists.get(), id)!.name).toBe('B');
    await deleteList(id);
    expect(findList(lists.get(), id)).toBeUndefined();
  });
  it('tap toggles the item in the last-used list', async () => {
    const id = await createList('Topik');
    await setLastUsed(id);
    expect(await toggleSave(word(1))).toMatchObject({ removed: false, list: { id } });
    expect(inList(findList(lists.get(), id), stableKey(word(1)))).toBe(true);
    expect(inList(saved(), stableKey(word(1)))).toBe(false);
    expect(await toggleSave(word(1))).toMatchObject({ removed: true, list: { id } });
    expect(findList(lists.get(), id)!.items).toHaveLength(0);
  });
  it('falls back to "Saved words" when the last-used list is gone, and makes it again if needed', async () => {
    const id = await createList('Gone');
    await setLastUsed(id);
    await deleteList(id);
    expect((await toggleSave(word(1))).list.id).toBe('saved');
    await deleteList('saved');
    expect(lists.get().lists).toHaveLength(0);
    const r = await toggleSave(word(2));
    expect(r.list.name).toBe('Saved words');
    expect(keys('saved')).toEqual([stableKey(word(2))]);
  });
  it('undo puts a removed item back where it was', async () => {
    for (const n of [1, 2, 3]) await addToList('saved', word(n));
    const it = saved().items[1];
    await removeFromList('saved', it.key);
    await restoreItem('saved', it, 1);
    expect(saved().items.map((i) => i.key)).toEqual([3, 2, 1].map((n) => stableKey(word(n))));
    await restoreItem('saved', it, 1);   // twice: no duplicate
    expect(saved().items).toHaveLength(3);
  });
});

describe('bookmarks become lists', () => {
  const old = {
    version: 1,
    folders: [{ id: 'saved', name: 'Saved' }, { id: 'fverbs', name: 'Verbs' }, { id: 'fempty', name: 'Empty' }],
    items: [
      { key: 'krdict|가다||verb', source: 'krdict', headword: '가다', homonym: null, pos: 'verb', hanja: null, gloss: 'go', folder: 'fverbs', added: 30 },
      { key: 'krdict|학교||noun', source: 'krdict', headword: '학교', homonym: null, pos: 'noun', hanja: '學校', gloss: 'school', folder: 'saved', added: 20 },
      { key: 'krdict:10', source: 'krdict', id: 10, headword: '배', hanja: null, gloss: 'pear', folder: 'saved', added: 10 },
    ],
  };
  it('puts the default folder in "Saved words" and every other folder in a list of its name', () => {
    const d = migrateBookmarks(emptyLists(), sanitizeBookmarks(old));
    expect(d.migrated).toBe(true);
    expect(d.lists.map((l) => [l.id, l.name, l.items.length])).toEqual([['saved', 'Saved words', 2], ['fverbs', 'Verbs', 1], ['fempty', 'Empty', 0]]);
    expect(d.lists[0].items.map((i) => i.key)).toEqual(['krdict|학교||noun', 'krdict|배||#10']);
    expect(d.lists[0].items[0]).toMatchObject({ kind: 'entry', hanja: '學校', gloss: 'school', added: 20 });
    expect(d.lists[1]).toMatchObject({ created: 30, updated: 30 });
  });
  it('is idempotent', () => {
    const once = migrateBookmarks(emptyLists(), sanitizeBookmarks(old));
    const twice = migrateBookmarks(once, sanitizeBookmarks(old));
    expect(twice).toEqual(once);
  });
  it('merges into lists that already exist without losing anything', () => {
    const base = emptyLists();
    base.lists[0].items.push({ kind: 'grammar', key: grammarKey('-고'), pattern: '-고', summary: null, level: null, added: 99, ref: null });
    const d = migrateBookmarks(base, sanitizeBookmarks(old));
    expect(d.lists[0].items.map((i) => i.key)).toEqual([grammarKey('-고'), 'krdict|학교||noun', 'krdict|배||#10']);
  });
  it('resolves legacy source:id items in every list once the engine is up', async () => {
    lists.set(migrateBookmarks(emptyLists(), sanitizeBookmarks(old)));
    const verbs = 'fverbs';
    await addToList(verbs, { kind: 'entry', source: 'krdict', headword: '배', homonym: 1, pos: 'noun' });
    const l = findList(lists.get(), verbs)!;
    l.items.push({ ...saved().items[1] });   // the same legacy item in a second list
    await upgradeLists(async (s, id) => (s === 'krdict' && id === 10 ? { headword: '배', homonym: 1, pos: 'noun' } : null));
    expect(keys('saved')).toEqual(['krdict|학교||noun', 'krdict|배|1|noun']);
    expect(keys(verbs)).toEqual(['krdict|배|1|noun', 'krdict|가다||verb']);   // deduped
    expect(lists.get().lists.flatMap((x) => x.items).some((i) => i.kind === 'entry' && i.legacyId != null)).toBe(false);
  });
  it('leaves legacy items alone if the engine is unavailable', async () => {
    lists.set(migrateBookmarks(emptyLists(), sanitizeBookmarks(old)));
    await upgradeLists(async () => { throw new Error('no engine'); });
    expect(keys('saved')).toContain('krdict|배||#10');
  });
});

describe('backup', () => {
  beforeEach(() => lists.set(emptyLists()));

  it('round-trips all lists, grammar items and the last-used list, and merges', async () => {
    const id = await createList('Grammar');
    await addToList(id, gram('-(으)니까'));
    await addToList('saved', word(1, { homonym: 3 }));
    await addToList(id, word(1, { homonym: 3 }));
    await setLastUsed(id);
    const json = exportBackup();
    lists.set(emptyLists());
    expect(await importBackup(json)).toEqual({ lists: 1, items: 3 });
    expect(lists.get().lists.map((l) => l.name)).toEqual(['Saved words', 'Grammar']);
    expect(findList(lists.get(), id)!.items[1]).toMatchObject({ kind: 'grammar', pattern: '-(으)니까', summary: 'because', ref: { source: 'krdict' } });
    expect(findList(lists.get(), 'saved')!.items[0]).toMatchObject({ kind: 'entry', homonym: 3 });
    expect(await importBackup(json)).toEqual({ lists: 0, items: 0 });
    // the backup is the stored data itself
    expect(sanitizeLists(JSON.parse(json)).lastUsed).toBe(id);
  });
  it('imports an old bookmarks-only backup into "Saved words" (and folder lists)', async () => {
    const oldBackup = JSON.stringify({
      app: 'korean-dictionary', exportedAt: '2026-01-01T00:00:00Z', version: 1,
      folders: [{ id: 'saved', name: 'Saved' }, { id: 'fx', name: 'Food' }],
      items: [
        { key: 'krdict|밥||noun', source: 'krdict', headword: '밥', homonym: null, pos: 'noun', hanja: null, gloss: 'rice', folder: 'saved', added: 1 },
        { key: 'krdict|김치||noun', source: 'krdict', headword: '김치', homonym: null, pos: 'noun', hanja: null, gloss: 'kimchi', folder: 'fx', added: 2 },
      ],
    });
    await addToList('saved', word(9));
    expect(await importBackup(oldBackup)).toEqual({ lists: 1, items: 2 });
    expect(keys('saved')).toEqual([stableKey(word(9)), 'krdict|밥||noun']);
    expect(findList(lists.get(), 'fx')).toMatchObject({ name: 'Food' });
    expect(await importBackup(oldBackup)).toEqual({ lists: 0, items: 0 });
  });
  it('rejects bad files', async () => {
    await expect(importBackup('not json')).rejects.toThrow();
    await expect(importBackup(' '.repeat(MAX_IMPORT_BYTES + 1))).rejects.toThrow(/too large/);
    const many = (n: number) => Array.from({ length: n }, (_, i) => ({ source: 'k', headword: 'w' + i, pos: null }));
    await expect(importBackup(JSON.stringify({ items: many(MAX_IMPORT_ITEMS + 1) }))).rejects.toThrow(/too many/);
    await expect(importBackup(JSON.stringify({ lists: [{ id: 'a', name: 'A', items: many(MAX_IMPORT_ITEMS + 1) }] }))).rejects.toThrow(/too many/);
    expect((await importBackup(JSON.stringify({ lists: [{ id: 'a', name: 'A', items: many(1000) }] }))).items).toBe(1000);
  });
  it('sanitises garbage', () => {
    const d = sanitizeLists({
      lists: [{ id: 'x' }, { id: 'ok', name: 'Ok', items: [{ kind: 'grammar' }, { kind: 'grammar', pattern: '-고' }, { source: 'a', headword: 'h', pos: null }, { bad: 1 }, null] },
        { id: 'ok', name: 'dup' }, { id: 'n', name: 'x'.repeat(500), items: 'nope' }],
      lastUsed: 'missing',
    });
    expect(d.lists.map((l) => l.id)).toEqual(['ok', 'n']);
    expect(d.lists[0].items.map((i) => i.key)).toEqual([grammarKey('-고'), 'a|h||']);
    expect(d.lists[1].name).toHaveLength(80);
    expect(d.lastUsed).toBe('saved');
    expect(d.migrated).toBe(false);
  });
});

describe('legacy history', () => {
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
