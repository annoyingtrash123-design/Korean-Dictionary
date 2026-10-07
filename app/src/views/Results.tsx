import { useEffect, useMemo, useState } from 'preact/hooks';
import { db, packStatus$ } from '../db/client';
import { groupResults } from '../lib/merge';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { normHeadword } from '../lib/search-mode';
import type { SearchResult } from '../lib/types';
import { Empty, GroupRow } from '../components/common';
import { entryPath, hanjaPath, href, searchPath, wordPath } from '../lib/router';

export function Results({ q }: { q: string }) {
  const s = useStore(settings);
  const status = useStore(packStatus$);
  const [res, setRes] = useState<SearchResult>();
  const [err, setErr] = useState<string>();
  const [busy, setBusy] = useState(true);
  const stamp = JSON.stringify(Object.values(status?.packs ?? {}).map((p) => p.version));

  useEffect(() => {
    let live = true;
    setBusy(true);
    db.search(q, { stdict: s.stdict, limit: 50 }).then(
      (r) => { if (live) { setRes(r); setErr(undefined); setBusy(false); } },
      (e) => { if (live) { setErr(String(e?.message ?? e)); setBusy(false); } });
    return () => { live = false; };
  }, [q, s.stdict, stamp]);

  const groups = useMemo(() => groupResults(res?.rows ?? []), [res]);
  const ruleFor = (hw: string) => res?.deconj?.find((d) => normHeadword(d.lemma) === normHeadword(hw))?.rule;
  const deconjGroups = groups.filter((g) => g.via === 'deconj' && !groups.some((o) => o !== g && o.via === 'exact' && o.key === g.key)).slice(0, 2);

  if (err) return <div class="page"><Empty title="Search failed">{err}</Empty></div>;
  return (
    <div class={`page${busy ? ' busy' : ''}`}>
      {res?.hanja?.map((h) => (
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
      {res?.grammarHints?.length ? (
        <div class="notice">{res.grammarHints.map((h) => <div key={h}><a href={href(`/grammar?q=${encodeURIComponent(h.split(/[:\s(]/)[0])}`)}>{h}</a></div>)}</div>
      ) : null}
      {!busy && groups.length === 0 && !res?.hanja?.length && (
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
export { entryPath, searchPath };
