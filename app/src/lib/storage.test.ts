import { describe, expect, it } from 'vitest';
import { bytesNeeded, freeSpaceProblem, isFatalInstallMessage, isQuotaError, noSpaceMessage } from './storage';

const MB = 1048576;
describe('free space check', () => {
  const packs = [{ gz_bytes: 30 * MB, bytes: 120 * MB }];
  it('needs (gz + bytes) * 1.1', () => expect(bytesNeeded(packs)).toBeCloseTo(165 * MB));
  it('subtracts already staged bytes', () => expect(bytesNeeded(packs, 30 * MB)).toBeCloseTo(132 * MB));
  it('flags when free < need', () => {
    expect(freeSpaceProblem({ quota: 200 * MB, usage: 50 * MB }, bytesNeeded(packs))).toBe('Not enough free storage: needs about 165 MB free');
  });
  it('passes when enough, or when quota unknown', () => {
    expect(freeSpaceProblem({ quota: 400 * MB, usage: 50 * MB }, bytesNeeded(packs))).toBeNull();
    expect(freeSpaceProblem({}, bytesNeeded(packs))).toBeNull();
    expect(freeSpaceProblem(undefined, 1e12)).toBeNull();
  });
  it('rounds MB up', () => expect(noSpaceMessage(1.2 * MB)).toBe('Not enough free storage: needs about 2 MB free'));
  it('classifies quota errors', () => {
    expect(isQuotaError(Object.assign(new Error('x'), { name: 'QuotaExceededError' }))).toBe(true);
    expect(isQuotaError(new Error('database or disk is full'))).toBe(true);
    expect(isQuotaError('SQLITE_FULL')).toBe(true);
    expect(isQuotaError(new Error('Download failed (404)'))).toBe(false);
    expect(isFatalInstallMessage(noSpaceMessage(MB))).toBe(true);
  });
});
