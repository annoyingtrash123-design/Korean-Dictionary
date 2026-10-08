import { packsKey } from '../lib/packs';
import { useEffect, useMemo, useState } from 'preact/hooks';
import { Lru } from '../lib/cache';
import { useDelayed } from '../lib/useAsync';
import { db, otherWindowInstalling, packStatus$, refreshStatus, restartWorker } from '../db/client';
import { groupResults } from '../lib/merge';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { normHeadword } from '../lib/search-mode';
import type { SearchResult } from '../lib/types';
import { Empty, GroupRow } from '../components/common';
import { EnKoPhrases, EnKoTranslations } from '../components/EnKo';
import { entryPath, hanjaPath, href, useRoute, withoutTab, wordPath } from '../lib/router';

const cache = new Lru<SearchResult>(40);

/** English searches: Korean results grouped by part of speech (nouns, verbs, …), in rank order. */
const POS_BUCKETS: { id: string; label: string; ko: string; pos: string[] }[] = [
  { id: 'noun', label: 'Nouns', ko: '명사', pos: ['noun', 'bound noun', 'pronoun', 'numeral'] },
  { id: 'verb', label: 'Verbs', ko: '동사', pos: ['verb', 'auxiliary verb'] },
  { id: 'adj', label: 'Adjectives', ko: '형용사·관형사', pos: ['adjective', 'auxiliary adjective', 'determiner'] },
  { id: 'adv', label: 'Adverbs', ko: '부사', pos: ['adverb'] },
  { id: 'phrase', label: 'Phrases & idioms', ko: '구·관용구', pos: ['phrase', 'idiom', 'expression', 'proverb'] },
];
export function posBucket(pos: string | undefined): string {
  const p = (pos ?? '').trim().toLowerCase();
  return POS_BUCKETS.find((b) => b.pos.includes(p))?.id ?? 'other';
}
export function byPos<T extends { pos: string[] }>(groups: T[]): { id: string; label: string; ko: string; items: T[] }[] {
  const out: { id: string; label: string; ko: string; items: T[] }[] = [];
  for (const g of groups) {
    const id = posBucket(g.pos[0]);
    let b = out.find((x) => x.id === id);
    if (!b) {
      const def = POS_BUCKETS.find((x) => x.id === id) ?? { id: 'other', label: 'Other', ko: '기타', pos: [] };
      b = { id, label: def.label, ko: def.ko, items: [] };
      out.push(b);
    }
    b.items.push(g);
  }
  return out;
}
const FIRST_PAINT = 15;
export const FIRST_PAGE = 20;
const FULL_PAGE = 50;

