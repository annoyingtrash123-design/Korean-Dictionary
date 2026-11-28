// The "Study lists" source of the Lists tab, backed by the engine (see core/src/search/study.rs).
// A pack without the `curriculum` / `topic` tables gives an empty overview, so those lists are hidden.
import { useEffect } from 'preact/hooks';
import { db, packStatus$ } from '../db/client';
import type { StudyOverview } from '../db/types';
import { createStore, useStore } from './store';
import { curriculumLabel } from './home-day';
import { IDIOM_LISTS, idiomRow } from './study-idioms';
import { USAGE_LISTS } from './study-usage';
import type { ListRow, ListSource, ListSummary } from './list-sources';
import { grammarRow, loadTopicSort, parseStudyId, saveTopicSort, STUDY_PREFIX, studyMemory, summariesOf, TOPIC_SORTS, wordRow } from './study-lists';

/** The overview (`null`: the pack has no study lists or could not be read; `undefined`: not loaded yet). */
export const overview$ = createStore<StudyOverview | null | undefined>(undefined);
let requested: string | undefined;

/** Load the overview once per installed core version (a dictionary update reloads it). */
export function loadOverview(version: string, fetch: () => Promise<StudyOverview> = () => db.studyOverview()): void {
  if (requested === version) return;
  requested = version;
  fetch().then((o) => overview$.set(o), () => overview$.set(null));
}
/** For tests: forget what was loaded. */
export function resetOverview(): void { requested = undefined; overview$.set(undefined); }

function useOverview() {
  const st = useStore(packStatus$);
  const core = st?.packs.core;
  const version = core?.installed ? core.version ?? 'core' : undefined;
  useEffect(() => { if (version) loadOverview(version); }, [version]);
  return useStore(overview$);
}

/** A page of a list's rows. `sort` matters for topics only ('level': 초급 → 고급, each in 가나다 order; 'az'). */
export async function fetchRows(listId: string, sort: string, offset: number, limit: number): Promise<ListRow[]> {
  const ref = parseStudyId(listId);
  if (!ref) return [];
  if (ref.kind === 'vocab') return (await db.studyWords(ref.level, limit, offset)).map(wordRow);
  if (ref.kind === 'grammar') return (await db.studyGrammar(ref.level)).slice(offset, offset + limit).map((g, i) => grammarRow(g, ref.level, offset + i));
  if (ref.kind === 'idiom') return (await db.studyIdioms(ref.list, limit, offset)).map(idiomRow);
  if (ref.kind === 'usage') return [];   // not a row list: the usage lists have their own views (views/UsageLists.tsx)
  return (await db.studyTopicWords(ref.category, sort !== 'az', limit, offset)).map(wordRow);
}

/** Every row of a list (for "Add all to a list"). */
export async function fetchAllRows(listId: string, sort: string, fetch: typeof fetchRows = fetchRows, step = 500): Promise<ListRow[]> {
  const out: ListRow[] = [];
  for (;;) {
    const page = await fetch(listId, sort, out.length, step);
    out.push(...page);
    if (page.length < step) return out;
  }
}

export const studySource: ListSource = {
  id: 'study',
  title: 'Study lists',
  readOnly: true,
  owns: (id) => id.startsWith(STUDY_PREFIX),
  useLists() {
    return summariesOf(useOverview());
  },
  useList(id) {
    const o = useOverview();
    const ref = parseStudyId(id);
    if (!ref) return undefined;
    const found = summariesOf(o).find((s) => s.id === id);
    if (found) return { ...found, items: [] };
    if (o !== undefined) return undefined;
    // the counts are still loading: the page can already show its title
    const name = ref.kind === 'topic' ? ref.category : ref.kind === 'idiom' ? IDIOM_LISTS[ref.list].name : ref.kind === 'usage' ? USAGE_LISTS[ref.list].name : curriculumLabel(ref.level);
    const s: ListSummary = { id, name, count: 0, loading: true };
    return { ...s, items: [] };
  },
  paged: {
    sorts: (id) => (parseStudyId(id)?.kind === 'topic' ? TOPIC_SORTS : undefined),
    defaultSort: loadTopicSort,
    setSort: saveTopicSort,
    fetch: fetchRows,
  },
};
