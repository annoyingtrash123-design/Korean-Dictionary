import { packsKey } from '../lib/packs';
import { useState } from 'preact/hooks';
import { db } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { Empty, GroupRow } from '../components/common';
import { groupResults } from '../lib/merge';
import { useStore } from '../lib/store';
import { settings } from '../lib/settings';

const PAGE = 30;
const infoCache = new Map<string, Awaited<ReturnType<typeof db.hanjaChar>>>();
export function HanjaPage({ ch }: { ch: string }) {
  const s = useStore(settings);
  const info = useAsync(async () => { const v = await db.hanjaChar(ch); infoCache.set(ch, v); return v; }, [ch], () => infoCache.get(ch));
  const [extra, setExtra] = useState<{ ch: string; rows: Awaited<ReturnType<typeof db.wordsWithHanja>>; more: boolean }>({ ch, rows: [], more: true });
  const first = useAsync(async () => {
    const rows = await db.wordsWithHanja(ch, PAGE, 0);
    setExtra({ ch, rows: [], more: rows.length === PAGE });
    return rows;
  }, [ch, packsKey(s)]);
  const rows = [...(first.data ?? []), ...(extra.ch === ch ? extra.rows : [])];
  const loadMore = async () => {
    const next = await db.wordsWithHanja(ch, PAGE, rows.length);
    setExtra((e) => ({ ch, rows: [...(e.ch === ch ? e.rows : []), ...next], more: next.length === PAGE }));
  };
  const h = info.data;
  if (info.loading && !h) return <div class="page"><div class="skeleton" /></div>;
  if (!h) return <div class="page"><Empty title={`No data for ${ch}`} /></div>;
  return (
    <div class="page">
      <div class="hanja-hero">
        <div class="hanja-giant" lang="zh-Hant">{h.ch}</div>
        <div class="hanja-hero-body">
          <div class="hanja-readings hangul">{h.readings}</div>
          <div class="hanja-mean">{h.meaning_en}</div>
          <dl class="facts">
            {h.strokes != null && <><dt>Strokes</dt><dd>{h.strokes}</dd></>}
            {h.radical && <><dt>Radical</dt><dd lang="zh-Hant">{h.radical}</dd></>}
            {h.word_count != null && <><dt>Words</dt><dd>{h.word_count}</dd></>}
          </dl>
        </div>
      </div>
      <div class="section-head"><h2>Words containing {h.ch}</h2></div>
      <ul class="plain list">
        {groupResults(rows).map((g) => <li key={g.key}><GroupRow g={g} /></li>)}
      </ul>
      {extra.more && rows.length > 0 && <button type="button" class="btn" onClick={loadMore}>Show more</button>}
    </div>
  );
}
