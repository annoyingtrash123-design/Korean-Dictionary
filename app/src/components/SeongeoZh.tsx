import { db } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { hanjaKey } from '../lib/hanja-marks';
import { Icon } from './Icons';

/** "Chinese & English" on a 성어 page: the Chinese chengyu from CC-CEDICT (exact hanja match, dictionary data) and an
 *  English idiom with the same meaning (AI-written, reviewed, labelled). Nothing when the entry has neither. */
export function SeongeoZhBlock({ hanja }: { hanja?: string | null }) {
  const key = hanjaKey(hanja);
  const r = useAsync(async () => (key.length === 4 ? await db.seongeoZh(key).catch(() => null) : null), [key]);
  const z = r.data;
  if (!z || (!z.zh_trad && !z.en_equiv)) return null;
  const simp = z.zh_simp ?? z.zh_trad ?? '';
  const differs = !!z.zh_trad && simp !== z.zh_trad;
  return (
    <details class="dict zhen" open data-testid="seongeo-zh">
      <summary><span>Chinese &amp; English</span><Icon name="down" size={18} /></summary>
      <div class="body zh-body">
        {z.zh_trad && (
          <div class="zh-row" data-testid="zh-chengyu">
            <div class="zh-lbl">Chinese chengyu</div>
            <div class="zh-head">
              <span class="hanja zh-han" lang="zh-Hans">{simp}</span>
              {differs && <span class="hanja zh-trad" lang="zh-Hant" aria-label="Traditional form">{z.zh_trad}</span>}
              <span class="zh-pin">{z.zh_pinyin}</span>
            </div>
            {z.zh_en.map((d, i) => <div key={i} class="def zh-def" lang="en">{d}</div>)}
            <div class="zh-src muted">Source: CC-CEDICT (CC BY-SA 4.0)</div>
          </div>
        )}
        {z.en_equiv && (
          <div class="zh-row" data-testid="en-equiv">
            <div class="zh-lbl">English equivalent</div>
            <div lang="en">
              <span class="gloss">{z.en_equiv}</span>
              {z.en_ai && <span class="chip-ai ex-ai" lang="en" title="English equivalent written by AI, then reviewed" aria-label="English equivalent written by AI">AI</span>}
            </div>
            {z.en_note && <div class="note zh-note" lang="en">{z.en_note}</div>}
          </div>
        )}
      </div>
    </details>
  );
}
