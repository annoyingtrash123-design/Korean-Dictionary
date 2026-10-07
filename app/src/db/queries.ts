import { tempDeconj as deconjugate, tempHints as grammarHints } from './temp-deconj';
import { detectMode, ftsQuery, hanChars, normHeadword, stemOf } from '../lib/search-mode';
import type {
  Entry, EntryRow, GrammarRow, HanjaChar, MatchKind, ResultRow, SearchResult, Sentence, Source,
} from './types';

/** The subset of sqlite-wasm's Database we use (keeps this file testable). */
export interface Db { selectObjects(sql: string, bind?: unknown): Record<string, unknown>[] }
export type Dbs = Record<string, Db | undefined>;

const COLS = 'e.id, e.source, e.headword, e.hw_norm, e.homonym, e.hanja, e.pos, e.pron, e.lang, e.level, e.rank, e.kind, e.gloss';
const packOf = (source: string) => (source === 'stdict' ? 'stdict' : 'core');
const toRow = (o: Record<string, unknown>, via: MatchKind, extra: Partial<ResultRow> = {}): ResultRow => {
  const e = o as unknown as EntryRow;
  return { source: e.source, id: e.id, headword: e.headword, hanja: e.hanja, pos: e.pos, level: e.level, gloss: e.gloss, kind: e.kind,
    pack: packOf(e.source), via, rank: e.rank, homonym: e.homonym, pron: e.pron, lang: e.lang, hw_norm: e.hw_norm, ...extra };
};
const TIER: Record<MatchKind, number> = { exact: 0, hanja: 1, form: 2, deconj: 3, prefix: 4, fts: 5 };
const order = (a: ResultRow, b: ResultRow) =>
  TIER[a.via!] - TIER[b.via!] || (a.score ?? 0) - (b.score ?? 0) || (a.rank ?? 0) - (b.rank ?? 0);

function active(dbs: Dbs, packs: string[]): Db[] {
  return packs.map((p) => dbs[p]).filter((d): d is Db => !!d);
}
const packFor = (dbs: Dbs, source: string): Db | undefined => dbs[packOf(source)];

function hangulSearch(dbs: Db[], q: string, limit: number): { rows: ResultRow[]; hints: string[]; deconj: { lemma: string; rule: string }[] } {
  const norm = normHeadword(q);
  const rows: ResultRow[] = [];
  const hints = grammarHints(q);
  const cands = deconjugate(q).filter((c) => c.lemma && normHeadword(c.lemma) !== norm);
  for (const db of dbs) {
    for (const o of db.selectObjects(`SELECT ${COLS} FROM entries e WHERE e.hw_norm = ? ORDER BY e.rank LIMIT 100`, [norm])) rows.push(toRow(o, 'exact'));
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM forms f JOIN entries e ON e.id = f.entry_id WHERE f.form = ? ORDER BY e.rank LIMIT 50`, [norm])) rows.push(toRow(o, 'form'));
    const seen = new Set<string>();
    for (const c of cands) {
      const lemma = normHeadword(c.lemma);
      if (seen.has(lemma)) continue;
      seen.add(lemma);
      for (const o of db.selectObjects(`SELECT ${COLS} FROM entries e WHERE e.hw_norm = ? ORDER BY e.rank LIMIT 20`, [lemma]))
        rows.push(toRow(o, 'deconj'));
    }
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM entries e WHERE e.hw_norm >= ?1 AND e.hw_norm < ?2 AND e.hw_norm <> ?3 ORDER BY e.rank LIMIT ?4`,
      [norm, norm + '￿', norm, limit])) rows.push(toRow(o, 'prefix'));
  }
  return { rows, hints, deconj: cands };
}

function latinSearch(core: Db | undefined, q: string, limit: number): ResultRow[] {
  const m = ftsQuery(q);
  if (!core || !m) return [];
  // Over-fetch by bm25, then bucket the score so `rank` can break near-ties.
  const rows = core.selectObjects(
    `SELECT ${COLS}, bm25(entries_fts) AS score FROM entries_fts JOIN entries e ON e.id = entries_fts.rowid
     WHERE entries_fts MATCH ? ORDER BY bm25(entries_fts) LIMIT ?`, [m, limit * 3]);
  return rows.map((o) => toRow(o, 'fts', { score: Math.round(Number(o.score) * 2) / 2 })).sort((a, b) => a.score! - b.score! || (a.rank ?? 0) - (b.rank ?? 0)).slice(0, limit * 2);
}

