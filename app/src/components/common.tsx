import { useEffect, useRef } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { entryPath, href } from '../lib/router';
import type { ResultGroup } from '../lib/merge';
import { hanjaKey } from '../lib/hanja-marks';

export const LEVEL_LABEL: Record<number, string> = { 1: '초', 2: '중', 3: '고' };
export const LEVEL_NAME: Record<number, string> = { 1: 'Beginner', 2: 'Intermediate', 3: 'Advanced' };

export function LevelBadge({ level }: { level: number | null | undefined }) {
  if (!level || !LEVEL_LABEL[level]) return null;
  return <span class={`lvl lvl${level}`} title={LEVEL_NAME[level]} aria-label={LEVEL_NAME[level]}>{LEVEL_LABEL[level]}</span>;
}
export const Pos = ({ pos }: { pos?: string | null }) => (pos ? <span class="pos">{pos}</span> : null);

/** Hanja without the dictionary's reading marks (困難▽ -> 困難; the entry page spells them out). */
export function Hanja({ text }: { text: string }) {
  return <span class="hanja" lang="zh-Hant">{hanjaKey(text)}</span>;
}

export function GroupRow({ g, note, current }: { g: ResultGroup; note?: string; current?: boolean }) {
  const p = g.primary;
  return (
    <a class="row" href={href(entryPath(p.source, p.id, g.headword))} aria-current={current ? 'true' : undefined}>
      <div class="row-main">
        <div class="row-head">
          <span class="hangul hw" lang="ko">{g.headword}</span>
          {g.hanja && <Hanja text={g.hanja} />}
          <Pos pos={g.pos.join(' · ')} />
          <LevelBadge level={g.level} />
        </div>
        <div class="row-gloss">{g.gloss || <span class="muted">Korean definition only</span>}</div>
        {note && <div class="row-note">{note}</div>}
      </div>
    </a>
  );
}

export function Empty({ title, children }: { title: string; children?: ComponentChildren }) {
  return <div class="empty"><div class="empty-title">{title}</div>{children && <div class="muted">{children}</div>}</div>;
}

export function IconButton({ icon, label, onClick, active, size }: { icon: string; label: string; onClick: () => void; active?: boolean; size?: number }) {
  return <button type="button" class={`iconbtn${active ? ' on' : ''}`} aria-label={label} title={label} onClick={onClick}><Icon name={icon} size={size} fill={active && icon === 'star'} /></button>;
}

/** Modal bottom sheet: focus moves into it (to the `data-autofocus` element, else the sheet), stays in it (Tab wraps), Escape closes it, and focus goes
 *  back to `back` (or what had it before) when it closes. */
export function Sheet({ title, onClose, children, back }: { title: string; onClose: () => void; children: ComponentChildren; back?: HTMLElement | null }) {
  const el = useRef<HTMLDivElement>(null);
  const close = useRef(onClose);
  close.current = onClose;
  useEffect(() => {
    const prev = back ?? (document.activeElement as HTMLElement | null);
    (el.current?.querySelector<HTMLElement>('[data-autofocus]') ?? el.current)?.focus();
    return () => { if (prev?.isConnected) prev.focus(); };
  }, []);
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') { e.preventDefault(); close.current(); return; }
    if (e.key !== 'Tab' || !el.current) return;
    const f = [...el.current.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), a[href], [tabindex]:not([tabindex="-1"])')];
    if (!f.length) return;
    const i = f.indexOf(document.activeElement as HTMLElement);
    if (e.shiftKey && i <= 0) { e.preventDefault(); f[f.length - 1].focus(); }
    else if (!e.shiftKey && i === f.length - 1) { e.preventDefault(); f[0].focus(); }
  };
  return (
    <div class="sheet-wrap" onClick={onClose}>
      <div ref={el} class="sheet" role="dialog" aria-modal="true" aria-label={title} tabIndex={-1} onKeyDown={onKey} onClick={(e) => e.stopPropagation()}>
        <div class="sheet-head"><strong>{title}</strong><IconButton icon="x" label="Close" onClick={onClose} /></div>
        <div class="sheet-body">{children}</div>
      </div>
    </div>
  );
}
