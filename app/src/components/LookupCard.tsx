import { useEffect, useRef } from 'preact/hooks';
import { db } from '../db/client';
import { Icon } from './Icons';
import { LevelBadge, Pos } from './common';
import { useAsync } from '../lib/useAsync';
import { useStore } from '../lib/store';
import { bookmarks, addBookmark, findBookmark, removeBookmark } from '../lib/bookmarks';
import { groupResults } from '../lib/merge';
import { classOf } from '../lib/reader-text';
import { entryPath, hanjaPath, href } from '../lib/router';
import { SOURCE_TITLE, isKoSource, type HanjaChar, type ResultRow, type TextMatch } from '../lib/types';

export interface LookupState { match: TextMatch | null; loading: boolean; manual: boolean; text: string }

const EXTRA_SOURCE: Record<string, string> = { cedict: 'CC-CEDICT', zhwikt: 'Wiktionary (Chinese)', hist: '옛말' };
export const sourceTitle = (s: string) => (SOURCE_TITLE as Record<string, string>)[s] ?? EXTRA_SOURCE[s] ?? s;
const glossesOf = (rows: ResultRow[]): string[] => {
  const seen = new Set<string>(); const out: string[] = [];
  const ordered = [...rows].sort((a, b) => Number(isKoSource(a.source)) - Number(isKoSource(b.source)));
  for (const r of ordered) for (const g of (r.gloss ?? '').split(/\s*;\s*/)) { const k = g.trim(); if (k && !seen.has(k)) { seen.add(k); out.push(k); } }
  return out.slice(0, 3);
};
const isHanText = (s: string) => !!s && classOf(s.codePointAt(0)!) === 'han';

/** Per-character 훈음 for a hanja selection. */
function CharList({ text }: { text: string }) {
  const chars = [...text].filter((c) => classOf(c.codePointAt(0)!) === 'han').slice(0, 8);
  const r = useAsync(() => Promise.all(chars.map((c) => db.hanjaChar(c).catch(() => null))), [chars.join('')]);
  const info = (c: HanjaChar | null | undefined, ch: string) => (c?.eumhun || (c?.hun && c?.readings ? `${c.hun} ${c.readings}` : c?.readings) || c?.meaning_en || '');
  return (
    <ul class="plain lk-chars" aria-label="Characters">
      {chars.map((ch, i) => (
        <li key={i}><a href={href(hanjaPath(ch))} class="hanja lk-ch" lang="ko">{ch}</a><span class="lk-eum" lang="ko">{r.data ? info(r.data[i], ch) : ''}</span></li>
      ))}
    </ul>
  );
}

