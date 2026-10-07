import { history, clearHistory } from '../lib/history';
import { useStore } from '../lib/store';
import { href } from '../lib/router';
import { savedPath } from '../lib/entry-key';
import { Hanja } from '../components/common';
import { UpdateBanner } from '../components/UpdateBanner';

export function Home() {
  const hist = useStore(history);
  return (
    <div class="page">
      <UpdateBanner />
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
              <a class="row compact" href={href(savedPath(h))}>
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
