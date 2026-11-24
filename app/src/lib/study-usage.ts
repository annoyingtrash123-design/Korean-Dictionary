// "Writing & usage" study lists (Lists tab → Study lists): 쓰기 expressions for TOPIK writing 51–54 and confusable words.
// Both are AI-written and reviewed (always labelled in the UI). Pure (no engine): the engine calls are in `db/client.ts`.
// They are not row lists with a save star: the first is grouped by function with expandable rows, the second opens a
// comparison view per group (`?g=<group id>` on the list's route).
import type { Confusable, UsageCounts, UsageEntryRef, WritingExpr } from '../db/types';
import type { ListSummary } from './list-sources';
import { entryPath, wordPath } from './router';

export const GROUP_USAGE = 'Writing & usage';
export const WRITING_ID = 'study:usage:writing';
export const CONFUSABLE_ID = 'study:usage:confusable';
export type UsageKind = 'writing' | 'confusable';

export const USAGE_LISTS: Record<UsageKind, { id: string; name: string; unit: string; eyebrow: string; badge: string; hint: string }> = {
  writing: {
    id: WRITING_ID, name: '쓰기 expressions (TOPIK writing)', unit: 'expression', eyebrow: 'TOPIK 쓰기 51–54', badge: '쓰',
    hint: 'Formal written expressions and connectors for the TOPIK writing section (items 53 and 54), grouped by function',
  },
  confusable: {
    id: CONFUSABLE_ID, name: 'Confusable words', unit: 'group', eyebrow: '헷갈리는 어휘', badge: '헷',
    hint: 'Look-alike spellings and near-synonyms that advanced learners mix up, with a contrasting example for each word',
  },
};

export const parseUsageId = (id: string): UsageKind | undefined => (id === WRITING_ID ? 'writing' : id === CONFUSABLE_ID ? 'confusable' : undefined);

/** The overview rows of the group (none for a pack without the tables, so the group is hidden). */
export function usageSummaries(u: UsageCounts | undefined): ListSummary[] {
  if (!u) return [];
  const out: ListSummary[] = [];
  for (const [kind, n] of [['writing', u.writing], ['confusable', u.confusables]] as const) {
    if (n > 0) {
      const l = USAGE_LISTS[kind];
      out.push({ id: l.id, name: l.name, count: n, unit: l.unit, group: GROUP_USAGE, eyebrow: l.eyebrow, hint: l.hint, badge: l.badge });
    }
  }
  return out;
}

/** Where an entry link opens: the entry when the pack has it, else a search for the word. */
export const usagePath = (e: UsageEntryRef | null | undefined, word: string): string => (e ? entryPath(e.source, e.id, word) : wordPath(word));

const norm = (s: string) => s.toLowerCase().replace(/[\s\-~()]+/g, '');

// ---- 쓰기 expressions ----
export interface WritingGroup { name: string; rows: WritingExpr[] }

/** Rows grouped by function, in the order of the list (a group is one consecutive block). */
export function writingGroups(rows: WritingExpr[]): WritingGroup[] {
  const out: WritingGroup[] = [];
  for (const r of rows) {
    const g = out[out.length - 1];
    if (g && g.name === r.group) g.rows.push(r); else out.push({ name: r.group, rows: [r] });
  }
  return out;
}

/** Rows whose expression, meaning, group or sentences contain the query (spaces, hyphens and parentheses ignored). */
export function filterWriting(rows: WritingExpr[], q: string): WritingExpr[] {
  const k = norm(q);
  if (!k) return rows;
  return rows.filter((r) => norm([r.expr, r.meaning, r.group, ...r.examples.map((e) => `${e.ko} ${e.en}`)].join(' ')).includes(k));
}

// ---- confusable words ----
export const CONFUSABLE_KINDS = [
  { id: 'all', label: 'All' },
  { id: 'spelling', label: 'Spelling' },
  { id: 'meaning', label: 'Meaning' },
  { id: 'grammar', label: 'Grammar' },
] as const;
export type ConfusableFilter = (typeof CONFUSABLE_KINDS)[number]['id'];
export const KIND_NAME: Record<Confusable['kind'], string> = { spelling: 'Spelling', meaning: 'Meaning', grammar: 'Grammar' };

/** "바라다 · 원하다" (wrong spellings are left out: they are a warning, not part of the contrast). */
export const confusableTitle = (c: Confusable): string => c.words.filter((w) => !w.wrong).map((w) => w.word).join(' · ');

/** Groups of one kind (or all) whose words, explanation or mistake contain the query. */
export function filterConfusables(rows: Confusable[], kind: ConfusableFilter, q: string): Confusable[] {
  const k = norm(q);
  return rows.filter((c) => (kind === 'all' || c.kind === kind) && (!k || norm([...c.words.map((w) => w.word), c.explanation, c.mistake].join(' ')).includes(k)));
}

/** The first sentence of an explanation, for the row under a group's title. */
export function firstSentence(s: string): string {
  const m = /^.*?[.!?](?=\s|$)/s.exec(s);
  return (m ? m[0] : s).trim();
}

export const confusablePath = (listPath: string, id: string) => `${listPath}?g=${encodeURIComponent(id)}`;
