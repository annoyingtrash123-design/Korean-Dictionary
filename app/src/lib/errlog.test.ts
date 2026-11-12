import { beforeEach, describe, expect, it } from 'vitest';
import { get } from 'idb-keyval';
import { userDb } from './idb';
import { bookmarks, emptyData, exportBookmarks, importBookmarks, addBookmark } from './bookmarks';
import { clearErrors, errors$, formatErrors, logError, makeEntry, MAX_ENTRIES, sanitizeEntries } from './errlog';

const flush = () => new Promise((r) => setTimeout(r, 20));

describe('errlog', () => {
  beforeEach(async () => { await clearErrors(); });

  it('records message, source and a short stack', () => {
    logError('test', new Error('boom'));
    const [e] = errors$.get();
    expect(e.src).toBe('test');
    expect(e.msg).toBe('Error: boom');
    expect((e.stack ?? '').split('\n').length).toBeLessThanOrEqual(4);
  });
  it('accepts strings and odd values', () => {
    logError('a', 'plain'); logError('b', { message: 'obj' }); logError('c', undefined); logError('d', null);
    expect(errors$.get().map((e) => e.msg).slice(0, 2)).toEqual(['plain', 'obj']);
    expect(errors$.get()).toHaveLength(4);
  });
  it('keeps only the newest MAX_ENTRIES', () => {
    for (let i = 0; i < MAX_ENTRIES + 25; i++) logError('t', 'm' + i);
    const l = errors$.get();
    expect(l).toHaveLength(MAX_ENTRIES);
    expect(l[0].msg).toBe('m25');
    expect(l[l.length - 1].msg).toBe('m' + (MAX_ENTRIES + 24));
  });
  it('persists to IndexedDB, and Clear removes it', async () => {
    logError('t', 'saved');
    await flush();
    expect(sanitizeEntries(await get('errlog', userDb())).map((e) => e.msg)).toEqual(['saved']);
    await clearErrors();
    expect(await get('errlog', userDb())).toBeUndefined();
    expect(errors$.get()).toEqual([]);
  });
  it('ignores corrupt stored data', () => {
    expect(sanitizeEntries('nope')).toEqual([]);
    expect(sanitizeEntries([{ t: 'x' }, null, 5, { t: 1, src: 's', msg: 'ok' }])).toEqual([{ t: 1, src: 's', msg: 'ok' }]);
  });
  it('formats newest first as plain text', () => {
    const text = formatErrors([makeEntry('a', 'first', 1000), makeEntry('b', 'second', 2000)]);
    expect(text.indexOf('second')).toBeLessThan(text.indexOf('first'));
    expect(text).toContain('[b] second');
  });
});

describe('backup round trip with the error log', () => {
  beforeEach(() => bookmarks.set(emptyData()));

  it('includes the log in the export and imports fine with or without it', async () => {
    await addBookmark({ source: 'krdict', headword: '학교', homonym: null, pos: 'noun', hanja: null, gloss: 'school' });
    const errs = [makeEntry('t', 'boom', 5)];
    const withLog = exportBookmarks(undefined, errs);
    expect(JSON.parse(withLog).errors).toEqual(errs);
    expect(JSON.parse(exportBookmarks())).not.toHaveProperty('errors');

    for (const json of [withLog, exportBookmarks()]) {
      bookmarks.set(emptyData());
      const r = await importBookmarks(json);
      expect(r.items).toBe(1);
      expect(bookmarks.get().items[0].headword).toBe('학교');
    }
  });
});