export function LookupCard({ st, onClose, onStep, onResize, onPick }: {
  st: LookupState; onClose: () => void;
  onStep: (what: 'prev' | 'next' | 'shrink' | 'extend') => void; onResize?: (h: number) => void;
  onPick?: (i: number) => void;
}) {
  const bm = useStore(bookmarks);
  const el = useRef<HTMLDivElement>(null);
  const m = st.match;
  const groups = m ? groupResults(m.rows) : [];
  const top = groups[0];
  const han = !!m && isHanText(m.match);
  const cedict = m?.rows.filter((r) => r.source === 'cedict' || r.source === 'zhwikt') ?? [];
  const koRows = top ? top.rows.filter((r) => r.source !== 'cedict' && r.source !== 'zhwikt') : [];
  const glosses = glossesOf(koRows.length ? koRows : top?.rows ?? []);
  const primary = top?.primary;
  const saved = primary ? findBookmark(bm, { source: primary.source, headword: primary.headword, homonym: primary.homonym, pos: primary.pos }) : undefined;
  const head = han ? m!.match : top?.headword ?? m?.match ?? '';
  const sub = han && top ? top.headword : undefined;

  // swipe down to dismiss
  const drag = useRef<{ y: number; dy: number } | null>(null);
  const onDown = (e: PointerEvent) => { if ((e.target as HTMLElement).closest('button, a')) return; drag.current = { y: e.clientY, dy: 0 }; (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId); };
  const onMove = (e: PointerEvent) => { const d = drag.current; if (!d || !el.current) return; d.dy = Math.max(0, e.clientY - d.y); el.current.style.transform = `translateY(${d.dy}px)`; };
  const onUp = () => { const d = drag.current; drag.current = null; if (!el.current) return; if (d && d.dy > 70) onClose(); else el.current.style.transform = ''; };
  useEffect(() => { if (el.current && onResize) onResize(el.current.offsetHeight); });

  return (
    <div ref={el} class="lk" role="dialog" aria-label="Dictionary lookup" data-loading={st.loading ? '1' : '0'}
      onPointerDown={onDown} onPointerMove={onMove} onPointerUp={onUp} onPointerCancel={onUp}>
      <div class="lk-grip" aria-hidden="true" />
      <div class="lk-top">
        <div class="lk-head">
          <span class={han ? 'hanja lk-hw' : 'hangul lk-hw'} lang="ko">{head || '…'}</span>
          {sub && <span class="hangul lk-sub" lang="ko">{sub}</span>}
          {!han && top?.hanja && <span class="hanja lk-hj" lang="zh-Hant">{top.hanja}</span>}
        </div>
        {primary && <button type="button" class={`iconbtn${saved ? ' on' : ''}`} aria-label={saved ? 'Remove bookmark' : 'Add bookmark'} aria-pressed={!!saved}
          onClick={() => (saved ? removeBookmark(saved.key) : addBookmark({ source: primary.source, headword: primary.headword, homonym: primary.homonym, pos: primary.pos, hanja: primary.hanja, gloss: primary.gloss }))}><Icon name="star" fill={!!saved} /></button>}
        <button type="button" class="iconbtn" aria-label="Close lookup" onClick={onClose}><Icon name="x" /></button>
      </div>
      <div class="lk-body">
        {st.loading && !m && <div class="muted small">Looking up…</div>}
        {m && (
          <>
            <div class="lk-meta">
              {top?.rows[0]?.pron && <span class="pron" lang="ko">[{top.rows[0].pron}]</span>}
              {top && <Pos pos={top.pos.join(' · ')} />}
              {top && <LevelBadge level={top.level} />}
              {m.rows[0]?.via && m.rows[0].via !== 'exact' && <span class="lk-via">{m.rows[0].via === 'deconj' ? `form of ${top?.headword ?? ''}` : m.rows[0].via}</span>}
              {st.manual && m.match && top && m.match !== top.headword && <span class="lk-via">matched {m.match}</span>}
            </div>
            {glosses.length > 0 && <ol class="plain lk-gloss">{glosses.map((g, i) => <li key={i}>{g}</li>)}</ol>}
            {!glosses.length && top && <div class="muted small">Korean definition only — open the full entry.</div>}
            {han && cedict[0]?.gloss && <div class="lk-cedict"><span class="lk-tag">{sourceTitle(cedict[0].source)}</span> {cedict[0].gloss}</div>}
            {han && <CharList text={m.match} />}
            {han && m.hanja && (m.hanja.eumhun || m.hanja.readings || m.hanja.meaning_en) && !top && <div class="lk-cedict" lang="ko">{[m.hanja.eumhun, m.hanja.readings, m.hanja.meaning_en].filter(Boolean).join(' · ')}</div>}
            {!m.rows.length && !m.hanja && !st.loading && <div class="muted small">No dictionary entry for “{m.match}”. Try ⇤ ⇥ to change the selection.</div>}
            {groups.length > 1 && (
              <div class="lk-alts" aria-label="Other matches">
                {groups.slice(1, 5).map((g, i) => <button key={g.key} type="button" class="chip" onClick={() => onPick?.(i + 1)}><span class="hangul" lang="ko">{g.headword}</span></button>)}
              </div>
            )}
            {top && <div class="lk-src small muted">{top.sources.map(sourceTitle).join(' · ')}</div>}
          </>
        )}
      </div>
      <div class="lk-bar">
        <div class="lk-nav" role="group" aria-label="Move or resize selection">
          <button type="button" aria-label="Shrink selection by one character" title="Shrink by a character" onClick={() => onStep('shrink')}>⇤</button>
          <button type="button" aria-label="Previous word" title="Previous word" onClick={() => onStep('prev')}>◀</button>
          <button type="button" aria-label="Next word" title="Next word" onClick={() => onStep('next')}>▶</button>
          <button type="button" aria-label="Extend selection by one character" title="Extend by a character" onClick={() => onStep('extend')}>⇥</button>
        </div>
        {primary
          ? <a class="btn lk-open" href={href(entryPath(primary.source, primary.id, primary.headword))}>Open full entry</a>
          : m?.hanja ? <a class="btn lk-open" href={href(hanjaPath(m.hanja.ch))}>Open character</a> : null}
      </div>
    </div>
  );
}