export function Results({ q }: { q: string }) {
  const s = useStore(settings);
  const route = useRoute();
  const status = useStore(packStatus$);
  const stamp = JSON.stringify(Object.values(status?.packs ?? {}).map((p) => p.version));
  const ck = (qq: string) => `${qq}|${packsKey(s)}|${stamp}`;
  const [res, setRes0] = useState<{ q: string; r: SearchResult } | undefined>(() => { const c = cache.get(ck(q)); return c ? { q, r: c } : undefined; });
  const setRes = (r: SearchResult) => setRes0({ q, r });
  const [err, setErr] = useState<string>();
  const [stampBump, setStampBump] = useState(0);
  const [busy, setBusy] = useState(() => !cache.get(ck(q)));
  const slow = useDelayed(busy, 150);

  useEffect(() => {
    let live = true;
    const hit = cache.get(ck(q));
    if (hit) { setRes(hit); setErr(undefined); setBusy(false); return; }
    setBusy(true);
    // A small first page keeps the engine to ~20 page reads per keystroke; the full list follows
    // once the first screenful is painted (by then its pages are cached).
    const done = (r: SearchResult, final: boolean) => {
      if (!live || r.rows === undefined) return;
      if (final && (r.rows.length || r.mode !== 'english' || r.hanja || r.translations || r.phrases)) cache.set(ck(q), r);
      setRes(r); setErr(undefined); setBusy(false);
    };
    const fail = (e: any) => { if (live) { setErr(String(e?.message ?? e)); setBusy(false); } };
    db.search(q, { limit: FIRST_PAGE }).then((r) => {
      done(r, r.rows !== undefined && r.rows.length < FIRST_PAGE);
      if (live && r.rows?.length >= FIRST_PAGE) setTimeout(() => { if (live) db.search(q, { limit: FULL_PAGE }).then((r2) => done(r2, true), fail); }, 0);
    }, fail);
    return () => { live = false; };
  }, [q, packsKey(s), stamp, stampBump]);

  const shown = res?.r;
  const groups = useMemo(() => groupResults(shown?.rows ?? []), [shown]);
  const ruleFor = (hw: string) => shown?.deconj?.find((d) => normHeadword(d.lemma) === normHeadword(hw))?.rule;
  // Paint the first screenful straight away and the rest a frame later, so a keystroke never
  // waits on laying out 50 rows.
  const [full, setFull] = useState(false);
  useEffect(() => {
    setFull(false);
    const t = setTimeout(() => setFull(true), 30);
    return () => clearTimeout(t);
  }, [shown]);
  const visible = full ? groups : groups.slice(0, FIRST_PAINT);
  const deconjGroups = groups.filter((g) => g.via === 'deconj' && !groups.some((o) => o !== g && o.via === 'exact' && o.key === g.key)).slice(0, 2);

  if (err) {
    const locked = /not initialised|Worker restarted|NoModificationAllowed|access handle|locked/i.test(err);
    return (
      <div class="page">
        <Empty title={locked ? 'Dictionary is busy in another window' : 'Search failed'}>
          {locked ? (otherWindowInstalling.get() ? 'The dictionary is still being installed in another tab or app window. Wait for it to finish (or close it), then tap Retry.' : 'The dictionary is open in another tab or app window. Close that one, then tap Retry.') : err}
        </Empty>
        <div class="center"><button type="button" class="btn primary" onClick={() => { otherWindowInstalling.set(false); cache.clear(); restartWorker(); refreshStatus().catch(() => undefined).finally(() => setStampBump((n) => n + 1)); }}>Retry</button></div>
      </div>
    );
  }
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
      {shown?.translations?.length ? <EnKoTranslations senses={shown.translations} /> : null}
      {deconjGroups.length > 0 && (
        <div class="notice">
          Did you mean:{' '}
          {deconjGroups.map((g, i) => (
            <span key={g.key}>{i > 0 && ', '}<a href={href(wordPath(g.headword))} class="hangul">{g.headword}</a>{ruleFor(g.headword) ? <span class="muted"> ({ruleFor(g.headword)})</span> : null}</span>
          ))}
        </div>
      )}
      {!busy && groups.length === 0 && !shown?.hanja?.length && !shown?.translations?.length && !shown?.phrases?.length && (
        <Empty title={`No results for “${q}”`}>Try another spelling, the dictionary form, or an English word.</Empty>
      )}
      {(() => {
        const row = (g: (typeof visible)[number]) => (
          <li key={g.key}><GroupRow g={g} current={withoutTab(route.raw) === entryPath(g.primary.source, g.primary.id, g.headword)} note={g.via === 'deconj' ? `← ${q}${ruleFor(g.headword) ? ` · ${ruleFor(g.headword)}` : ''}` : g.via === 'form' ? `form: ${q}` : undefined} /></li>
        );
        const sections = shown?.mode === 'english' ? byPos(visible) : [];
        if (sections.length < 2) return <ul class="plain list">{visible.map(row)}</ul>;
        return sections.map((sec) => (
          <section key={sec.id} class="pos-sec" aria-label={sec.label}>
            <h2 class="pos-h"><span>{sec.label}</span><span class="pos-ko" lang="ko">{sec.ko}</span><span class="pos-n">{sec.items.length}</span></h2>
            <ul class="plain list">{sec.items.map(row)}</ul>
          </section>
        ));
      })()}
      {shown?.phrases?.length ? <EnKoPhrases key={res?.q} phrases={shown.phrases} /> : null}
    </div>
  );
}
