import { useAsync } from '../lib/useAsync';
import { db } from '../db/client';
import { history, clearHistory } from '../lib/history';
import { useStore } from '../lib/store';
import { entryPath, href } from '../lib/router';
import { Hanja, LevelBadge, Pos } from '../components/common';
import { UpdateBanner } from '../components/UpdateBanner';
import { todayStr } from '../lib/app-state';

export function Home() {
  const hist = useStore(history);
  const wotd = useAsync(() => db.randomWordOfDay(todayStr()), []);
  const w = wotd.data;
  return (
    <div class="page">
      <UpdateBanner />
      {w && (
        <a class="wotd" href={href(entryPath(w.source, w.id, w.headword))}>
          <div class="eyebrow">Word of the day</div>
          <div class="wotd-head">
            <span class="hangul wotd-hw" lang="ko">{w.headword}</span>
            {w.hanja && <Hanja text={w.hanja} />}
            <LevelBadge level={w.level} />
          </div>
          <div class="wotd-gloss"><Pos pos={w.pos} /> {w.gloss}</div>
        </a>
      )}
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
              <a class="row compact" href={href(entryPath(h.source, h.id, h.headword))}>
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
