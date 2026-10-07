import { packsKey } from '../lib/packs';
import { useEffect, useState } from 'preact/hooks';
import { db, packStatus$ } from '../db/client';
import { useAsync, useDelayed } from '../lib/useAsync';
import { Lru } from '../lib/cache';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { bookmarks, findBookmark, addBookmark, removeBookmark, moveBookmark } from '../lib/bookmarks';
import { resolveEntry, type EntryRef } from '../lib/entry-key';
import { recordHistory } from '../lib/history';
import { sameWordRows } from '../lib/merge';
import { hanChars } from '../lib/search-mode';
import { TabBar, WordsTab, CharsTab, SentsTab, useEntryTab } from './EntryTabs';
import { SOURCE_ORDER, SOURCE_TITLE, type Entry, type Sense } from '../lib/types';
import { Empty, IconButton, LevelBadge, Pos, Sheet } from '../components/common';
import { Icon } from '../components/Icons';
import { hanjaPath, href, searchPath, wordPath } from '../lib/router';
import { useWide } from '../lib/layout';

type Loaded = { primary: Entry | null; entries: Entry[] };
const viewCache = new Lru<Loaded>(40);

export function EntryView({ source, id, hw, word, pref }: { source?: string; id?: number; hw?: string; word?: string; pref?: Partial<EntryRef> }) {
  const s = useStore(settings);
  const status = useStore(packStatus$);
  const stamp = JSON.stringify(Object.values(status?.packs ?? {}).map((p) => p.version));
  const ck = `${source}:${id}:${word}:${hw}:${pref ? JSON.stringify(pref) : ''}|${packsKey(s)}|${stamp}`;
  const r = useAsync(async (): Promise<Loaded> => {
    let primary: Entry | null = null;
    let all: Entry[] = [];
    if (source && id != null) {
      // headword comes from the link (?hw=) so both lookups run together
      const [p, a] = await Promise.all([db.getEntry(source, id), hw ? db.getEntriesByHeadword(hw) : Promise.resolve(null)]);
      primary = p;
      all = a ?? (primary ? await db.getEntriesByHeadword(primary.headword) : []);
    } else if (word) {
      all = await db.getEntriesByHeadword(word);
      primary = pref?.source ? resolveEntry(all, pref) : [...all].filter((e) => e.lang !== "ko").sort((a, b) => a.rank - b.rank)[0] ?? all[0] ?? null;
    }
    if (!primary && hw) all = await db.getEntriesByHeadword(hw);
    let out: Loaded;
    if (!primary) out = { primary: null, entries: all };
    else {
      const entries = sameWordRows(all, primary);
      if (!entries.some((e) => e.source === primary!.source && e.id === primary!.id)) entries.push(primary);
      out = { primary, entries };
    }
    viewCache.set(ck, out);
    return out;
  }, [source, id, word, pref?.source, pref?.homonym, pref?.pos, packsKey(s), stamp], () => viewCache.get(ck));

  const primary = r.data?.primary;
  const hasHanja = !!primary?.hanja && hanChars(primary.hanja).length > 0;
  const [tab, setTab] = useEntryTab(hasHanja);
  useEffect(() => {
    if (primary) recordHistory({ source: primary.source, headword: primary.headword, homonym: primary.homonym, pos: primary.pos, hanja: primary.hanja, gloss: primary.gloss });
  }, [primary?.source, primary?.id]);

  const showSkel = useDelayed(r.loading && !r.data, 150);
  if (r.loading && !r.data) return <div class="page">{showSkel && <div class="skeleton" />}</div>;
  if (r.error) return <div class="page"><Empty title="Could not load entry">{r.error}</Empty></div>;
  if (!primary) {
    const t = hw || word || '';
    return <div class="page"><Empty title="Entry not found">{t ? <a href={href(searchPath(t))}>Search for “{t}”</a> : 'It may have been removed in a dictionary update.'}</Empty></div>;
  }
  const entries = r.data!.entries;
  const bySource = SOURCE_ORDER.map((src) => ({ src, list: entries.filter((e) => e.source === src) })).filter((x) => x.list.length);
  return (
    <article class="page entry" style={{ fontSize: 'var(--entry-font-size)' }}>
      <Header primary={primary} entries={entries} />
      <TabBar tab={tab} onTab={setTab} hasHanja={hasHanja} />
      <div class="etab-panel" role="tabpanel" id="etab-panel" aria-labelledby={`etab-${tab}`}>
        {tab === 'dict' && bySource.map(({ src, list }) => (
          <details key={src} class="dict" open>
            <summary><span>{SOURCE_TITLE[src]}</span><Icon name="down" size={18} /></summary>
            {list.map((e) => <EntryBody key={e.id} e={e} many={list.length > 1} showKo={s.showKoDef || e.lang === 'ko'} />)}
          </details>
        ))}
        {tab === 'words' && <WordsTab primary={primary} />}
        {tab === 'chars' && <CharsTab primary={primary} />}
        {tab === 'sents' && <SentsTab headword={primary.headword} />}
      </div>
    </article>
  );
}

