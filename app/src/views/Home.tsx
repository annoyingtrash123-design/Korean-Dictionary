import { useEffect, useState } from 'preact/hooks';
import { db } from '../db/client';
import { logError } from '../lib/errlog';
import type { HomeData } from '../db/types';
import { dayNumber } from '../lib/home-day';
import { IdiomCard, TodayCarousel } from '../components/TodayCarousel';
import { history, clearHistory } from '../lib/history';
import { useStore } from '../lib/store';
import { href, useRoute, withoutTab } from '../lib/router';
import { savedPath } from '../lib/entry-key';
import { Hanja } from '../components/common';

/** Start screen of the Search tab: today's cards (curriculum words + grammar when the pack has them), the idiom of the day, then recent lookups. */
export function Home() {
  const hist = useStore(history);
  const route = useRoute();
  const [day, setDay] = useState(() => dayNumber());
  // the app can stay open past midnight: pick up the new day when it comes back to the foreground
  useEffect(() => {
    const f = () => setDay(dayNumber());
    document.addEventListener('visibilitychange', f);
    window.addEventListener('focus', f);
    return () => { document.removeEventListener('visibilitychange', f); window.removeEventListener('focus', f); };
  }, []);
  const [h, setH] = useState<HomeData | undefined>();
  useEffect(() => {
    let live = true;
    (async () => {
      const cached = db.cachedIdiomPool();
      const first = await db.home(day, cached ?? []);
      if (!live) return;
      setH(first);
      if (cached) return;
      // the idiom pool is found in the background (once per dictionary version): the cards show first
      const pool = await db.idiomPool();
      if (live) setH(await db.home(day, pool));
    })().catch((e) => logError('home', e));
    return () => { live = false; };
  }, [day]);
  return (
    <div class="page">
      {h && <TodayCarousel words={h.words} grammar={h.grammar} />}
      {h?.idiom && <IdiomCard i={h.idiom} />}
      <div class="section-head">
        <h2>Recent</h2>
        {hist.length > 0 && <button type="button" class="link" onClick={() => clearHistory()}>Clear</button>}
      </div>
      {hist.length === 0 ? (
        <p class="muted pad">Words you look up will appear here.</p>
      ) : (
        <ul class="plain list">
          {hist.slice(0, 30).map((h) => (
            <li key={h.key}>
              <a class="row compact" href={href(savedPath(h))} aria-current={withoutTab(route.raw) === savedPath(h) ? 'true' : undefined}>
                <div class="row-main">
                  <div class="row-head">
                    <span class="hangul hw" lang="ko">{h.headword}</span>{h.hanja && <Hanja text={h.hanja} />}
                  </div>
                  {h.gloss && <div class="row-gloss">{h.gloss}</div>}
                </div>
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
