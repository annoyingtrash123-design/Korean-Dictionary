import { useState } from 'preact/hooks';
import type { EnKoPhrase, EnKoSense, EnKoWord } from '../lib/types';
import { PHRASES_COLLAPSED, SENSES_COLLAPSED, collapse, groupTranslations, posShort, visiblePhrases } from '../lib/enko';
import { href, searchPath } from '../lib/router';

const SOURCE = 'Wiktionary, CC BY-SA';

function Words({ words }: { words: EnKoWord[] }) {
  return (
    <span class="enko-words">
      {words.map((w, i) => (
        <span key={w.ko}>{i > 0 && ', '}<a class="hangul" lang="ko" href={href(searchPath(w.ko))} title={w.roman ?? undefined}>{w.ko}</a></span>
      ))}
    </span>
  );
}

function More({ open, total, onToggle, label }: { open: boolean; total: number; onToggle: () => void; label: string }) {
  return (
    <button type="button" class="btn enko-more" aria-expanded={open} onClick={onToggle}>
      {open ? 'Show fewer' : `${label} (${total})`}
    </button>
  );
}

/** "Wiktionary translations" block for an English query: per part of speech, each sense
 *  (muted) with its Korean words as links to the Korean search. Long lists collapse. */
export function EnKoTranslations({ senses }: { senses: EnKoSense[] }) {
  const [open, setOpen] = useState(false);
  const rows = groupTranslations(senses).flatMap((g) => g.senses.map((s, i) => ({ s, pos: i === 0 ? g.pos : '' })));
  const { shown, hidden } = collapse(rows, open, SENSES_COLLAPSED);
  return (
    <section class="enko" aria-label="Wiktionary translations">
      <h2 class="enko-h"><span>{senses[0].term}</span><span class="enko-src">{SOURCE}</span></h2>
      <ul class="plain enko-list">
        {shown.map(({ s, pos }) => (
          <li class="enko-row" key={`${s.pos}|${s.sense ?? ''}`}>
            <span class="pos" title={pos || undefined}>{pos ? posShort(pos) : ''}</span>
            <span class="enko-body">
              {s.sense ? <span class="muted enko-sense">{s.sense}</span> : null}
              <Words words={s.words} />
            </span>
          </li>
        ))}
      </ul>
      {(hidden > 0 || (open && rows.length > SENSES_COLLAPSED + 2)) && <More open={open} total={rows.length} label="All senses" onToggle={() => setOpen(!open)} />}
    </section>
  );
}

/** "English phrases" section: English expressions with the query, each with its Korean (collapsible). */
export function EnKoPhrases({ phrases }: { phrases: EnKoPhrase[] }) {
  const [open, setOpen] = useState(false);
  const { shown, hidden } = visiblePhrases(phrases, open);
  return (
    <section class="enko enko-phrases" aria-label="English phrases">
      <h2 class="enko-h"><span>English phrases</span><span class="enko-src">{SOURCE}</span></h2>
      <ul class="plain enko-list">
        {shown.map((p) => (
          <li class="enko-row enko-prow" key={p.term}>
            <span class="enko-body">
              <span class="enko-term">{p.term}</span>
              <Words words={p.words} />
            </span>
          </li>
        ))}
      </ul>
      {(hidden > 0 || (open && phrases.length > PHRASES_COLLAPSED + 2)) && <More open={open} total={phrases.length} label="All phrases" onToggle={() => setOpen(!open)} />}
    </section>
  );
}
