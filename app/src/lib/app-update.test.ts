import { describe, expect, it } from 'vitest';
import { checkAppUpdate, isNewerVersion, parseReleaseVersion } from './app-update';

describe('app update', () => {
  it('parses the version line from the release body', () => {
    expect(parseReleaseVersion('Hanppuri\nversion: 2026.11.13-42\nnotes')).toBe('2026.11.13-42');
    expect(parseReleaseVersion('  Version:  1.2.3  ')).toBe('1.2.3');
    expect(parseReleaseVersion('no version here')).toBeNull();
    expect(parseReleaseVersion(null)).toBeNull();
  });
  it('compares versions', () => {
    expect(isNewerVersion('2026.11.13-43', '2026.11.13-42')).toBe(true);
    expect(isNewerVersion('2026.11.13-42', '2026.11.13-42')).toBe(false);
    expect(isNewerVersion(null, '1')).toBe(false);
    expect(isNewerVersion('2026.11.13-42', 'dev')).toBe(false);
  });
  it('is a no-op on the web', async () => {
    expect(await checkAppUpdate()).toBeNull();
  });
});
