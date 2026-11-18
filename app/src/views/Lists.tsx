import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import { Empty, Hanja, IconButton, LevelBadge } from '../components/common';
import { Icon } from '../components/Icons';
import { NameForm } from '../components/SaveButton';
import { createList, deleteList, removeFromList, renameList, restoreItem, lists, type ListItem } from '../lib/lists';
import { LIST_SOURCES, sourceOf, type ListSource, type ListSummary } from '../lib/list-sources';
import { PagedDetail } from './StudyList';
import { href, navigate, useRoute, withoutTab } from '../lib/router';
import { savedPath } from '../lib/entry-key';
import { showToast } from '../lib/toast';

export const listPath = (id?: string) => (id ? `/lists/${encodeURIComponent(id)}` : '/lists');

/** Lists tab: the overview, or one list when `listId` is given. */
export function ListsView({ listId }: { listId?: string }) {
  return listId ? <ListDetail key={listId} id={listId} /> : (
    <div class="page lists">
      {LIST_SOURCES.map((s) => <SourceSection key={s.id} src={s} />)}
    </div>
  );
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
/** "today", "yesterday", "3 Nov", "3 Nov 2025". */
export function fmtWhen(t: number, now = new Date()): string {
  const d = new Date(t);
  const day = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const diff = Math.round((day(now) - day(d)) / 86_400_000);
  if (diff === 0) return 'today';
  if (diff === 1) return 'yesterday';
  return `${d.getDate()} ${MONTHS[d.getMonth()]}${d.getFullYear() === now.getFullYear() ? '' : ` ${d.getFullYear()}`}`;
}
const plural = (n: number, w: string) => `${n.toLocaleString('en-US')} ${w}${n === 1 ? '' : 's'}`;

function SourceSection({ src }: { src: ListSource }) {
  const all = src.useLists();
  const route = useRoute();
  const [adding, setAdding] = useState(false);
  const onNew = async (name: string | null) => {
    setAdding(false);
    if (name) navigate(listPath(await createList(name)));
  };
  // built-in sources have nothing to show until their lists are known (or when the data has none)
  if (src.readOnly && !all.length) return null;
  const current = (id: string): 'true' | undefined => (withoutTab(route.raw) === listPath(id) ? 'true' : undefined);
  const groups: { name?: string; lists: ListSummary[] }[] = [];
  for (const l of all.filter((x) => !x.parent)) {
    const g = groups.find((x) => x.name === l.group);
    if (g) g.lists.push(l); else groups.push({ name: l.group, lists: [l] });
  }
  const listItems = (ls: ListSummary[]) => ls.map((l) => (
    <li key={l.id}>
      <ListRow l={l} current={current(l.id)} subs={all.filter((s) => s.parent === l.id)} currentOf={current} />
    </li>
  ));
  const resume = src.lastOpened?.();
  const last = resume ? all.find((l) => l.id === resume) : undefined;
  return (
    <section aria-labelledby={`lists-${src.id}`} data-source={src.id}>
      <div class="section-head">
        <h2 id={`lists-${src.id}`}>{src.title}</h2>
        {!src.readOnly && !adding && <div class="actions"><button type="button" class="link" onClick={() => setAdding(true)}>New list</button></div>}
      </div>
      {adding && <div class="pad"><NameForm label="New list name" submit="Create" onDone={onNew} /></div>}
      {!all.length && !adding && <Empty title="No lists yet">Tap the star on any entry to save it, or make a list with “New list”.</Empty>}
      {last && (
        <ul class="plain list resume">
          <li><ListRow l={last} current={current(last.id)} lead="Continue" /></li>
        </ul>
      )}
      {groups.map((g) => (g.name
        ? <FoldGroup key={g.name} id={`${src.id}:${g.name}`} name={g.name} lists={g.lists}>{listItems(g.lists)}</FoldGroup>
        : <div key="" class="list-group"><ul class="plain list">{listItems(g.lists)}</ul></div>))}
    </section>
  );
}

const OPEN_KEY = 'kd.lists.open';
/** Groups left open on the overview, remembered on this device; all collapsed by default. */
function loadOpen(): string[] { try { const v = JSON.parse(localStorage.getItem(OPEN_KEY) || '[]'); return Array.isArray(v) ? v.filter((x) => typeof x === 'string') : []; } catch { return []; /* storage unavailable or unparsable: all collapsed */ } }
function saveOpen(v: string[]) { try { localStorage.setItem(OPEN_KEY, JSON.stringify(v)); } catch { /* storage unavailable: the choice lasts for this visit */ } }
const unitOf = (name: string) => (/level/i.test(name) ? 'level' : /topic/i.test(name) ? 'topic' : 'list');

/** A collapsible group of lists: the heading is the toggle; collapsed it summarises the count. Closed groups stay mounted (hidden). */
function FoldGroup({ id, name, lists: ls, children }: { id: string; name: string; lists: ListSummary[]; children: ComponentChildren }) {
  const [open, setOpen] = useState(() => loadOpen().includes(id));
  const toggle = () => {
    const next = !open;
    setOpen(next);
    const cur = loadOpen().filter((x) => x !== id);
    saveOpen(next ? [...cur, id] : cur);
  };
  const body = `lg-${id.replace(/[^\w-]+/g, '-')}`;
  return (
    <div class="list-group" role="group" aria-label={name}>
      <h3 class="group-head"><button type="button" class="group-fold" aria-expanded={open} aria-controls={body} onClick={toggle}>
        <span class="rd-chev" aria-hidden="true" /><span class="group-title">{name}</span>
        {!open && <span class="group-sum muted small">{plural(ls.length, unitOf(name))}</span>}
      </button></h3>
      <ul id={body} class="plain list" hidden={!open}>{children}</ul>
    </div>
  );
}

/** One list on the overview. A list with sub-lists (a top-level topic) gets a button that unfolds them. */
function ListRow({ l, current, lead, subs = [], currentOf }: { l: ListSummary; current?: 'true'; lead?: string; subs?: ListSummary[]; currentOf?: (id: string) => 'true' | undefined }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <div class={`list-row-wrap${subs.length ? ' has-subs' : ''}`}>
        <a class="row compact list-row" href={href(listPath(l.id))} aria-current={current} title={l.hint} aria-label={l.hint ? `${l.name}. ${l.hint}. ${plural(l.count, l.unit ?? 'item')}` : undefined}>
          <span class="list-ico" aria-hidden="true">{l.badge ? <span class="list-badge">{l.badge}</span> : <Icon name="lists" size={20} />}</span>
          <div class="row-main">
            <div class="row-head"><span class="list-name">{lead ? `${lead}: ${l.name}` : l.name}</span></div>
            <div class="row-gloss muted small">{lead && l.eyebrow ? `${l.eyebrow} · ` : ''}{plural(l.count, l.unit ?? 'item')}{l.updated ? ` · updated ${fmtWhen(l.updated)}` : ''}</div>
          </div>
          {!subs.length && <Icon name="chev" size={18} />}
        </a>
        {subs.length > 0 && (
          <button type="button" class="iconbtn sub-toggle" aria-expanded={open} aria-label={`${open ? 'Hide' : 'Show'} the ${subs.length} sub-categories of ${l.name}`} onClick={() => setOpen(!open)}>
            <span class={`sub-chev${open ? ' open' : ''}`}><Icon name="down" size={20} /></span>
          </button>
        )}
      </div>
      {open && (
        <ul class="plain list subs" aria-label={`${l.name}: sub-categories`}>
          {subs.map((s) => (
            <li key={s.id}>
              <a class="row compact list-row" href={href(listPath(s.id))} aria-current={currentOf?.(s.id)}>
                <div class="row-main"><div class="row-head"><span class="list-name sub-name">{s.name}</span></div></div>
                <span class="count" aria-label={plural(s.count, s.unit ?? 'item')}>{s.count.toLocaleString('en-US')}</span>
                <Icon name="chev" size={18} />
              </a>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}

type Sort = 'added' | 'az';
const SORT_KEY = 'kd.lists.sort';
const loadSort = (): Sort => { try { return localStorage.getItem(SORT_KEY) === 'az' ? 'az' : 'added'; } catch { return 'added'; /* storage unavailable */ } };
const saveSort = (s: Sort) => { try { localStorage.setItem(SORT_KEY, s); } catch { /* storage unavailable: the choice lasts for this visit */ } };

const itemTitle = (i: ListItem) => (i.kind === 'grammar' ? i.pattern : i.headword);
export const itemPath = (i: ListItem) => (i.kind === 'grammar'
  ? savedPath({ source: i.ref?.source ?? 'krdict', headword: i.pattern, homonym: i.ref?.homonym ?? null, pos: i.ref?.pos ?? null })
  : savedPath(i));
const alphaKey = (i: ListItem) => itemTitle(i).replace(/^[-~·\s]+/, '');

function ListDetail({ id }: { id: string }) {
  const src = sourceOf(id);
  const l = src?.useList(id);
  const route = useRoute();
  const [sort, setSort] = useState<Sort>(loadSort);
  const [edit, setEdit] = useState(false);
  const [renaming, setRenaming] = useState(false);
  if (src?.paged && l) return <PagedDetail key={id} src={src} list={l} back={<BackLink />} />;
  if (!src || !l) {
    return <div class="page"><BackLink /><Empty title="List not found">It may have been deleted. <a href={href(listPath())}>All lists</a></Empty></div>;
  }
  const items = sort === 'az' ? [...l.items].sort((a, b) => alphaKey(a).localeCompare(alphaKey(b), 'ko')) : l.items;
  const onSort = (s: Sort) => { setSort(s); saveSort(s); };
  const onRename = async (name: string | null) => { setRenaming(false); if (name) await renameList(id, name); };
  const onDelete = async () => {
    if (!confirm(`Delete the list “${l.name}”? Its ${plural(l.count, 'item')} will be removed from this list.`)) return;
    await deleteList(id);
    navigate(listPath(), true);
  };
  const onRemove = async (it: ListItem) => {
    const index = lists.get().lists.find((x) => x.id === id)?.items.findIndex((i) => i.key === it.key) ?? -1;
    await removeFromList(id, it.key);
    showToast(`Removed ${itemTitle(it)} from ${l.name}`, [{ label: 'Undo', run: () => void restoreItem(id, it, index) }]);
  };
  return (
    <div class="page list-detail">
      <BackLink />
      {renaming ? <div class="pad"><NameForm initial={l.name} label="List name" submit="Rename" onDone={onRename} /></div> : (
        <div class="list-title-row">
          <h1 class="list-title">{l.name}</h1>
          {!src.readOnly && (l.count > 0 || edit) && <button type="button" class="link" onClick={() => setEdit(!edit)}>{edit ? 'Done' : 'Edit'}</button>}
        </div>
      )}
      <div class="list-sub">
        <span class="muted small">{plural(l.count, 'item')}</span>
        {l.count > 1 && (
          <div class="seg inline" role="radiogroup" aria-label="Sort">
            <button type="button" role="radio" aria-checked={sort === 'added'} class={sort === 'added' ? 'on' : ''} onClick={() => onSort('added')}>Recent</button>
            <button type="button" role="radio" aria-checked={sort === 'az'} aria-label="Alphabetical" class={sort === 'az' ? 'on' : ''} onClick={() => onSort('az')}>가나다</button>
          </div>
        )}
      </div>
      {edit && !renaming && !src.readOnly && (
        <div class="list-edit">
          <button type="button" class="btn" onClick={() => setRenaming(true)}><Icon name="edit" size={18} /> Rename</button>
          <button type="button" class="btn danger" onClick={onDelete}><Icon name="trash" size={18} /> Delete list</button>
        </div>
      )}
      {l.count === 0 && <Empty title="Nothing here yet">Tap the star on an entry to save it here, or long-press the star to pick lists.</Empty>}
      <ul class="plain list">
        {items.map((it) => (
          <li key={it.key} class="bm">
            <a class="row compact" href={href(itemPath(it))} aria-current={withoutTab(route.raw) === itemPath(it) ? 'true' : undefined}>
              <div class="row-main">
                {it.kind === 'grammar' ? <>
                  <div class="row-head"><span class="hangul hw" lang="ko">{it.pattern}</span><span class="pos">grammar</span><LevelBadge level={it.level} /></div>
                  {it.summary && <div class="row-gloss">{it.summary}</div>}
                </> : <>
                  <div class="row-head"><span class="hangul hw" lang="ko">{it.headword}</span>{it.hanja && <Hanja text={it.hanja} />}</div>
                  {it.gloss && <div class="row-gloss">{it.gloss}</div>}
                </>}
              </div>
            </a>
            {edit && !src.readOnly && <div class="bm-actions"><IconButton icon="trash" label={`Remove ${itemTitle(it)}`} onClick={() => void onRemove(it)} /></div>}
          </li>
        ))}
      </ul>
    </div>
  );
}

function BackLink() {
  return <a class="back-link" href={href(listPath())}><Icon name="back" size={18} /> <span>All lists</span></a>;
}
