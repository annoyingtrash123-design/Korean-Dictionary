import { describe, expect, it } from 'vitest';
import { stalePacks } from './app-state';

const m = { version: '20261101', packs: [{ id: 'core' }, { id: 'stdict' }, { id: 'opendict' }] };
describe('stalePacks', () => {
  it('compares every installed pack, not just core', () => {
    const packs = { core: { installed: true, version: '20261101' }, stdict: { installed: true, version: '20261001' } };
    expect(stalePacks(packs, m)).toEqual(['stdict']);
  });
  it('lists core first and ignores uninstalled / unknown packs', () => {
    const packs = { stdict: { installed: true, version: 'old' }, core: { installed: true, version: 'old' }, opendict: { installed: false }, gone: { installed: true, version: 'x' } };
    expect(stalePacks(packs, m)).toEqual(['core', 'stdict']);
  });
  it('is empty when current or offline', () => {
    expect(stalePacks({ core: { installed: true, version: '20261101' } }, m)).toEqual([]);
    expect(stalePacks({ core: { installed: true, version: 'a' } }, null)).toEqual([]);
  });
});
