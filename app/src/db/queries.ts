import { deconjugate, grammarHints } from '../lib/deconjugate';
import { detectMode, ftsQuery, hanChars, normHeadword, stemOf } from '../lib/search-mode';
import type {
  Entry, EntryRow, GrammarRow, HanjaChar, MatchKind, ResultRow, SearchOpts, SearchResponse, Sentence, Source,
} from '../lib/types';

/** The subset of sqlite-wasm's Database we use (keeps this file testable). */
export interface Db { selectObjects(sql: string, bind?: unknown): Record<string, unknown>[] }
export interface Dbs { core?: Db; stdict?: Db }

const COLS = 'e.id, e.source, e.headword, e.hw_norm, e.homonym, e.hanja, e.pos, e.pron, e.lang, e.level, e.rank, e.kind, e.gloss';
const toRow = (o: Record<string, unknown>, match: MatchKind, extra: Partial<ResultRow> = {}): ResultRow =>
  ({ ...(o as unknown as EntryRow), match, ...extra });

function active(dbs: Dbs, stdict: boolean): Db[] {
  return [dbs.core, stdict ? dbs.stdict : undefined].filter((d): d is Db => !!d);
}
const packFor = (dbs: Dbs, source: Source): Db | undefined => (source === 'stdict' ? dbs.stdict : dbs.core);

function hangulSearch(dbs: Db[], q: string, limit: number): { rows: ResultRow[]; hints: string[] } {
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
        rows.push(toRow(o, 'deconj', { via: c.rule, lemma: c.lemma }));
    }
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM entries e WHERE e.hw_norm >= ?1 AND e.hw_norm < ?2 AND e.hw_norm <> ?3 ORDER BY e.rank LIMIT ?4`,
      [norm, norm + '￿', norm, limit])) rows.push(toRow(o, 'prefix'));
  }
  return { rows, hints };
}

function latinSearch(core: Db | undefined, q: string, limit: number): ResultRow[] {
  const m = ftsQuery(q);
  if (!core || !m) return [];
  // Over-fetch by bm25, then bucket the score so `rank` can break near-ties.
  const rows = core.selectObjects(
    `SELECT ${COLS}, bm25(entries_fts) AS score FROM entries_fts JOIN entries e ON e.id = entries_fts.rowid
     WHERE entries_fts MATCH ? ORDER BY bm25(entries_fts) LIMIT ?`, [m, limit * 3]);
  return rows.map((o) => toRow(o, 'fts', { score: Math.round(Number(o.score) * 2) / 2 })).sort((a, b) => a.score! - b.score! || a.rank - b.rank).slice(0, limit * 2);
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

export function search(dbs: Dbs, query: string, opts: SearchOpts): SearchResponse {
  const q = query.trim();
  const mode = detectMode(q);
  const limit = opts.limit ?? 50;
  const act = active(dbs, opts.stdict);
  const base: SearchResponse = { query: q, mode, rows: [], hanja: [], hints: [] };
  if (mode === 'empty' || !act.length) return base;
  if (mode === 'hangul') { const r = hangulSearch(act, q, limit); return { ...base, rows: r.rows, hints: r.hints }; }
  if (mode === 'han') { const r = hanSearch(act, dbs.core, q, limit); return { ...base, rows: r.rows, hanja: r.hanja }; }
  return { ...base, rows: latinSearch(dbs.core, q, limit) };
}

export function getEntriesByHeadword(dbs: Dbs, hw: string, stdict: boolean): EntryRow[] {
  const norm = normHeadword(hw);
  return active(dbs, stdict).flatMap((db) =>
    db.selectObjects(`SELECT ${COLS} FROM entries e WHERE e.hw_norm = ? ORDER BY e.rank`, [norm]) as unknown as EntryRow[]);
}

export function getEntry(dbs: Dbs, source: Source, id: number): Entry | null {
  const db = packFor(dbs, source);
  if (!db) return null;
  const o = db.selectObjects(`SELECT ${COLS}, e.data FROM entries e WHERE e.id = ?`, [id])[0];
  if (!o) return null;
  let data = { senses: [] } as Entry['data'];
  try { data = JSON.parse(String(o.data)); } catch { /* keep empty */ }
  return { ...(o as unknown as EntryRow), data };
}

/** Entry ids are per pack, so fetch a batch of full entries by (source, id) from one pack. */
export function getEntries(dbs: Dbs, source: Source, ids: number[]): Entry[] {
  return ids.map((id) => getEntry(dbs, source, id)).filter((e): e is Entry => !!e);
}

export function hanjaChar(dbs: Dbs, ch: string): HanjaChar | null {
  return (dbs.core?.selectObjects('SELECT * FROM hanja_chars WHERE ch = ?', [ch])[0] as unknown as HanjaChar) ?? null;
}

export function wordsWithHanja(dbs: Dbs, ch: string, limit: number, offset: number, stdict: boolean): ResultRow[] {
  const rows: ResultRow[] = [];
  for (const db of active(dbs, stdict))
    for (const o of db.selectObjects(
      `SELECT ${COLS} FROM entries e WHERE e.id IN (SELECT entry_id FROM hanja_words WHERE ch = ?) ORDER BY e.rank LIMIT ? OFFSET ?`,
      [ch, limit + offset, 0])) rows.push(toRow(o, 'hanja'));
  rows.sort((a, b) => a.rank - b.rank);
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
export function randomWordOfDay(dbs: Dbs, dateStr: string): EntryRow | null {
  const core = dbs.core;
  if (!core) return null;
  for (const where of ["e.source = 'krdict' AND e.level IN (1,2) AND e.kind = 'word'", "e.kind = 'word'"]) {
    const n = Number(core.selectObjects(`SELECT COUNT(*) AS n FROM entries e WHERE ${where}`)[0]?.n ?? 0);
    if (!n) continue;
    const o = core.selectObjects(`SELECT ${COLS} FROM entries e WHERE ${where} ORDER BY e.id LIMIT 1 OFFSET ?`, [hash(dateStr) % n])[0];
    if (o) return o as unknown as EntryRow;
  }
  return null;
}
