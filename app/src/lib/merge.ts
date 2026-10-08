import { SOURCE_ORDER, isKoSource, type MatchKind, type ResultRow, type Source } from './types';
import { normHeadword } from './search-mode';

export interface ResultGroup {
  key: string;
  headword: string;
  hanja?: string;
  rows: ResultRow[];       // all entries of this headword+hanja, in engine order
  primary: ResultRow;      // row used for navigation
  level?: number;
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
      g = { key, headword: r.headword, hanja: r.hanja, rows: [], primary: r, level: undefined, pos: [], gloss: '', sources: [], via: r.via ?? 'prefix' };
      groups.set(key, g);
    }
    g.rows.push(r);
  }
  // A row without hanja (e.g. Wiktionary) is the same word as a hanja-tagged group with the same
  // headword: fold it in instead of listing the word twice. With several homographs
  // (報告 / 寶庫), it joins the one whose glosses share the most words with it.
  const words = (s?: string) => new Set((s ?? '').toLowerCase().split(/[^a-z]+/).filter((w) => w.length > 2 && w !== 'the'));
  for (const [key, g] of [...groups]) {
    if (g.hanja) continue;
    const hw = normHeadword(g.headword);
    const homes = [...groups.values()].filter((o) => o !== g && o.hanja && normHeadword(o.headword) === hw);
    if (!homes.length) continue;
    const homeWords = new Map(homes.map((o) => [o, words(o.rows.map((r) => r.gloss).join(' '))]));
    const keep: ResultRow[] = [];
    let best: ResultGroup | undefined;   // the home that received the earliest row
    for (const r of g.rows) {
      const mine = words(r.gloss);
      const scored = homes.map((o) => ({ o, n: [...mine].filter((x) => homeWords.get(o)!.has(x)).length })).sort((x, y) => y.n - x.n);
      // a single homograph takes any row; several need a clear gloss match
      if (homes.length > 1 && (scored[0].n === 0 || scored[0].n === scored[1].n)) { keep.push(r); continue; }
      scored[0].o.rows.push(r);
      best ??= scored[0].o;
    }
    g.rows = keep;
    // the merged word keeps the better (earlier) position in the list
    const order = [...groups.keys()];
    if (best && order.indexOf(key) < order.indexOf(best.key)) {
      const rebuilt = new Map<string, ResultGroup>();
      for (const k of order) {
        if (k === key) { rebuilt.set(best.key, best); if (keep.length) rebuilt.set(key, g); }
        else if (k !== best.key) rebuilt.set(k, groups.get(k)!);
      }
      groups.clear(); for (const [k, v] of rebuilt) groups.set(k, v);
    } else if (!keep.length) groups.delete(key);
  }
  for (const g of groups.values()) {
    // Primary: an English-defined row (carries gloss + level) by source order; else first row.
    const en = g.rows.filter((r) => r.lang !== 'ko' && !isKoSource(r.source));
    const pool = en.length ? en : g.rows;
    g.primary = [...pool].sort((a, b) => srcIdx(a.source) - srcIdx(b.source))[0];
    g.headword = g.primary.headword;
    g.level = g.rows.map((r) => r.level).find((l) => l != null);
    g.pos = [...new Set(g.rows.map((r) => r.pos).filter((p): p is string => !!p))].slice(0, 2);
    g.gloss = (g.primary.gloss ?? pool.find((r) => r.gloss)?.gloss ?? g.rows.find((r) => r.gloss)?.gloss ?? '').trim();
    g.sources = [...new Set(g.rows.map((r) => r.source))].sort((a, b) => srcIdx(a) - srcIdx(b));
    g.via = g.rows.reduce<MatchKind>((m, r) => (TIER[r.via ?? 'prefix'] < TIER[m] ? (r.via ?? 'prefix') : m), g.rows[0].via ?? 'prefix');
  }
  return [...groups.values()];
}

/** Entries shown on the entry page: same headword + same hanja as the primary, ordered by source. */
export function sameWordRows<T extends { headword: string; hanja?: string | null; source: Source; homonym?: number | null; id: number; rank?: number }>(
  all: T[], primary: { headword: string; hanja?: string | null },
): T[] {
  const k = normHeadword(primary.headword);
  const hanjas = new Set(all.filter((e) => normHeadword(e.headword) === k && e.hanja).map((e) => e.hanja));
  const unique = !!primary.hanja && hanjas.size === 1;
  return all
    // rows without hanja (e.g. Wiktionary) belong with the hanja-tagged word when it is the only one
    .filter((e) => normHeadword(e.headword) === k && ((e.hanja ?? '') === (primary.hanja ?? '') || (!e.hanja && unique)))
    .sort((a, b) => srcIdx(a.source) - srcIdx(b.source) || (a.rank ?? 0) - (b.rank ?? 0) || (a.homonym ?? 0) - (b.homonym ?? 0) || a.id - b.id);
}
