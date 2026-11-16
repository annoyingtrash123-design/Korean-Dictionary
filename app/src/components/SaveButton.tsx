import { useEffect, useRef, useState } from 'preact/hooks';
import { Icon } from './Icons';
import { Sheet } from './common';
import { createStore, useStore } from '../lib/store';
import { showToast } from '../lib/toast';
import {
  addToList, createList, inList, itemKey, lists, listsWith, MAX_NAME, removeFromList, restoreItem, setLastUsed, targetList, toggleSave,
  type ItemInput,
} from '../lib/lists';

/** The open list picker (one for the whole app, so the toast's "Change" can open it too). */
const picker$ = createStore<{ item: ItemInput; title: string; back?: HTMLElement | null } | null>(null);
export const openPicker = (item: ItemInput, title: string, back?: HTMLElement | null) => picker$.set({ item, title, back });

const LONG_PRESS_MS = 500;

/**
 * Star button: a tap saves to the last-used list (or removes from it, with Undo); a long-press,
 * right-click or the keyboard's context-menu key (Shift+F10) opens the list picker.
 */
export function SaveButton({ item, title, size }: { item: ItemInput; title: string; size?: number }) {
  const d = useStore(lists);
  const btn = useRef<HTMLButtonElement>(null);
  const press = useRef<{ x: number; y: number; t: ReturnType<typeof setTimeout> } | null>(null);
  const longDone = useRef(false);
  const key = itemKey(item);
  const target = targetList(d);
  const inTarget = inList(target, key);
  const saved = listsWith(d, key).length > 0;
  const name = target?.name ?? 'Saved words';

  const pick = () => { longDone.current = true; openPicker(item, title, btn.current); };
  const cancel = () => { if (press.current) clearTimeout(press.current.t); press.current = null; };
  const tap = async () => {
    const before = targetList(lists.get());
    const index = before ? before.items.findIndex((i) => i.key === key) : -1;
    const old = index >= 0 ? before!.items[index] : undefined;
    const r = await toggleSave(item);
    if (r.removed && old) showToast(`Removed from ${r.list.name}`, [{ label: 'Undo', run: () => void restoreItem(r.list.id, old, index) }]);
    else showToast(`Saved to ${r.list.name}`, [{ label: 'Change', run: () => openPicker(item, title, btn.current) }]);
  };

  return (
    <button ref={btn} type="button" class={`iconbtn save-btn${saved ? ' on' : ''}`} data-kind={item.kind}
      aria-label={inTarget ? `Remove from ${name}` : `Save to ${name}`}
      aria-keyshortcuts="Shift+F10" title={`${inTarget ? `Remove from ${name}` : `Save to ${name}`} (long-press or right-click: choose lists)`}
      onPointerDown={(e) => {
        longDone.current = false;
        if (e.button !== 0) return;
        cancel();
        press.current = { x: e.clientX, y: e.clientY, t: setTimeout(() => { press.current = null; pick(); }, LONG_PRESS_MS) };
      }}
      onPointerMove={(e) => { const p = press.current; if (p && Math.hypot(e.clientX - p.x, e.clientY - p.y) > 10) cancel(); }}
      onPointerUp={cancel} onPointerCancel={cancel} onPointerLeave={cancel}
      onKeyDown={() => { longDone.current = false; }}
      onContextMenu={(e) => { e.preventDefault(); cancel(); pick(); }}
      onClick={() => { if (longDone.current) { longDone.current = false; return; } void tap(); }}>
      <Icon name="star" size={size} fill={saved} />
    </button>
  );
}

/** Inline name input (new list, rename): Enter saves, Escape cancels; focused when shown. */
export function NameForm({ initial = '', label, submit, onDone }: { initial?: string; label: string; submit: string; onDone: (name: string | null) => void }) {
  const [name, setName] = useState(initial);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => { input.current?.focus(); input.current?.select(); }, []);
  return (
    <form class="name-form" onSubmit={(e) => { e.preventDefault(); if (name.trim()) onDone(name); }}>
      <input ref={input} class="field" value={name} maxLength={MAX_NAME} placeholder="List name" aria-label={label}
        onInput={(e) => setName((e.currentTarget as HTMLInputElement).value)}
        onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); onDone(null); } }} />
      <button type="submit" class="btn" disabled={!name.trim()}>{submit}</button>
      <button type="button" class="btn ghost" onClick={() => onDone(null)}>Cancel</button>
    </form>
  );
}

/** Bottom sheet: every list with a checkbox (an item can be in several), and "New list". */
export function ListPicker() {
  const p = useStore(picker$);
  const d = useStore(lists);
  const [adding, setAdding] = useState(false);
  const newBtn = useRef<HTMLButtonElement>(null);
  if (!p) return null;
  const key = itemKey(p.item);
  const close = () => { setAdding(false); picker$.set(null); };
  const toggle = async (id: string, on: boolean) => {
    if (on) { await addToList(id, p.item); await setLastUsed(id); } else await removeFromList(id, key);
  };
  const create = async (name: string | null) => {
    setAdding(false);
    if (name) {
      const id = await createList(name);
      await addToList(id, p.item);
      await setLastUsed(id);
    }
    setTimeout(() => newBtn.current?.focus());
  };
  const last = targetList(d)?.id;
  return (
    <Sheet title="Save to list" onClose={close} back={p.back}>
      <p class="picker-what"><span class="hangul" lang="ko">{p.title}</span></p>
      <ul class="plain picker-lists" aria-label="Lists">
        {d.lists.map((l, i) => (
          <li key={l.id}>
            <label class="sheet-item pick">
              <input type="checkbox" data-autofocus={i === 0 ? '' : undefined} checked={inList(l, key)} onChange={(e) => toggle(l.id, (e.currentTarget as HTMLInputElement).checked)} />
              <span class="pick-name">{l.name}</span>
              {l.id === last && <span class="pick-last">tap saves here</span>}
              <span class="count" aria-label={`${l.items.length} items`}>{l.items.length}</span>
            </label>
          </li>
        ))}
      </ul>
      {adding
        ? <NameForm label="New list name" submit="Create" onDone={create} />
        : <button ref={newBtn} type="button" class="sheet-item new-list-btn" data-autofocus={d.lists.length ? undefined : ''} onClick={() => setAdding(true)}><Icon name="plus" size={20} /> <span>New list</span></button>}
      <button type="button" class="btn" onClick={close}>Done</button>
    </Sheet>
  );
}
