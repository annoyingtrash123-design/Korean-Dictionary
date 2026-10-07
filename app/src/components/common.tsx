import type { ComponentChildren } from 'preact';
import { Icon } from './Icons';
import { entryPath, href } from '../lib/router';
import type { ResultGroup } from '../lib/merge';
import { SOURCE_SHORT } from '../lib/types';

export const LEVEL_LABEL: Record<number, string> = { 1: '초', 2: '중', 3: '고' };
export const LEVEL_NAME: Record<number, string> = { 1: 'Beginner', 2: 'Intermediate', 3: 'Advanced' };

export function LevelBadge({ level }: { level: number | null | undefined }) {
  if (!level || !LEVEL_LABEL[level]) return null;
  return <span class={`lvl lvl${level}`} title={LEVEL_NAME[level]} aria-label={LEVEL_NAME[level]}>{LEVEL_LABEL[level]}</span>;
}
export const Pos = ({ pos }: { pos?: string | null }) => (pos ? <span class="pos">{pos}</span> : null);

export function Hanja({ text }: { text: string }) {
  return <span class="hanja" lang="zh-Hant">{text}</span>;
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
        <div class="row-src">{g.sources.map((s) => SOURCE_SHORT[s]).join(' · ')}</div>
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

export function Sheet({ title, onClose, children }: { title: string; onClose: () => void; children: ComponentChildren }) {
  return (
    <div class="sheet-wrap" onClick={onClose}>
      <div class="sheet" role="dialog" aria-label={title} onClick={(e) => e.stopPropagation()}>
        <div class="sheet-head"><strong>{title}</strong><IconButton icon="x" label="Close" onClick={onClose} /></div>
        {children}
      </div>
    </div>
  );
}
