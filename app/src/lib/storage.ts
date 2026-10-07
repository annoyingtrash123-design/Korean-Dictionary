// Free-space arithmetic and error classification for pack installs (pure, shared by UI and worker).
export const SPACE_MARGIN = 1.1;
export const NO_SPACE_PREFIX = 'Not enough free storage';
export const CORRUPT_MSG = 'Downloaded data is corrupt (checksum mismatch) — please retry';

export const fmtMB = (n: number) => `${Math.max(1, Math.ceil(n / 1048576))} MB`;

/** Bytes needed during an install: compressed download + decompressed DB, with 10% headroom. */
export const bytesNeeded = (packs: { gz_bytes: number; bytes: number }[], alreadyStaged = 0) =>
  Math.max(0, packs.reduce((a, p) => a + p.gz_bytes + p.bytes, 0) - alreadyStaged) * SPACE_MARGIN;

export const noSpaceMessage = (need: number) => `${NO_SPACE_PREFIX}: needs about ${fmtMB(need)} free`;

/** Returns the user-facing message if (quota - usage) < need, else null. Unknown quota => null. */
export function freeSpaceProblem(est: { quota?: number; usage?: number } | undefined, need: number): string | null {
  if (!est || !est.quota) return null;
  return est.quota - (est.usage ?? 0) < need ? noSpaceMessage(need) : null;
}

/** QuotaExceededError / SQLite "database or disk is full" / VFS full errors. */
export function isQuotaError(e: unknown): boolean {
  const x = e as { name?: string; message?: string } | string | null | undefined;
  const name = typeof x === 'object' && x ? x.name ?? '' : '';
  const msg = typeof x === 'string' ? x : x?.message ?? String(x ?? '');
  return name === 'QuotaExceededError' || /quota|SQLITE_FULL|disk is full|database or disk|\bvfs\b.*\bfull\b|no space/i.test(msg);
}
/** Errors for which "resume" would not help (the user must free space / retry from scratch). */
export const isFatalInstallMessage = (m: string) => m.startsWith(NO_SPACE_PREFIX) || m.startsWith('Downloaded data is corrupt');
