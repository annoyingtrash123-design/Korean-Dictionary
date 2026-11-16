// Where the Lists tab gets its lists from. Today only the user's own lists; built-in read-only lists
// (TOPIK vocabulary and grammar, idioms, topics) come later as further sources with their own id prefix
// (`topik:…`), so the views never need to know where a list comes from.
import { useStore } from './store';
import { findList, lists, type ListItem } from './lists';

export interface ListSummary { id: string; name: string; count: number; updated?: number }
export interface ListSource {
  id: string;
  /** Section heading on the Lists tab. */
  title: string;
  /** Read-only lists offer no rename, delete or remove. */
  readOnly: boolean;
  /** Does this source own the list id? */
  owns(listId: string): boolean;
  /** Hook: the source's lists (a built-in source may load them lazily and return [] meanwhile). */
  useLists(): ListSummary[];
  /** Hook: one list (undefined while loading or when it does not exist). */
  useList(listId: string): (ListSummary & { items: ListItem[] }) | undefined;
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
export const LIST_SOURCES: ListSource[] = [userLists];
export const sourceOf = (listId: string) => LIST_SOURCES.find((s) => s.owns(listId));
