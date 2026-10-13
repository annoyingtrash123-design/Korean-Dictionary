import { describe, expect, it } from 'vitest';
import { installLive } from './install-live';
import type { Manifest } from './types';

const man = (version: string) => ({ version, packs: [{ id: 'core' }, { id: 'texts' }] }) as unknown as Manifest;

describe('installLive', () => {
  it('uses the live manifest, not the one the screen was opened with', async () => {
    const got: string[] = [];
    await installLive(man('old'), ['core'], async () => man('new'), async (p, v) => { got.push(`${p.id}@${v}`); });
    expect(got).toEqual(['core@new']);
  });
  it('restarts once with the new manifest when a deploy lands mid-download', async () => {
    const versions = ['v1', 'v2'];
    const got: string[] = [];
    const m = await installLive(man('v1'), ['core'], async () => man(versions.shift() ?? 'v2'), async (p, v) => {
      got.push(v);
      if (v === 'v1') throw new Error('Downloaded data is corrupt (checksum mismatch) — please retry');
    });
    expect(got).toEqual(['v1', 'v2']);
    expect(m.version).toBe('v2');
  });
  it('rethrows when the version did not change (real corruption, no space, offline)', async () => {
    await expect(installLive(man('v1'), ['core'], async () => man('v1'), async () => { throw new Error('boom'); })).rejects.toThrow('boom');
    await expect(installLive(man('v1'), ['core'], async () => { throw new Error('offline'); }, async () => { throw new Error('net'); })).rejects.toThrow('net');
  });
});
