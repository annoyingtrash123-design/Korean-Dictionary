import { packsKey } from '../lib/packs';
import { useEffect, useState } from 'preact/hooks';
import { db, packStatus$ } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { settings } from '../lib/settings';
import { useStore } from '../lib/store';
import { bookmarks, isBookmarked, addBookmark, removeBookmark, moveBookmark } from '../lib/bookmarks';
import { recordHistory } from '../lib/history';
import { sameWordRows } from '../lib/merge';
import { hanChars } from '../lib/search-mode';
import { stemOf } from '../lib/search-mode';
import { SOURCE_ORDER, SOURCE_TITLE, type Entry, type Sense } from '../lib/types';
import { Empty, Hanja, IconButton, LevelBadge, Pos, Sheet } from '../components/common';
import { Icon } from '../components/Icons';
import { entryPath, hanjaPath, href, searchPath } from '../lib/router';

export function EntryView({ source, id, hw, word }: { source?: string; id?: number; hw?: string; word?: string }) {
  const s = useStore(settings);
  const status = useStore(packStatus$);
  const stamp = JSON.stringify(Object.values(status?.packs ?? {}).map((p) => p.version));
  const r = useAsync(async () => {
    let primary: Entry | null = null;
    let all: Entry[] = [];
    if (source && id != null) {
      primary = await db.getEntry(source, id);
      if (primary) all = await db.getEntriesByHeadword(primary.headword);
    } else if (word) {
      all = await db.getEntriesByHeadword(word);
      primary = [...all].filter((e) => e.lang !== "ko").sort((a, b) => a.rank - b.rank)[0] ?? all[0] ?? null;
    }
    if (!primary && hw) all = await db.getEntriesByHeadword(hw);
    if (!primary) return { primary: null, entries: all };
    const entries = sameWordRows(all, primary);
    if (!entries.some((e) => e.source === primary!.source && e.id === primary!.id)) entries.push(primary);
    return { primary, entries };
  }, [source, id, word, packsKey(s), stamp]);

  const primary = r.data?.primary;
  useEffect(() => {
    if (primary) recordHistory({ source: primary.source, id: primary.id, headword: primary.headword, hanja: primary.hanja, gloss: primary.gloss });
  }, [primary?.source, primary?.id]);

  if (r.loading && !r.data) return <div class="page"><div class="skeleton" /></div>;
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
      {bySource.map(({ src, list }) => (
        <details key={src} class="dict" open>
          <summary><span>{SOURCE_TITLE[src]}</span><Icon name="down" size={18} /></summary>
          {list.map((e) => <EntryBody key={e.id} e={e} many={list.length > 1} showKo={s.showKoDef || e.lang === 'ko'} />)}
        </details>
      ))}
      {primary.hanja && hanChars(primary.hanja).length > 0 && <HanjaSection primary={primary} />}
      <MoreExamples headword={primary.headword} />
      {entries.some((e) => e.kind === 'grammar') && (
        <a class="btn-link" href={href(`/grammar?q=${encodeURIComponent(primary.headword)}`)}><Icon name="book" size={18} /> Open in grammar reference</a>
      )}
    </article>
  );
}

function Header({ primary, entries }: { primary: Entry; entries: Entry[] }) {
  const bm = useStore(bookmarks);
  const [sheet, setSheet] = useState(false);
  const marked = isBookmarked(bm, primary.source, primary.id);
  const pron = entries.find((e) => e.pron)?.pron;
  const level = entries.map((e) => e.level).find((l) => l != null) ?? null;
  const hom = primary.homonym;
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
  const cur = bm.items.find((b) => b.key === `${entry.source}:${entry.id}`);
  const pick = async (folder: string) => {
    if (cur) await moveBookmark(cur.key, folder);
    else await addBookmark({ source: entry.source, id: entry.id, headword: entry.headword, hanja: entry.hanja, gloss: entry.gloss, folder });
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
      {marked && <button type="button" class="btn danger" onClick={async () => { await removeBookmark(entry.source, entry.id); onClose(); }}>Remove bookmark</button>}
    </Sheet>
  );
}

const RELTYPE: Record<string, string> = { synonym: 'syn', antonym: 'ant', honorific: 'hon', humble: 'hum', 'see also': 'see', reference: 'ref', derived: 'der', variant: 'var', abbreviation: 'abbr' };
function Chips({ rels }: { rels?: { type: string; word: string }[] }) {
  if (!rels?.length) return null;
  return (
    <div class="chips">
      {rels.map((r, i) => (
        <a key={i} class="chip" href={href(searchPath(r.word))}><span class="chip-type">{RELTYPE[r.type] ?? r.type}</span> <span class="hangul" lang="ko">{r.word}</span></a>
      ))}
    </div>
  );
}

function SenseView({ s, n, showKo, isKo }: { s: Sense; n: number; showKo: boolean; isKo: boolean }) {
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
        {s.examples?.map((x, i) => (
          <div key={i} class="ex">
            {x.ko.split('\n').map((l, j) => <div key={j} class="ex-ko hangul" lang="ko">{l}</div>)}
            {x.en && x.en.split('\n').map((l, j) => <div key={j} class="ex-en">{l}</div>)}
          </div>
        ))}
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

function HanjaSection({ primary }: { primary: Entry }) {
  const chars = [...new Set(hanChars(primary.hanja!))];
  const r = useAsync(async () => Promise.all(chars.map(async (ch) => ({
    ch, info: await db.hanjaChar(ch),
    words: (await db.wordsWithHanja(ch, 12, 0)).filter((w) => !(w.source === primary.source && w.id === primary.id)).slice(0, 5),
  }))), [primary.source, primary.id]);
  return (
    <details class="dict" open>
      <summary><span>Hanja</span><Icon name="down" size={18} /></summary>
      {r.data?.map(({ ch, info, words }) => (
        <div key={ch} class="hanja-block">
          <a class="hanja-big sm" lang="zh-Hant" href={href(hanjaPath(ch))}>{ch}</a>
          <div class="hanja-block-body">
            <div><strong class="hangul">{info?.readings ?? '?'}</strong> <span class="muted">{info?.meaning_en}</span></div>
            <div class="chips">
              {words.map((w) => (
                <a key={`${w.source}:${w.id}`} class="chip" href={href(entryPath(w.source, w.id, w.headword))}>
                  <span class="hangul" lang="ko">{w.headword}</span> {w.hanja && <Hanja text={w.hanja} />}
                </a>
              ))}
              {words.length === 0 && <span class="muted small">No other words.</span>}
            </div>
          </div>
        </div>
      ))}
    </details>
  );
}

function MoreExamples({ headword }: { headword: string }) {
  const [open, setOpen] = useState(false);
  const r = useAsync(() => (open ? db.sentences(stemOf(headword), 8) : Promise.resolve(undefined)), [open, headword]);
  return (
    <details class="dict" onToggle={(e) => setOpen((e.currentTarget as HTMLDetailsElement).open)}>
      <summary><span>More examples</span><Icon name="down" size={18} /></summary>
      {open && r.loading && <p class="muted pad">Loading…</p>}
      {r.data?.length === 0 && <p class="muted pad">No example sentences found.</p>}
      {r.data?.map((x, i) => (
        <div key={i} class="ex"><div class="ex-ko hangul" lang="ko">{x.ko}</div>{x.en && <div class="ex-en">{x.en}</div>}</div>
      ))}
    </details>
  );
}
