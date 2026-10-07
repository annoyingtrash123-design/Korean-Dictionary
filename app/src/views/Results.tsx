import { packsKey } from '../lib/packs';
import { useEffect, useMemo, useState } from 'preact/hooks';
import { Lru } from '../lib/cache';
import { useDelayed } from '../lib/useAsync';
import { db, packStatus$ } from '../db/client';
import { groupResults } from '../lib/merge';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { normHeadword } from '../lib/search-mode';
import type { SearchResult } from '../lib/types';
import { Empty, GroupRow } from '../components/common';
import { hanjaPath, href, wordPath } from '../lib/router';

const cache = new Lru<SearchResult>(40);

export function Results({ q }: { q: string }) {
  const s = useStore(settings);
  const status = useStore(packStatus$);
  const stamp = JSON.stringify(Object.values(status?.packs ?? {}).map((p) => p.version));
  const ck = (qq: string) => `${qq}|${packsKey(s)}|${stamp}`;
  const [res, setRes0] = useState<{ q: string; r: SearchResult } | undefined>(() => { const c = cache.get(ck(q)); return c ? { q, r: c } : undefined; });
  const setRes = (r: SearchResult) => setRes0({ q, r });
  const [err, setErr] = useState<string>();
  const [busy, setBusy] = useState(() => !cache.get(ck(q)));
  const slow = useDelayed(busy, 150);

  useEffect(() => {
    let live = true;
    const hit = cache.get(ck(q));
    if (hit) { setRes(hit); setErr(undefined); setBusy(false); return; }
    setBusy(true);
    db.search(q, { limit: 50 }).then(
      (r) => { if (live && r.rows !== undefined) { if (r.rows.length || r.mode !== 'english' || r.hanja) cache.set(ck(q), r); setRes(r); setErr(undefined); setBusy(false); } },
      (e) => { if (live) { setErr(String(e?.message ?? e)); setBusy(false); } });
    return () => { live = false; };
  }, [q, packsKey(s), stamp]);

  const shown = res?.r;
  const groups = useMemo(() => groupResults(shown?.rows ?? []), [shown]);
  const ruleFor = (hw: string) => shown?.deconj?.find((d) => normHeadword(d.lemma) === normHeadword(hw))?.rule;
  const deconjGroups = groups.filter((g) => g.via === 'deconj' && !groups.some((o) => o !== g && o.via === 'exact' && o.key === g.key)).slice(0, 2);

  if (err) return <div class="page"><Empty title="Search failed">{err}</Empty></div>;
  return (
    <div class={`page${slow ? ' busy' : ''}`} data-q={res?.q} data-busy={busy ? 1 : 0}>
      {shown?.hanja?.map((h) => (
        <a key={h.ch} class="hanja-card" href={href(hanjaPath(h.ch))}>
          <span class="hanja-big" lang="zh-Hant">{h.ch}</span>
          <div class="hanja-card-body">
            <div><strong class="hangul">{h.readings}</strong> <span class="muted">{h.strokes ? `${h.strokes} strokes` : ''}{h.radical ? ` · radical ${h.radical}` : ''}</span></div>
            <div class="row-gloss">{h.meaning_en}</div>
          </div>
        </a>
      ))}
      {deconjGroups.length > 0 && (
        <div class="notice">
          Did you mean:{' '}
          {deconjGroups.map((g, i) => (
            <span key={g.key}>{i > 0 && ', '}<a href={href(wordPath(g.headword))} class="hangul">{g.headword}</a>{ruleFor(g.headword) ? <span class="muted"> ({ruleFor(g.headword)})</span> : null}</span>
          ))}
        </div>
      )}
      {shown?.grammarHints?.length ? (
        <div class="notice">{shown.grammarHints.map((h) => <div key={h}><a href={href(`/grammar?q=${encodeURIComponent(h.split(/[:\s(]/)[0])}`)}>{h}</a></div>)}</div>
      ) : null}
      {!busy && groups.length === 0 && !shown?.hanja?.length && (
        <Empty title={`No results for “${q}”`}>Try another spelling, the dictionary form, or an English word.</Empty>
      )}
      <ul class="plain list">
        {groups.map((g) => (
          <li key={g.key}><GroupRow g={g} note={g.via === 'deconj' ? `← ${q}${ruleFor(g.headword) ? ` · ${ruleFor(g.headword)}` : ''}` : g.via === 'form' ? `form: ${q}` : undefined} /></li>
        ))}
      </ul>
    </div>
  );
}
