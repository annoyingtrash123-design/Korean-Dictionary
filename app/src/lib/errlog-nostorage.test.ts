import { describe, expect, it, vi } from 'vitest';

// Every IndexedDB call fails: the logger must keep working in memory and never throw.
vi.mock('idb-keyval', () => ({
  get: () => Promise.reject(new Error('storage unavailable')),
  set: () => Promise.reject(new Error('storage unavailable')),
  del: () => Promise.reject(new Error('storage unavailable')),
  createStore: () => { throw new Error('storage unavailable'); },
}));

import { clearErrors, errors$, loadErrors, logError } from './errlog';

describe('errlog without storage', () => {
  it('never throws and keeps the log in memory', async () => {
    expect(() => logError('t', new Error('x'))).not.toThrow();
    logError('t', 'y');
    await new Promise((r) => setTimeout(r, 20));
    await expect(loadErrors()).resolves.toHaveLength(2);
    expect(errors$.get().map((e) => e.msg)).toEqual(['Error: x', 'y']);
    await expect(clearErrors()).resolves.toBeUndefined();
    expect(errors$.get()).toEqual([]);
  });
});