function Header({ primary, entries }: { primary: Entry; entries: Entry[] }) {
  const bm = useStore(bookmarks);
  const [sheet, setSheet] = useState(false);
  const marked = !!findBookmark(bm, primary);
  const pron = entries.find((e) => e.pron)?.pron;
  const level = entries.map((e) => e.level).find((l) => l != null) ?? null;
  const hom = entries.filter((e) => e.source === primary.source).length > 1 ? undefined : primary.homonym;
  return (
    <header class="entry-head">
      <div class="entry-title">
        <h1 class="hangul" lang="ko">{primary.headword}{hom ? <sup>{hom}</sup> : null}</h1>
        <IconButton icon="star" label={marked ? 'Edit bookmark' : 'Add bookmark'} active={marked} onClick={() => setSheet(true)} size={26} />
      </div>
      <div class="entry-meta">
        {primary.hanja && (
          <span class="hanja-chars">
            {[...primary.hanja].map((c, i) => /[㐀-鿿豈-﫿]/.test(c)
              ? <a key={i} class="hanja hj-link" lang="zh-Hant" href={href(hanjaPath(c))} aria-label={`Hanja ${c}`}>{c}</a>
              : <span key={i} class="hanja">{c}</span>)}
          </span>
        )}
        {pron && <span class="pron">[{pron}]</span>}
        <Pos pos={primary.pos} />
        <LevelBadge level={level} />
        {primary.kind !== 'word' && <span class="pos">{primary.kind}</span>}
      </div>
      {sheet && (
        <FolderSheet marked={marked} bm={bm} entry={primary} onClose={() => setSheet(false)} />
      )}
    </header>
  );
}

function FolderSheet({ marked, bm, entry, onClose }: { marked: boolean; bm: ReturnType<typeof bookmarks.get>; entry: Entry; onClose: () => void }) {
  const cur = findBookmark(bm, entry);
  const pick = async (folder: string) => {
    if (cur) await moveBookmark(cur.key, folder);
    else await addBookmark({ source: entry.source, headword: entry.headword, homonym: entry.homonym, pos: entry.pos, hanja: entry.hanja, gloss: entry.gloss, folder });
    onClose();
  };
  return (
    <Sheet title={marked ? 'Bookmarked in…' : 'Save to…'} onClose={onClose}>
      <ul class="plain">
        {bm.folders.map((f) => (
          <li key={f.id}>
            <button type="button" class="sheet-item" onClick={() => pick(f.id)}>
              <Icon name="folder" size={20} /> <span>{f.name}</span>
              {cur?.folder === f.id && <span class="check" aria-label="Current folder">✓</span>}
            </button>
          </li>
        ))}
      </ul>
      {marked && <button type="button" class="btn danger" onClick={async () => { await removeBookmark(cur!.key); onClose(); }}>Remove bookmark</button>}
    </Sheet>
  );
}

const RELTYPE: Record<string, string> = { synonym: 'syn', antonym: 'ant', honorific: 'hon', humble: 'hum', 'see also': 'see', reference: 'ref', derived: 'der', variant: 'var', abbreviation: 'abbr' };
function Chips({ rels }: { rels?: { type: string; word: string }[] }) {
  const wide = useWide();
  if (!rels?.length) return null;
  return (
    <div class="chips">
      {rels.map((r, i) => (
        <a key={i} class="chip" href={href(wide ? wordPath(r.word) : searchPath(r.word))}><span class="chip-type">{RELTYPE[r.type] ?? r.type}</span> <span class="hangul" lang="ko">{r.word}</span></a>
      ))}
    </div>
  );
}

function SenseView({ s, n, showKo, isKo }: { s: Sense; n: number; showKo: boolean; isKo: boolean }) {
  const [all, setAll] = useState(false);
  const exs = s.examples ?? [];
  const shownEx = all || exs.length <= 4 ? exs : exs.slice(0, 3);
  return (
    <li class="sense">
      <div class="sense-body">
        {s.pos && <Pos pos={s.pos} />}
        {s.gloss ? <span class="gloss">{s.gloss}</span> : s.roman ? <span class="roman">{s.roman}</span> : null}
        {s.tags?.length ? <span class="tags">{s.tags.map((t) => <span key={t} class="tag">{t}</span>)}</span> : null}
        {s.def && <div class="def">{s.def}</div>}
        {s.ko_def && (showKo || isKo) && <div class={isKo ? 'def ko-main hangul' : 'def ko'} lang="ko">{s.ko_def}</div>}
        {s.note && <div class="note">{s.note}</div>}
        {s.pattern && <div class="pattern" lang="ko">{s.pattern}</div>}
        {shownEx.map((x, i) => (
          <div key={i} class="ex">
            {x.ko.split('\n').map((l, j) => <div key={j} class="ex-ko hangul" lang="ko">{l}</div>)}
            {x.en && x.en.split('\n').map((l, j) => <div key={j} class="ex-en">{l}</div>)}
          </div>
        ))}
        {exs.length > shownEx.length && <button type="button" class="link" onClick={() => setAll(true)}>Show {exs.length - shownEx.length} more examples</button>}
        <Chips rels={s.rel} />
      </div>
      <span class="sense-n" aria-hidden="true">{n}</span>
    </li>
  );
}

function EntryBody({ e, many, showKo }: { e: Entry; many: boolean; showKo: boolean }) {
  const senses = e.data?.senses ?? [];
  const isKo = e.lang === 'ko';
  return (
    <section class="body">
      {many && (
        <div class="body-head">
          <span class="hangul" lang="ko">{e.headword}</span>{e.homonym ? <sup>{e.homonym}</sup> : null}
          <Pos pos={e.pos} />{e.gloss ? <span class="muted"> {e.gloss}</span> : null}
        </div>
      )}
      <ol class="senses">{senses.map((s, i) => <SenseView key={i} s={s} n={i + 1} showKo={showKo} isKo={isKo} />)}</ol>
      {e.data?.category && <div class="muted small">Category: {e.data.category}</div>}
      {e.data?.etym && <div class="note">{e.data.etym}</div>}
      {e.data?.origin_note && <div class="note">{e.data.origin_note}</div>}
      <Chips rels={e.data?.related} />
    </section>
  );
}
