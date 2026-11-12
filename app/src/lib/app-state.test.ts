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
  it('compares each pack by its own content version when the manifest has one', () => {
    const mv = { version: '20261201', packs: [{ id: 'core', version: 'c2' }, { id: 'stdict', version: 's1' }, { id: 'opendict', version: 'o1' }] };
    // a new release that changed only core: stdict/opendict stay current
    const packs = { core: { installed: true, version: 'c1' }, stdict: { installed: true, version: 's1' }, opendict: { installed: true, version: 'o1' } };
    expect(stalePacks(packs, mv)).toEqual(['core']);
    // an install from before per-pack versions (it holds the old build version) updates once
    expect(stalePacks({ stdict: { installed: true, version: '20261101' } }, mv)).toEqual(['stdict']);
  });
  it('is empty when current or offline', () => {
    expect(stalePacks({ core: { installed: true, version: '20261101' } }, m)).toEqual([]);
    expect(stalePacks({ core: { installed: true, version: 'a' } }, null)).toEqual([]);
  });
});
