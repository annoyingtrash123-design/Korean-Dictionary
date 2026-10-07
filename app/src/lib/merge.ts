import { SOURCE_ORDER, type MatchKind, type ResultRow, type Source } from './types';
import { normHeadword } from './search-mode';

export interface ResultGroup {
  key: string;
  headword: string;
  hanja: string | null;
  rows: ResultRow[];       // all entries of this headword+hanja, in engine order
  primary: ResultRow;      // row used for navigation
  level: number | null;
  pos: string[];
  gloss: string;
  sources: Source[];
  via: MatchKind;
}

const srcIdx = (s: Source) => SOURCE_ORDER.indexOf(s);
const TIER: Record<MatchKind, number> = { exact: 0, hanja: 1, form: 2, deconj: 3, prefix: 4, fts: 5 };

/** Dedupe by source:id across packs, keeping the best match tier. Order of first appearance is kept. */
export function mergeRows(...lists: ResultRow[][]): ResultRow[] {
  const best = new Map<string, ResultRow>();
  for (const r of lists.flat()) {
    const k = `${r.source}:${r.id}`;
    const cur = best.get(k);
    if (!cur) best.set(k, r);
    else if (TIER[r.via ?? 'prefix'] < TIER[cur.via ?? 'prefix']) best.set(k, { ...r });
  }
  return [...best.values()];
}

/** One group per headword+hanja; group order follows the engine's row order (already ranked). */
export function groupResults(rows: ResultRow[]): ResultGroup[] {
  const groups = new Map<string, ResultGroup>();
  for (const r of rows) {
    const key = `${normHeadword(r.headword)}|${r.hanja ?? ''}`;
    let g = groups.get(key);
    if (!g) {
      g = { key, headword: r.headword, hanja: r.hanja, rows: [], primary: r, level: null, pos: [], gloss: '', sources: [], via: r.via ?? 'prefix' };
      groups.set(key, g);
    }
    g.rows.push(r);
  }
  for (const g of groups.values()) {
    // Primary: an English-defined row (carries gloss + level) by source order; else first row.
    const en = g.rows.filter((r) => r.source !== 'stdict');
    const pool = en.length ? en : g.rows;
    g.primary = [...pool].sort((a, b) => srcIdx(a.source) - srcIdx(b.source))[0];
    g.headword = g.primary.headword;
    g.level = g.rows.map((r) => r.level).find((l) => l != null) ?? null;
    g.pos = [...new Set(g.rows.map((r) => r.pos).filter((p): p is string => !!p))].slice(0, 2);
    g.gloss = (pool.find((r) => r.gloss)?.gloss ?? g.rows.find((r) => r.gloss)?.gloss ?? '').trim();
    g.sources = [...new Set(g.rows.map((r) => r.source))].sort((a, b) => srcIdx(a) - srcIdx(b));
    g.via = g.rows.reduce<MatchKind>((m, r) => (TIER[r.via ?? 'prefix'] < TIER[m] ? (r.via ?? 'prefix') : m), g.rows[0].via ?? 'prefix');
  }
  return [...groups.values()];
}

/** Entries shown on the entry page: same headword + same hanja as the primary, ordered by source. */
export function sameWordRows<T extends { headword: string; hanja: string | null; source: Source; homonym: number | null; id: number }>(
  all: T[], primary: { headword: string; hanja: string | null },
): T[] {
  const k = normHeadword(primary.headword);
  return all
    .filter((e) => normHeadword(e.headword) === k && (e.hanja ?? '') === (primary.hanja ?? ''))
    .sort((a, b) => srcIdx(a.source) - srcIdx(b.source) || (a.homonym ?? 0) - (b.homonym ?? 0) || a.id - b.id);
}
