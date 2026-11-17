/** Day number for the home page's daily picks: whole local calendar days since 2026-01-01 (never negative).
 *  Built from the local year/month/day, so it flips at the user's midnight and is the same for everyone on that date. */
export const HOME_EPOCH = Date.UTC(2026, 0, 1);
const DAY_MS = 86_400_000;
export function dayNumber(d: Date = new Date()): number {
  const local = Date.UTC(d.getFullYear(), d.getMonth(), d.getDate());
  return Math.max(0, Math.round((local - HOME_EPOCH) / DAY_MS));
}

/** "5급 · 표준 교육과정": the official curriculum level, never plain "TOPIK 5". */
export const curriculumLabel = (level: number) => `${level}급 · 표준 교육과정`;
export const curriculumHint = (level: number) => `National standard Korean curriculum level ${level} (≈ TOPIK ${level})`;
