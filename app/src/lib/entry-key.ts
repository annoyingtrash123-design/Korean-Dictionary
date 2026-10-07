// Stable identity for saved words. entries.id is renumbered by every monthly data rebuild, so bookmarks and
// history are keyed on `source|headword|homonym|pos` and re-resolved against the current database when opened.
export interface EntryRef { source: string; headword: string; homonym?: number | null; pos?: string | null }
/** What a saved item remembers about its entry. */
export interface SavedRef extends EntryRef { hanja: string | null; gloss: string | null; homonym: number | null; pos: string | null }

export const stableKey = (e: EntryRef) => `${e.source}|${e.headword}|${e.homonym ?? ''}|${e.pos ?? ''}`;

/** Pick the saved entry among all entries of its headword: exact source+homonym+pos, then source+homonym, source+pos, source, any. */
export function resolveEntry<T extends { source: string; homonym?: number | null; pos?: string | null }>(entries: T[], ref: Partial<EntryRef>): T | null {
  if (!entries.length) return null;
  const hom = ref.homonym ?? null, pos = ref.pos ?? null;
  const same = ref.source ? entries.filter((e) => e.source === ref.source) : [];
  const tests: ((e: T) => boolean)[] = [
    (e) => (e.homonym ?? null) === hom && (e.pos ?? null) === pos,
    (e) => (e.homonym ?? null) === hom,
    (e) => (e.pos ?? null) === pos,
  ];
  for (const t of tests) { const m = same.find(t); if (m) return m; }
  return same[0] ?? entries[0];
}

/** Route that opens a saved word by headword (never by id). */
export function savedPath(r: EntryRef): string {
  const q = new URLSearchParams();
  q.set('s', r.source);
  if (r.homonym != null) q.set('h', String(r.homonym));
  if (r.pos) q.set('p', r.pos);
  return `/word/${encodeURIComponent(r.headword)}?${q}`;
}
/** Inverse of the query part of `savedPath`. */
export function refFromParams(p: URLSearchParams): Partial<EntryRef> {
  const h = p.get('h');
  return { source: p.get('s') ?? undefined, homonym: h != null && h !== '' ? Number(h) : null, pos: p.get('p') };
}

/** Legacy items (saved as `source:id`) carry the old id until `upgradeLegacy` resolves them. */
export const legacyKey = (source: string, headword: string, id: number) => `${source}|${headword}||#${id}`;

/**
 * Fill in homonym/pos of legacy items by looking their old id up while the ids still match; items whose id no longer
 * points at the same headword fall back to `source|headword||` (resolved by fallback rules when opened).
 */
export async function upgradeLegacy<T extends SavedRef & { key: string; legacyId?: number }>(
  items: T[], lookup: (source: string, id: number) => Promise<{ headword: string; homonym?: number | null; pos?: string | null } | null>,
): Promise<{ items: T[]; changed: boolean }> {
  if (!items.some((i) => i.legacyId != null)) return { items, changed: false };
  const out: T[] = [];
  for (const it of items) {
    if (it.legacyId == null) { out.push(it); continue; }
    let e: Awaited<ReturnType<typeof lookup>> = null;
    try { e = await lookup(it.source, it.legacyId); } catch { /* engine unavailable: keep as legacy */ return { items, changed: false }; }
    const next = { ...it, legacyId: undefined } as T;
    if (e && e.headword === it.headword) { next.homonym = e.homonym ?? null; next.pos = e.pos ?? null; }
    next.key = stableKey(next);
    out.push(next);
  }
  const seen = new Set<string>();
  return { items: out.filter((i) => (seen.has(i.key) ? false : (seen.add(i.key), true))), changed: true };
}