function hanSearch(dbs: Db[], core: Db | undefined, q: string, limit: number): { rows: ResultRow[]; hanja: HanjaChar[] } {
  const chars = [...new Set(hanChars(q))];
  const hanja = core && chars.length
    ? (core.selectObjects(`SELECT * FROM hanja_chars WHERE ch IN (${chars.map(() => '?').join(',')})`, chars) as unknown as HanjaChar[])
    : [];
  hanja.sort((a, b) => chars.indexOf(a.ch) - chars.indexOf(b.ch));
  const rows: ResultRow[] = [];
  const ph = chars.map(() => '?').join(',');
  for (const db of dbs) {
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM entries e WHERE e.id IN (SELECT entry_id FROM hanja_words WHERE ch IN (${ph}) GROUP BY entry_id HAVING COUNT(DISTINCT ch) = ?)
       ORDER BY e.rank LIMIT ?`, [...chars, chars.length, limit])) rows.push(toRow(o, 'hanja'));
  }
  return { rows, hanja };
}

export function search(dbs: Dbs, query: string, opts: { packs: string[]; limit?: number }): SearchResult {
  const q = query.trim();
  const mode = detectMode(q);
  const limit = opts.limit ?? 50;
  const act = active(dbs, opts.packs);
  if (mode === 'empty' || !act.length) return { mode: 'english', rows: [] };
  if (mode === 'hangul') {
    const r = hangulSearch(act, q, limit);
    return { mode: 'hangul', rows: r.rows.sort(order), grammarHints: r.hints, deconj: r.deconj };
  }
  if (mode === 'han') { const r = hanSearch(act, dbs.core, q, limit); return { mode: 'hanja', rows: r.rows.sort(order), hanja: r.hanja }; }
  return { mode: 'english', rows: latinSearch(dbs.core, q, limit) };
}

export function getEntriesByHeadword(dbs: Dbs, hw: string, packs: string[]): Entry[] {
  const norm = normHeadword(hw);
  return active(dbs, packs).flatMap((db) =>
    db.selectObjects(`SELECT ${COLS}, e.data FROM entries e WHERE e.hw_norm = ? ORDER BY e.rank`, [norm]).map(parseEntry));
}

function parseEntry(o: Record<string, unknown>): Entry {
  let data = { senses: [] } as Entry['data'];
  try { data = JSON.parse(String(o.data)); } catch { /* keep empty */ }
  return { ...(o as unknown as EntryRow), data };
}

export function getEntry(dbs: Dbs, source: string, id: number): Entry | null {
  const db = packFor(dbs, source);
  if (!db) return null;
  const o = db.selectObjects(`SELECT ${COLS}, e.data FROM entries e WHERE e.id = ?`, [id])[0];
  if (!o) return null;
  return parseEntry(o);
}

export function hanjaChar(dbs: Dbs, ch: string): HanjaChar | null {
  return (dbs.core?.selectObjects('SELECT * FROM hanja_chars WHERE ch = ?', [ch])[0] as unknown as HanjaChar) ?? null;
}

export function wordsWithHanja(dbs: Dbs, ch: string, limit: number, offset: number, packs: string[]): ResultRow[] {
  const rows: ResultRow[] = [];
  for (const db of active(dbs, packs))
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM entries e WHERE e.id IN (SELECT entry_id FROM hanja_words WHERE ch = ?) ORDER BY e.rank LIMIT ? OFFSET ?`,
      [ch, limit + offset, 0])) rows.push(toRow(o, 'hanja'));
  rows.sort((a, b) => (a.rank ?? 0) - (b.rank ?? 0));
  return rows.slice(offset, offset + limit);
}

export function sentences(dbs: Dbs, query: string, limit: number): Sentence[] {
  const core = dbs.core;
  const stem = stemOf(query).replace(/["%_]/g, '');
  if (!core || !stem) return [];
  if ([...stem].length >= 3) {
    try {
      return core.selectObjects(
        `SELECT s.id, s.ko, s.en, s.source FROM sentences_fts f JOIN sentences s ON s.id = f.rowid
         WHERE sentences_fts MATCH ? ORDER BY length(s.ko) LIMIT ?`, [`"${stem}"`, limit]) as unknown as Sentence[];
    } catch { /* fall through to LIKE */ }
  }
  return core.selectObjects('SELECT id, ko, en, source FROM sentences WHERE ko LIKE ? ORDER BY length(ko) LIMIT ?', [`%${stem}%`, limit]) as unknown as Sentence[];
}

export function grammarList(dbs: Dbs): GrammarRow[] {
  return (dbs.core?.selectObjects('SELECT id, entry_id, pattern, category, level, summary_en, sort FROM grammar ORDER BY sort, id') ?? []) as unknown as GrammarRow[];
}

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); }
  return h >>> 0;
}
/** Deterministic pick (by date) among krdict level 1–2 words; falls back to any core word. */
export function wordOfDay(dbs: Dbs, dateStr: string): ResultRow | null {
  const core = dbs.core;
  if (!core) return null;
  for (const where of ["e.source = 'krdict' AND e.level IN (1,2) AND e.kind = 'word'", "e.kind = 'word'"]) {
    const n = Number(core.selectObjects(`SELECT COUNT(*) AS n FROM entries e WHERE ${where}`)[0]?.n ?? 0);
    if (!n) continue;
    const o = core.selectObjects(`SELECT ${COLS} FROM entries e WHERE ${where} ORDER BY e.id LIMIT 1 OFFSET ?`, [hash(dateStr) % n])[0];
    if (o) return toRow(o, 'exact');
  }
  return null;
}
