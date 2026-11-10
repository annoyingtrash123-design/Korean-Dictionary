import { Icon } from './Icons';
import { TAP } from './TapLookup';
import { canSpeak, dialogueTurns, speak } from '../lib/examples';

/** Numbered example sentences: tappable Korean (dialogues as speaker turns), English when the
 *  source has it, and a listen button that uses the device's Korean voice. */
export function ExampleList({ items }: { items: { ko: string; en?: string | null }[] }) {
  const voice = canSpeak();
  return (
    <ol class="exs">
      {items.map((x, i) => {
        const turns = dialogueTurns(x.ko);
        const en = x.en ? x.en.split('\n') : [];
        return (
          <li key={i} class="ex">
            <div class="ex-main">
              <div class="ex-text">
                {turns
                  ? turns.map((t, j) => <div key={j} class="ex-turn"><span class="ex-who" lang="ko">{t.who}</span><span class="ex-ko hangul" lang="ko" {...TAP}>{t.text}</span></div>)
                  : x.ko.split('\n').map((l, j) => <div key={j} class="ex-ko hangul" lang="ko" {...TAP}>{l}</div>)}
                {en.map((l, j) => <div key={j} class="ex-en">{l}</div>)}
              </div>
              {voice && <button type="button" class="ex-say" aria-label="Listen" title="Listen" onClick={() => speak(x.ko)}><Icon name="speaker" size={18} /></button>}
            </div>
          </li>
        );
      })}
    </ol>
  );
}
