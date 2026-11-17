// Built-in study lists (Lists tab): list ids, labels, rows and the remembered position. Pure (no engine):
// the engine glue is in `study-source.ts`.
import type { StudyGrammar, StudyOverview, StudyWord } from '../db/types';
import type { ListRow, ListSummary } from './list-sources';
import { entryPath, wordPath } from './router';
import { curriculumHint, curriculumLabel } from './home-day';

export const STUDY_PREFIX = 'study:';
export const LEVELS = [1, 2, 3, 4, 5, 6];
export const PAGE = 100;
export const GROUP_VOCAB = 'Vocabulary by level';
export const GROUP_GRAMMAR = 'Grammar by level';
export const GROUP_TOPICS = 'Topics';

export type StudyRef = { kind: 'vocab'; level: number } | { kind: 'grammar'; level: number } | { kind: 'topic'; category: string };

export const studyId = (r: StudyRef): string => (r.kind === 'topic' ? `${STUDY_PREFIX}topic:${r.category}` : `${STUDY_PREFIX}${r.kind}:${r.level}`);
export function parseStudyId(id: string): StudyRef | undefined {
  const m = /^study:(vocab|grammar|topic):(.+)$/.exec(id);
  if (!m) return undefined;
  if (m[1] === 'topic') return { kind: 'topic', category: m[2] };
  const level = Number(m[2]);
  return LEVELS.includes(level) ? { kind: m[1] as 'vocab' | 'grammar', level } : undefined;
}

/** A topic category's top level ("인간 > 감각" -> "인간"). */
export const topOf = (category: string) => category.split(' > ')[0];
/** Its last part ("인간 > 감각" -> "감각"). */
export const subOf = (category: string) => category.split(' > ').slice(1).join(' > ') || category;

/** The Lists tab's rows for the study lists: levels first, then topics (each top-level topic, followed by its sub-categories). */
export function summariesOf(o: StudyOverview | null | undefined): ListSummary[] {
  if (!o) return [];
  const out: ListSummary[] = [];
  for (const v of o.vocab) {
    if (v.count > 0) out.push({ id: studyId({ kind: 'vocab', level: v.level }), name: curriculumLabel(v.level), count: v.count, unit: 'word', group: GROUP_VOCAB, eyebrow: 'Vocabulary', hint: curriculumHint(v.level), badge: `${v.level}` });
  }
  for (const g of o.grammar) {
    if (g.count > 0) out.push({ id: studyId({ kind: 'grammar', level: g.level }), name: curriculumLabel(g.level), count: g.count, unit: 'pattern', group: GROUP_GRAMMAR, eyebrow: 'Grammar', hint: curriculumHint(g.level), badge: `${g.level}` });
  }
  const tops = new Map<string, { count: number; subs: { category: string; count: number }[] }>();
  for (const t of o.topics) {
    const top = topOf(t.category);
    const e = tops.get(top) ?? { count: 0, subs: [] };
    e.count += t.count;
    e.subs.push(t);
    tops.set(top, e);
  }
  for (const [top, e] of tops) {
    const id = studyId({ kind: 'topic', category: top });
    out.push({ id, name: top, count: e.count, unit: 'word', group: GROUP_TOPICS, eyebrow: 'Topic' });
    for (const s of e.subs) out.push({ id: studyId({ kind: 'topic', category: s.category }), name: subOf(s.category), count: s.count, unit: 'word', group: GROUP_TOPICS, eyebrow: `Topic · ${top}`, parent: id, fullName: s.category });
  }
  return out;
}

/** A row of a vocabulary or topic list. */
export function wordRow(w: StudyWord): ListRow {
  return {
    key: `${w.source}:${w.id}`, title: w.headword, hanja: w.hanja, pos: w.pos, level: w.level, gloss: w.gloss, path: entryPath(w.source, w.id, w.headword),
    item: { kind: 'entry', source: w.source, headword: w.headword, homonym: w.homonym, pos: w.pos, hanja: w.hanja, gloss: w.gloss },
  };
}

/** A row of a grammar list: the pattern as the official list writes it, the grammar row's English summary. */
export function grammarRow(g: StudyGrammar, level: number, i: number): ListRow {
  return {
    key: `g${level}:${i}`, title: g.form.replace(/(?<=[가-힣])\d+$/, ''), hanja: null, pos: null, level: null, gloss: g.summary_en ?? '',
    path: g.entry ? entryPath(g.entry.source, g.entry.id, g.pattern) : wordPath(g.pattern),
    item: { kind: 'grammar', pattern: g.pattern, summary: g.summary_en, level, ref: g.entry ? { source: g.entry.source, homonym: g.entry.homonym, pos: g.entry.pos } : null },
  };
}

export const TOPIC_SORTS = [
  { id: 'level', label: '초·중·고', aria: 'By level, then 가나다' },
  { id: 'az', label: '가나다', aria: 'Alphabetical (가나다)' },
] as const;

// ---- remembered position: the list opened last, and the first visible row of each list ----
const MEM_KEY = 'kd.study.pos';
interface Mem { last?: string; rows: Record<string, number> }
function readMem(): Mem {
  try {
    const v = JSON.parse(localStorage.getItem(MEM_KEY) ?? '{}');
    const rows: Record<string, number> = {};
    for (const [k, n] of Object.entries(v?.rows ?? {})) if (typeof n === 'number' && n >= 0) rows[k] = Math.floor(n);
    return { last: typeof v?.last === 'string' ? v.last : undefined, rows };
  } catch { return { rows: {} }; /* storage unavailable or damaged */ }
}
export const studyMemory = {
  last: () => readMem().last,
  row: (id: string) => readMem().rows[id] ?? 0,
  remember(id: string, row: number) {
    try {
      const m = readMem();
      m.last = id;
      m.rows[id] = Math.max(0, Math.floor(row));
      // keep the file small: positions of the 40 lists seen last are plenty
      const keys = Object.keys(m.rows);
      if (keys.length > 40) for (const k of keys.slice(0, keys.length - 40)) if (k !== id) delete m.rows[k];
      localStorage.setItem(MEM_KEY, JSON.stringify(m));
    } catch { /* storage unavailable: the position is not remembered */ }
  },
};

const SORT_KEY = 'kd.study.topicSort';
export const loadTopicSort = (): 'level' | 'az' => { try { return localStorage.getItem(SORT_KEY) === 'az' ? 'az' : 'level'; } catch { return 'level'; /* storage unavailable */ } };
export const saveTopicSort = (s: string) => { try { localStorage.setItem(SORT_KEY, s); } catch { /* storage unavailable: the choice lasts for this visit */ } };
