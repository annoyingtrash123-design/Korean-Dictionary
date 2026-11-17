import { db } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { CTX_LABEL, lessonForm, parseLessons, type Lesson, type LessonCompare } from '../lib/lesson';
import { entryPath, href } from '../lib/router';
import { Icon } from './Icons';
import { ExampleList } from './Examples';
import { TAP } from './TapLookup';
import { speak } from '../lib/examples';

/** "Lesson" on a grammar entry's page: the reviewed AI-written mini-lessons of the curriculum rows that
 *  link to this entry (usually one; `-거들랑1` and `-거들랑2` give two). Nothing when there is none. */
export function GrammarLessons({ source, id }: { source: string; id: number }) {
  const r = useAsync(async () => parseLessons(await db.grammarLessons(source, id).catch(() => [])), [source, id]);
  const lessons = r.data ?? [];
  if (!lessons.length) return null;
  return <div class="lessons">{lessons.map((l) => <LessonCard key={`${l.level}:${l.form}`} l={l} many={lessons.length > 1} />)}</div>;
}

function AiChip() {
  return <span class="chip-ai" lang="en" title="Written by AI, then checked by a second review pass" aria-label="Written by AI">AI</span>;
}

function LessonCard({ l, many }: { l: Lesson; many: boolean }) {
  return (
    <details class="lesson" open data-testid="lesson" aria-label={`Lesson: ${lessonForm(l.form)}`}>
      <summary>
        <span class="lesson-title">
          <span class="lesson-h">Lesson</span>
          {many && <span class="lesson-form hangul" lang="ko">{lessonForm(l.form)}</span>}
          <span class="lesson-label">{l.label}</span>
          <AiChip />
        </span>
        <span class="lesson-ai muted">AI-written lesson, reviewed</span>
        <Icon name="down" size={18} />
      </summary>
      <div class="lesson-body">
        <p class="lesson-meaning">{l.meaning_en}</p>

        <h3 class="lesson-sh">How it attaches</h3>
        <p class="lesson-usage">{l.usage}</p>

        <h3 class="lesson-sh">Examples</h3>
        <ExampleList items={l.examples} />
        {l.examples.some((e) => e.ctx) && (
          <ul class="lesson-ctx plain" aria-label="Where each example is used">
            {l.examples.map((e, i) => e.ctx ? <li key={i}><span class="ex-n-sm">{i + 1}</span> {CTX_LABEL[e.ctx] ?? e.ctx}</li> : null)}
          </ul>
        )}

        {l.compare.length > 0 && (
          <>
            <h3 class="lesson-sh">Compare with</h3>
            <div class="lesson-compare">{l.compare.map((c, i) => <CompareCard key={i} c={c} mine={lessonForm(l.form)} />)}</div>
          </>
        )}

        {l.pitfalls.length > 0 && (
          <>
            <h3 class="lesson-sh">Common mistakes</h3>
            <ul class="lesson-pitfalls plain">
              {l.pitfalls.map((p, i) => (
                <li key={i} class="pitfall">
                  <p class="pitfall-note">{p.note}</p>
                  <div class="pitfall-wrong"><span class="mark" aria-label="Wrong">✗</span><span class="hangul" lang="ko" {...TAP}>{p.wrong}</span></div>
                  <div class="pitfall-right">
                    <span class="mark" aria-label="Right">✓</span>
                    <span class="pitfall-right-text"><span class="hangul" lang="ko" {...TAP}>{p.right}</span><span class="ex-en">{p.en}</span></span>
                    <button type="button" class="ex-say" aria-label="Listen" title="Listen" onClick={() => speak(p.right)}><Icon name="speaker" size={18} /></button>
                  </div>
                </li>
              ))}
            </ul>
          </>
        )}

        {l.topik_tip && <p class="lesson-tip"><span class="lesson-tip-h">TOPIK tip</span> {l.topik_tip}</p>}
      </div>
    </details>
  );
}

function CompareCard({ c, mine }: { c: LessonCompare; mine: string }) {
  const to = c.entry ? href(entryPath(c.entry.source, c.entry.id, c.pattern)) : undefined;
  const head = (
    <span class="cmp-head">
      <span class="hangul cmp-pattern" lang="ko">{c.pattern}</span>
      {to && <Icon name="chev" size={16} />}
    </span>
  );
  // tapping the pattern opens its own entry when the dictionary has one
  return (
    <div class="cmp" data-linked={to ? '' : undefined}>
      {to ? <a class="cmp-link" href={to} aria-label={`Open ${c.pattern}`}>{head}</a> : head}
      <p class="cmp-diff">{c.difference}</p>
      <div class="cmp-pair">
        <PairLine s={c.pair[0]} tag={mine} />
        <PairLine s={c.pair[1]} tag={c.pattern} />
      </div>
    </div>
  );
}

function PairLine({ s, tag }: { s: { ko: string; en: string }; tag: string }) {
  return (
    <div class="cmp-line">
      <span class="cmp-tag hangul" lang="ko">{tag}</span>
      <div class="cmp-text">
        <div class="ex-ko hangul" lang="ko" {...TAP}>{s.ko}</div>
        <div class="ex-en">{s.en}</div>
      </div>
      <button type="button" class="ex-say" aria-label="Listen" title="Listen" onClick={() => speak(s.ko)}><Icon name="speaker" size={18} /></button>
    </div>
  );
}
