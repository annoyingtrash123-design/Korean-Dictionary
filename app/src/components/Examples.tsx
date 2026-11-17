import { Icon } from './Icons';
import { TAP } from './TapLookup';
import { dialogueTurns, speak } from '../lib/examples';

/** Numbered example sentences: tappable Korean (dialogues as speaker turns), English when the
 *  source has it, and a listen button that uses the device's Korean voice. */
export function ExampleList({ items }: { items: { ko: string; en?: string | null; type?: string | null; en_ai?: boolean; ko_ai?: boolean }[] }) {
  return (
    <ol class="exs">
      {items.map((x, i) => {
        const turns = dialogueTurns(x.ko, x.type);
        const en = x.en ? x.en.split('\n') : [];
        return (
          <li key={i} class="ex">
            <span class="ex-n" aria-hidden="true">{i + 1}</span>
            <div class="ex-main">
              <div class="ex-text">
                {turns
                  ? turns.map((t, j) => <div key={j} class="ex-turn"><span class="ex-who" lang="ko">{t.who}</span><span class="ex-ko hangul" lang="ko" {...TAP}>{t.text}</span></div>)
                  : x.ko.split('\n').map((l, j) => <div key={j} class="ex-ko hangul" lang="ko" {...TAP}>{x.ko_ai && j === 0 && <span class="chip-ai ex-ai" lang="en" title="Korean sentence written by AI" aria-label="Korean sentence written by AI">AI</span>}{l}</div>)}
                {en.map((l, j) => <div key={j} class="ex-en">{x.en_ai && j === 0 && <span class="chip-ai ex-ai" title="AI translation, reviewed">AI</span>}{l}</div>)}
              </div>
              <button type="button" class="ex-say" aria-label="Listen" title="Listen" onClick={() => speak(x.ko, x.type)}><Icon name="speaker" size={18} /></button>
            </div>
          </li>
        );
      })}
    </ol>
  );
}
