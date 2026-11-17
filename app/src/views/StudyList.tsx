import { useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Hanja, LevelBadge, Pos, Sheet } from '../components/common';
import { Icon } from '../components/Icons';
import { NameForm, SaveButton } from '../components/SaveButton';
import { VirtualList } from '../components/VirtualList';
import { addManyToList, createList, itemKey, lists, targetList } from '../lib/lists';
import { useStore } from '../lib/store';
import { showToast } from '../lib/toast';
import { href, useRoute, withoutTab } from '../lib/router';
import type { ListRow, ListSource, ListSummary } from '../lib/list-sources';
import { PAGE, studyMemory } from '../lib/study-lists';
import { fetchAllRows } from '../lib/study-source';

export const ROW_HEIGHT = 60;
const plural = (n: number, w: string) => `${n.toLocaleString('en-US')} ${w}${n === 1 ? '' : 's'}`;

/** A built-in (read-only) list: header, then every row of the list in a virtual list, each with the save star. */
export function PagedDetail({ src, list, back }: { src: ListSource; list: ListSummary; back: ComponentChildren }) {
  const paged = src.paged!;
  const sorts = paged.sorts?.(list.id);
  const [sort, setSort] = useState(() => (sorts ? paged.defaultSort?.() ?? sorts[0].id : ''));
  const [adding, setAdding] = useState(false);
  const route = useRoute();
  const unit = list.unit ?? 'item';
  const title = list.fullName ?? list.name;
  const [start, setStart] = useState(() => studyMemory.row(list.id));
  const onSort = (s: string) => { setSort(s); paged.setSort?.(s); setStart(0); studyMemory.remember(list.id, 0); };

  return (
    <div class="page list-detail study-host">
      {back}
      <div class="list-title-row">
        <div class="list-titles">
          {list.eyebrow && <div class="eyebrow">{list.eyebrow}</div>}
          <h1 class="list-title" title={list.hint} lang="ko">{title}</h1>
        </div>
        {list.count > 0 && <button type="button" class="link" aria-label="Add all to a list…" onClick={() => setAdding(true)}>Add all…</button>}
      </div>
      {list.hint && <p class="list-hint muted small">{list.hint}</p>}
      <div class="list-sub">
        <span class="muted small" data-testid="study-count">{list.loading ? 'Loading…' : plural(list.count, unit)}</span>
        {sorts && list.count > 1 && (
          <div class="seg inline" role="radiogroup" aria-label="Sort">
            {sorts.map((s) => <button key={s.id} type="button" role="radio" aria-checked={sort === s.id} aria-label={s.aria} class={sort === s.id ? 'on' : ''} onClick={() => onSort(s.id)}>{s.label}</button>)}
          </div>
        )}
      </div>
      {list.count > 0 && (
        <VirtualList<ListRow>
          key={`${list.id}|${sort}`}
          count={list.count} rowHeight={ROW_HEIGHT} pageSize={PAGE} label={`${title}: ${plural(list.count, unit)}`}
          startRow={start}
          onRow={(row) => studyMemory.remember(list.id, row)}
          fetchPage={(o, n) => paged.fetch(list.id, sort, o, n)}
          renderRow={(r) => <Row r={r} current={withoutTab(route.raw) === r.path} />}
        />
      )}
      {!list.loading && list.count === 0 && <div class="empty"><div class="muted">Nothing in this list.</div></div>}
      {adding && <AddAll src={src} list={list} sort={sort} onClose={() => setAdding(false)} />}
    </div>
  );
}

function Row({ r, current }: { r: ListRow; current: boolean }) {
  return (
    <div class="study-row">
      <a class="row compact" href={href(r.path)} aria-current={current ? 'true' : undefined}>
        <div class="row-main">
          <div class="row-head">
            <span class="hangul hw" lang="ko">{r.title}</span>
            {r.hanja && <Hanja text={r.hanja} />}
            <Pos pos={r.pos} />
            <LevelBadge level={r.level} />
          </div>
          <div class="row-gloss" title={r.gloss || undefined}>{r.gloss || <span class="muted">Korean definition only</span>}</div>
        </div>
      </a>
      <div class="bm-actions"><SaveButton item={r.item} title={r.title} size={22} /></div>
    </div>
  );
}

/** Sheet: copy every row of the list into one of the user's lists. */
function AddAll({ src, list, sort, onClose }: { src: ListSource; list: ListSummary; sort: string; onClose: () => void }) {
  const d = useStore(lists);
  const [busy, setBusy] = useState(false);
  const [naming, setNaming] = useState(false);
  const last = targetList(d)?.id;
  const go = async (listId: string, name: string) => {
    setBusy(true);
    try {
      const rows = await fetchAllRows(list.id, sort, src.paged!.fetch);
      const added = await addManyToList(listId, rows.map((r) => r.item));
      onClose();
      // two rows of the official list can be one saved item (two spellings of one pattern)
      const distinct = new Set(rows.map((r) => itemKey(r.item))).size;
      showToast(added === distinct ? `Added ${plural(added, list.unit ?? 'item')} to ${name}` : `Added ${added.toLocaleString('en-US')} new of ${distinct.toLocaleString('en-US')} to ${name}`);
    } catch {
      setBusy(false);
      showToast('Could not add the list. Try again.');
    }
  };
  const create = async (name: string | null) => {
    setNaming(false);
    if (name) { const id = await createList(name); await go(id, name); }
  };
  return (
    <Sheet title="Add all to list" onClose={onClose}>
      <p class="picker-what"><span class="hangul" lang="ko">{list.fullName ?? list.name}</span> · {plural(list.count, list.unit ?? 'item')}</p>
      <ul class="plain picker-lists" aria-label="Lists">
        {d.lists.map((l, i) => (
          <li key={l.id}>
            <button type="button" class="sheet-item pick-btn" disabled={busy} data-autofocus={i === 0 ? '' : undefined} onClick={() => void go(l.id, l.name)}>
              <span class="pick-name">{l.name}</span>
              {l.id === last && <span class="pick-last">tap saves here</span>}
              <span class="count" aria-label={`${l.items.length} items`}>{l.items.length}</span>
            </button>
          </li>
        ))}
      </ul>
      {naming
        ? <NameForm label="New list name" submit="Create" onDone={create} />
        : <button type="button" class="sheet-item new-list-btn" disabled={busy} data-autofocus={d.lists.length ? undefined : ''} onClick={() => setNaming(true)}><Icon name="plus" size={20} /> <span>New list</span></button>}
      {busy && <p class="muted small" role="status">Adding…</p>}
      <button type="button" class="btn" onClick={onClose}>Cancel</button>
    </Sheet>
  );
}
