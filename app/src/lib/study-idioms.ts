// "Idioms & proverbs" study lists (Lists tab → Study lists): the most common 속담 and 성어, ranked by corpus
// frequency at build time. Pure (no engine): the engine glue is in `study-source.ts`.
// A row opens its dictionary entry when that pack is installed (stdict / 우리말샘 hold most of them) and
// otherwise searches for the headword.
import type { StudyIdiom, StudyIdiomList } from '../db/types';
import type { ListRow, ListSummary } from './list-sources';
import { entryPath, wordPath } from './router';

export const GROUP_IDIOMS = 'Idioms & proverbs';

interface IdiomListInfo { name: string; unit: string; eyebrow: string; badge: string; hint: string }
export const IDIOM_LISTS: Record<string, IdiomListInfo> = {
  sokdam100: { name: 'Common proverbs — top 100', unit: 'proverb', eyebrow: 'Proverbs 속담', badge: '속', hint: 'The 100 proverbs that occur most often in Korean Wikipedia, film subtitles and Wikisource' },
  sokdam_all: { name: 'All learner proverbs', unit: 'proverb', eyebrow: 'Proverbs 속담', badge: '속', hint: 'The proverbs in krdict (the Korean Learners’ Dictionary), most common first' },
  seongeo100: { name: 'Essential 성어 — top 100', unit: 'idiom', eyebrow: 'Four-character idioms 성어', badge: '성', hint: 'The 100 four-character idioms that occur most often in Korean text' },
  seongeo300: { name: 'Common 성어 — top 300', unit: 'idiom', eyebrow: 'Four-character idioms 성어', badge: '성', hint: 'The 300 four-character idioms that occur most often in Korean text' },
  seongeo_learner: { name: 'All learner 성어', unit: 'idiom', eyebrow: 'Four-character idioms 성어', badge: '성', hint: 'The four-character idioms in krdict (the Korean Learners’ Dictionary), most common first' },
};

export const KIND_LABEL: Record<StudyIdiom['kind'], string> = { sokdam: '속담', seongeo: '성어' };

export const idiomId = (list: string) => `study:idiom:${list}`;
export const parseIdiomId = (id: string): string | undefined => {
  const m = /^study:idiom:(\w+)$/.exec(id);
  return m && IDIOM_LISTS[m[1]] ? m[1] : undefined;
};

/** The overview rows of the group (only lists that have rows; none for a pack without the table). */
export function idiomSummaries(lists: StudyIdiomList[] | undefined): ListSummary[] {
  return (lists ?? []).filter((l) => l.count > 0 && IDIOM_LISTS[l.list]).map((l) => {
    const i = IDIOM_LISTS[l.list];
    return { id: idiomId(l.list), name: i.name, count: l.count, unit: i.unit, group: GROUP_IDIOMS, eyebrow: i.eyebrow, hint: i.hint, badge: i.badge };
  });
}

/** A row: headword, kind label, gloss, and the save star's item. */
export function idiomRow(x: StudyIdiom): ListRow {
  const e = x.entry;
  return {
    key: `${x.kind}:${x.pack}:${x.headword}`, title: x.title, hanja: null, pos: null, level: null, gloss: x.gloss, tag: KIND_LABEL[x.kind],
    path: e ? entryPath(e.source, e.id, x.headword) : wordPath(x.title),
    item: { kind: 'entry', source: e?.source ?? (x.pack === 'core' ? 'krdict' : x.pack), headword: x.headword, homonym: e?.homonym ?? null, pos: e?.pos ?? null, hanja: null, gloss: x.gloss },
  };
}
