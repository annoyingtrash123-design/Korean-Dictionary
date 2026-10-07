import { describe, expect, it } from 'vitest';
import { migrateSettings } from './settings';
import { activePacks, optionalPacks } from './packs';

describe('settings migration & packs', () => {
  it('migrates stdict boolean', () => {
    expect(migrateSettings({ stdict: false }).packs).toEqual({ stdict: false, opendict: false });
    expect(migrateSettings({}).packs).toEqual({ stdict: true, opendict: false });
    expect(migrateSettings({ packs: { opendict: true } }).packs).toEqual({ stdict: true, opendict: true });
    expect('stdict' in migrateSettings({ stdict: true })).toBe(false);
  });
  it('active packs = core + enabled installed', () => {
    const st = { ready: true, packs: { core: { id: 'core', installed: true }, stdict: { id: 'stdict', installed: true }, opendict: { id: 'opendict', installed: true } } };
    expect(activePacks({ packs: { stdict: true, opendict: false } }, st)).toEqual(['core', 'stdict']);
    expect(activePacks({ packs: {} }, st)).toEqual(['core']);
  });
  it('lists optional packs from a manifest with an unknown id', () => {
    const m = { version: '1', packs: [{ id: 'core', required: true }, { id: 'opendict', required: false }, { id: 'weird', required: false }] } as any;
    expect(optionalPacks(m).map((p) => p.id)).toEqual(['opendict', 'weird']);
  });
});
