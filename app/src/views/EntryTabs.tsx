import { useRef } from 'preact/hooks';
import { db } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { Lru } from '../lib/cache';
import { groupResults } from '../lib/merge';
import { hanChars, normHeadword, stemOf } from '../lib/search-mode';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { packsKey } from '../lib/packs';
import type { Entry, HanjaChar, ResultRow, Sentence } from '../lib/types';
import { GroupRow, Hanja } from '../components/common';
import { entryPath, hanjaPath, href, navigate, useRoute, withTab } from '../lib/router';

export type TabId = 'dict' | 'words' | 'chars' | 'sents';
export const TABS: { id: TabId; label: string; ko: string }[] = [
  { id: 'dict', label: 'Dict', ko: '사전' },
  { id: 'words', label: 'Words', ko: '단어' },
  { id: 'chars', label: 'Chars', ko: '한자' },
  { id: 'sents', label: 'Sents', ko: '예문' },
];
const TAB_KEY = 'kd.entryTab';
const isTab = (v: unknown): v is TabId => TABS.some((t) => t.id === v);
let remembered: TabId = (() => { try { const v = localStorage.getItem(TAB_KEY); return isTab(v) ? v : 'dict'; } catch { return 'dict'; } })();

/** Active tab: explicit ?tab= wins, else the last one chosen (Pleco-style). Chars falls back to Dict without hanja. */
export function useEntryTab(hasHanja: boolean): [TabId, (t: TabId) => void] {
  const route = useRoute();
  const p = route.params.get('tab');
  let tab: TabId = isTab(p) ? p : remembered;
  if (tab === 'chars' && !hasHanja) tab = 'dict';
  const set = (t: TabId) => {
    remembered = t;
    try { localStorage.setItem(TAB_KEY, t); } catch { /* storage unavailable */ }
    navigate(withTab(route.raw, t));
  };
  return [tab, set];
}

export function TabBar({ tab, onTab, hasHanja }: { tab: TabId; onTab: (t: TabId) => void; hasHanja: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  const enabled = TABS.filter((t) => t.id !== 'chars' || hasHanja);
  const onKey = (e: KeyboardEvent) => {
    const i = enabled.findIndex((t) => t.id === tab);
    let n = -1;
    if (e.key === 'ArrowRight') n = (i + 1) % enabled.length;
    else if (e.key === 'ArrowLeft') n = (i - 1 + enabled.length) % enabled.length;
    else if (e.key === 'Home') n = 0;
    else if (e.key === 'End') n = enabled.length - 1;
    if (n < 0) return;
    e.preventDefault();
    onTab(enabled[n].id);
    requestAnimationFrame(() => ref.current?.querySelector<HTMLElement>(`#etab-${enabled[n].id}`)?.focus());
  };
  return (
    <div class="etabs" role="tablist" aria-label="Entry sections" ref={ref} onKeyDown={onKey}>
      {TABS.map((t) => {
        const off = t.id === 'chars' && !hasHanja;
        return (
          <button key={t.id} type="button" role="tab" id={`etab-${t.id}`} aria-selected={tab === t.id} aria-controls="etab-panel"
            aria-disabled={off || undefined} disabled={off} tabIndex={tab === t.id ? 0 : -1} title={t.ko} class="etab" onClick={() => onTab(t.id)}>
            {t.label}
          </button>
        );
      })}
    </div>
  );
}

const wordsCache = new Lru<ResultRow[]>(40);
const charsCache = new Lru<CharInfo[]>(60);
const sentsCache = new Lru<Sentence[]>(60);
type CharInfo = { ch: string; info: HanjaChar | null; words: ResultRow[] };
const same = (a: { source: string; id: number }, b: { source: string; id: number }) => a.source === b.source && a.id === b.id;

export function WordsTab({ primary }: { primary: Entry }) {
  const s = useStore(settings);
  const hw = primary.headword;
  const k = `${primary.source}:${primary.id}|${packsKey(s)}`;
  const r = useAsync(async () => {
    const chars = [...new Set(hanChars(primary.hanja ?? ''))];
    const [pre, ...byHanja] = await Promise.all([db.search(hw, { limit: 60 }), ...chars.map((c) => db.wordsWithHanja(c, 30, 0))]);
    const nh = normHeadword(hw);
    const contains = (pre.rows ?? []).filter((x) => normHeadword(x.headword).includes(nh));
    const rows = [...contains, ...byHanja.flat()].filter((x) => !same(x, primary) && normHeadword(x.headword) !== nh);
    // A superseded search comes back empty; don't cache that as the answer.
    if (pre.rows?.length) wordsCache.set(k, rows);
    return rows;
  }, [k], () => wordsCache.get(k));
  if (r.loading && !r.data) return <p class="muted pad">Loading…</p>;
  const groups = groupResults(r.data ?? []).slice(0, 60);
  if (!groups.length) return <p class="muted pad">No other words found.</p>;
  return <ul class="plain list flush">{groups.map((g) => <li key={g.key}><GroupRow g={g} /></li>)}</ul>;
}

export function CharsTab({ primary }: { primary: Entry }) {
  const chars = [...new Set(hanChars(primary.hanja ?? ''))];
  const k = `${primary.source}:${primary.id}`;
  const r = useAsync<CharInfo[]>(async () => {
    const v = await Promise.all(chars.map(async (ch) => ({
      ch, info: await db.hanjaChar(ch),
      words: (await db.wordsWithHanja(ch, 12, 0)).filter((w) => !same(w, primary)).slice(0, 8),
    })));
    charsCache.set(k, v); return v;
  }, [k], () => charsCache.get(k));
  if (r.loading && !r.data) return <p class="muted pad">Loading…</p>;
  return (
    <div class="chars">
      {r.data?.map(({ ch, info, words }) => (
        <section key={ch} class="char">
          <a class="hanja-big" lang="zh-Hant" href={href(hanjaPath(ch))} aria-label={`Hanja ${ch}`}>{ch}</a>
          <div class="char-body">
            <div class="char-read"><strong class="hangul" lang="ko">{info?.readings ?? '?'}</strong></div>
            {info?.meaning_en && <div class="char-mean">{info.meaning_en}</div>}
            {(info?.strokes != null || info?.radical) && (
              <div class="muted small">{info?.strokes != null ? `${info.strokes} strokes` : ''}{info?.strokes != null && info?.radical ? ' · ' : ''}{info?.radical ? <>radical <span lang="zh-Hant">{info.radical}</span></> : null}</div>
            )}
            <div class="chips">
              {words.map((w) => (
                <a key={`${w.source}:${w.id}`} class="chip" href={href(entryPath(w.source, w.id, w.headword))}>
                  <span class="hangul" lang="ko">{w.headword}</span> {w.hanja && <Hanja text={w.hanja} />}
                </a>
              ))}
              {words.length === 0 && <span class="muted small">No other words.</span>}
            </div>
          </div>
        </section>
      ))}
    </div>
  );
}

export function SentsTab({ headword }: { headword: string }) {
  const k = headword;
  const r = useAsync(async () => { const v = await db.sentences(stemOf(headword), 20); sentsCache.set(k, v); return v; }, [k], () => sentsCache.get(k));
  if (r.loading && !r.data) return <p class="muted pad">Loading…</p>;
  if (!r.data?.length) return <p class="muted pad">No example sentences found.</p>;
  return (
    <div class="sents">
      {r.data.map((x, i) => (
        <div key={i} class="ex"><div class="ex-ko hangul" lang="ko">{x.ko}</div>{x.en && <div class="ex-en">{x.en}</div>}</div>
      ))}
    </div>
  );
}
