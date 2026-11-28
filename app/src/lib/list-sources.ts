// Where the Lists tab gets its lists from: the user's own lists, and read-only built-in "study lists"
// (curriculum vocabulary and grammar by level, topics; `study:…` ids). The views never need to know where
// a list comes from.
import { useStore } from './store';
import { findList, lists, type ItemInput, type ListItem } from './lists';
import { studySource } from './study-source';

export interface ListSummary {
  id: string;
  name: string;
  count: number;
  updated?: number;
  /** What the items are called in "2,575 words" (default: item). */
  unit?: string;
  /** Sub-heading on the overview this list sits under. */
  group?: string;
  /** Small label above the title on the list's own page. */
  eyebrow?: string;
  /** Tooltip / accessible description. */
  hint?: string;
  /** Short mark in the overview's icon slot (instead of the list icon). */
  badge?: string;
  /** Overview: the list this one is nested under (a sub-category's top-level topic). */
  parent?: string;
  /** Title of the list's own page when it differs from `name`. */
  fullName?: string;
  /** Counts are not known yet. */
  loading?: boolean;
}

/** One row of a built-in list: what to show, where it opens, and what the star saves. */
export interface ListRow {
  key: string;
  title: string;
  hanja: string | null;
  pos: string | null;
  /** krdict level 1-3 (shows the 초/중/고 badge). */
  level: number | null;
  gloss: string;
  /** Small label next to the title (Idioms & proverbs: 속담 / 성어). */
  tag?: string;
  path: string;
  item: ItemInput;
  /** A reviewed mini-lesson exists (grammar rows: shows the Lesson badge). */
  lesson?: boolean;
}

export interface PagedLists {
  /** Orders a list offers (none: a single fixed order). */
  sorts?(listId: string): readonly { id: string; label: string; aria: string }[] | undefined;
  defaultSort?(): string;
  setSort?(sort: string): void;
  /** A page of rows (the whole list may be too long to load at once). */
  fetch(listId: string, sort: string, offset: number, limit: number): Promise<ListRow[]>;
}

export interface ListSource {
  id: string;
  /** Section heading on the Lists tab. */
  title: string;
  /** Read-only lists offer no rename, delete or remove. */
  readOnly: boolean;
  /** Does this source own the list id? */
  owns(listId: string): boolean;
  /** Hook: the source's lists (a built-in source loads them lazily and returns [] meanwhile; it is hidden while empty). */
  useLists(): ListSummary[];
  /** Hook: one list (undefined while loading or when it does not exist). */
  useList(listId: string): (ListSummary & { items: ListItem[] }) | undefined;
  /** Built-in lists read in pages (`useList().items` is empty). */
  paged?: PagedLists;
}

export const userLists: ListSource = {
  id: 'user',
  title: 'My lists',
  readOnly: false,
  owns: (id) => !id.includes(':'),
  useLists() {
    const d = useStore(lists);
    return d.lists.map((l) => ({ id: l.id, name: l.name, count: l.items.length, updated: l.updated }));
  },
  useList(id) {
    const l = findList(useStore(lists), id);
    return l && { id: l.id, name: l.name, count: l.items.length, updated: l.updated, items: l.items };
  },
};

/** All sources, in the order the Lists tab shows them. */
export const LIST_SOURCES: ListSource[] = [userLists, studySource];
export const sourceOf = (listId: string) => LIST_SOURCES.find((s) => s.owns(listId));
