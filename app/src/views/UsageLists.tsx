import { useState } from 'preact/hooks';
import type { ComponentChildren } from 'preact';
import { db } from '../db/client';
import type { Confusable, ConfusableWord, WritingExpr } from '../db/types';
import { Icon } from '../components/Icons';
import { ExampleList } from '../components/Examples';
import { TAP } from '../components/TapLookup';
import { speak } from '../lib/examples';
import { useAsync } from '../lib/useAsync';
import { href, useRoute } from '../lib/router';
import type { ListSummary } from '../lib/list-sources';
import {
  CONFUSABLE_KINDS, confusablePath, confusableTitle, filterConfusables, filterWriting, firstSentence, KIND_NAME, parseUsageId, usagePath, writingGroups,
  type ConfusableFilter, type UsageKind,
} from '../lib/study-usage';

// "Writing & usage" study lists: 쓰기 expressions (grouped by function, each row expands to its model sentences) and
// confusable words (each group opens a comparison view). Everything here is AI-written and reviewed: always labelled.

const AI_TITLE = 'Written by AI, then checked by an independent review pass';
function AiChip() {
  return <span class="chip-ai" lang="en" title={AI_TITLE} aria-label="Written by AI">AI</span>;
}
/** The list-level label: what the AI wrote and that it was reviewed. */
function AiNote({ what }: { what: string }) {
  return <p class="uw-ai small" data-testid="usage-ai"><AiChip /><span>AI-written, reviewed. {what}</span></p>;
}

/** Text with “quoted” Korean set in the Korean face (the data quotes every Korean word inside English). */
function Quoted({ text }: { text: string }) {
  return <>{text.split(/(“[^”]*”)/).map((p, i) => (p.startsWith('“') ? <span key={i} class="hangul uw-q" lang="ko">{p}</span> : p))}</>;
}

const OPEN_KEY = 'kd.usage.open';
const loadOpen = (): string[] | null => { try { const v = JSON.parse(localStorage.getItem(OPEN_KEY) ?? 'null'); return Array.isArray(v) ? v.filter((x) => typeof x === 'string') : null; } catch { return null; /* storage unavailable or damaged */ } };
const saveOpen = (v: string[]) => { try { localStorage.setItem(OPEN_KEY, JSON.stringify(v)); } catch { /* storage unavailable: the choice lasts for this visit */ } };

export function UsageDetail({ id, list, path, back }: { id: string; list: ListSummary; path: string; back: ComponentChildren }) {
  const kind = parseUsageId(id) as UsageKind;
  return kind === 'writing' ? <WritingList list={list} back={back} /> : <ConfusableList list={list} path={path} back={back} />;
}

function Head({ list, back, children }: { list: ListSummary; back: ComponentChildren; children?: ComponentChildren }) {
  return (
    <>
      {back}
      <div class="list-title-row">
        <div class="list-titles">
          {list.eyebrow && <div class="eyebrow">{list.eyebrow}</div>}
          <h1 class="list-title" title={list.hint} lang="ko">{list.name}</h1>
        </div>
      </div>
      {list.hint && <p class="list-hint muted small">{list.hint}</p>}
      {children}
    </>
  );
}

function Filter({ q, setQ, label }: { q: string; setQ: (s: string) => void; label: string }) {
  return (
    <div class="uw-filter">
      <Icon name="search" size={18} />
      <input type="search" value={q} aria-label={label} placeholder={label} autocomplete="off" autocapitalize="off" spellcheck={false} onInput={(e) => setQ((e.target as HTMLInputElement).value)} />
    </div>
  );
}

// ---------------------------------------------------------------- 쓰기 expressions

function WritingList({ list, back }: { list: ListSummary; back: ComponentChildren }) {
  const r = useAsync(() => db.writingExpressions(), []);
  const [q, setQ] = useState('');
  const all = r.data ?? [];
  const groups = writingGroups(filterWriting(all, q));
  const searching = q.trim() !== '';
  // groups left open are remembered on this device; until the reader has chosen, the first group is open
  const [open, setOpen] = useState<string[] | null>(loadOpen);
  const isOpen = (name: string, i: number) => searching || (open ? open.includes(name) : i === 0);
  const toggle = (name: string, i: number) => {
    const cur = open ?? (groups[0] ? [groups[0].name] : []);
    const next = isOpen(name, i) ? cur.filter((x) => x !== name) : [...cur, name];
    setOpen(next);
    saveOpen(next);
  };
  return (
    <div class="page list-detail uw" data-testid="writing-list">
      <Head list={list} back={back}>
        <AiNote what="Each note says which spoken forms to avoid in an essay." />
        <Filter q={q} setQ={setQ} label="Filter expressions" />
        <div class="list-sub"><span class="muted small" data-testid="usage-count">{r.loading && !all.length ? 'Loading…' : searching ? `${groups.reduce((n, g) => n + g.rows.length, 0)} of ${all.length} expressions` : `${all.length} expressions · ${groups.length} groups`}</span></div>
      </Head>
      {groups.map((g, i) => {
        const body = `uw-g-${i}`;
        return (
          <section class="uw-group" key={g.name} aria-label={g.name} data-testid="writing-group">
            <h2 class="group-head"><button type="button" class="group-fold" aria-expanded={isOpen(g.name, i)} aria-controls={body} onClick={() => toggle(g.name, i)}>
              <span class="rd-chev" aria-hidden="true" /><span class="group-title">{g.name}</span><span class="group-sum muted small">{g.rows.length}</span>
            </button></h2>
            <div id={body} class="uw-rows" hidden={!isOpen(g.name, i)}>
              {g.rows.map((x) => <WritingItem key={x.id} x={x} />)}
            </div>
          </section>
        );
      })}
      {!r.loading && !groups.length && <div class="empty"><div class="muted">{searching ? 'No expression matches.' : 'Nothing in this list.'}</div></div>}
    </div>
  );
}

function WritingItem({ x }: { x: WritingExpr }) {
  return (
    <details class="uw-item" data-testid="writing-item">
      <summary>
        <span class="uw-main">
          <span class="hangul uw-expr" lang="ko">{x.expr}</span>
          <span class="uw-mean" lang="en">{x.meaning}</span>
        </span>
        <Icon name="down" size={18} />
      </summary>
      <div class="uw-body">
        <p class="uw-register" lang="en"><span class="uw-lbl">Register</span> <Quoted text={x.register} /></p>
        <div class="uw-lbl">Model sentences</div>
        <ExampleList items={x.examples} />
        <div class="uw-foot">
          {x.entry && <a class="uw-open" href={href(usagePath(x.entry, x.expr))} data-testid="uw-entry">Dictionary entry <Icon name="chev" size={16} /></a>}
          {x.ai && <span class="uw-ai-inline muted small"><AiChip /> AI-written, reviewed</span>}
        </div>
      </div>
    </details>
  );
}

// ---------------------------------------------------------------- confusable words

function ConfusableList({ list, path, back }: { list: ListSummary; path: string; back: ComponentChildren }) {
  const r = useAsync(() => db.confusables(), []);
  const route = useRoute();
  const [kind, setKind] = useState<ConfusableFilter>('all');
  const [q, setQ] = useState('');
  const all = r.data ?? [];
  const gid = route.params.get('g');
  const idx = gid ? all.findIndex((c) => c.id === gid) : -1;
  if (idx >= 0) return <Compare c={all[idx]} prev={all[idx - 1]} next={all[idx + 1]} path={path} list={list} />;
  const rows = filterConfusables(all, kind, q);
  const countOf = (k: ConfusableFilter) => filterConfusables(all, k, q).length;
  return (
    <div class="page list-detail uw" data-testid="confusable-list">
      <Head list={list} back={back}>
        <AiNote what="Each group explains the difference and gives one contrasting sentence per word." />
        <Filter q={q} setQ={setQ} label="Filter groups" />
        <div class="seg uw-seg" role="radiogroup" aria-label="Kind">
          {CONFUSABLE_KINDS.map((k) => (
            <button key={k.id} type="button" role="radio" aria-checked={kind === k.id} class={kind === k.id ? 'on' : ''} onClick={() => setKind(k.id)}>{k.label}<span class="count">{countOf(k.id)}</span></button>
          ))}
        </div>
        <div class="list-sub"><span class="muted small" data-testid="usage-count">{r.loading && !all.length ? 'Loading…' : `${rows.length} ${rows.length === 1 ? 'group' : 'groups'}`}</span></div>
      </Head>
      <ul class="plain list uw-groups">
        {rows.map((c) => (
          <li key={c.id}>
            <a class="row compact" href={href(confusablePath(path, c.id))} data-testid="confusable-row">
              <div class="row-main">
                <div class="row-head"><span class="hangul hw" lang="ko">{confusableTitle(c)}</span><span class="kind-tag" lang="en">{KIND_NAME[c.kind]}</span></div>
                <div class="row-gloss uw-gloss" lang="en"><Quoted text={firstSentence(c.explanation)} /></div>
              </div>
              <Icon name="chev" size={18} />
            </a>
          </li>
        ))}
      </ul>
      {!r.loading && !rows.length && <div class="empty"><div class="muted">No group matches.</div></div>}
    </div>
  );
}

function Compare({ c, prev, next, path, list }: { c: Confusable; prev?: Confusable; next?: Confusable; path: string; list: ListSummary }) {
  const real = c.words.filter((w) => !w.wrong).length;
  return (
    <div class="page list-detail uw uw-compare" data-testid="confusable-detail">
      <a class="back-link" href={href(path)}><Icon name="back" size={18} /> <span>{list.name}</span></a>
      <div class="list-title-row">
        <div class="list-titles">
          <div class="eyebrow">{KIND_NAME[c.kind]} · {c.id}</div>
          <h1 class="list-title hangul" lang="ko">{confusableTitle(c)}</h1>
        </div>
        <AiChip />
      </div>
      <p class="uw-expl" lang="en"><Quoted text={c.explanation} /></p>
      <div class="uw-cmp" data-count={real}>
        {c.words.map((w) => <WordCard key={w.word} w={w} />)}
      </div>
      <div class="uw-mistake" data-testid="confusable-mistake">
        <span class="uw-lbl">Common mistake</span>
        <p lang="en"><Quoted text={c.mistake} /></p>
      </div>
      <p class="uw-ai-inline muted small"><AiChip /> AI-written, reviewed</p>
      <nav class="uw-pager" aria-label="Other groups">
        {prev ? <a class="btn" href={href(confusablePath(path, prev.id))} rel="prev"><Icon name="back" size={16} /> <span class="hangul" lang="ko">{confusableTitle(prev)}</span></a> : <span />}
        {next ? <a class="btn" href={href(confusablePath(path, next.id))} rel="next"><span class="hangul" lang="ko">{confusableTitle(next)}</span> <Icon name="chev" size={16} /></a> : <span />}
      </nav>
    </div>
  );
}

function WordCard({ w }: { w: ConfusableWord }) {
  if (w.wrong) {
    return (
      <div class="uw-card wrong" data-testid="word-card-wrong">
        <div class="uw-word"><s class="hangul" lang="ko">{w.word}</s><span class="uw-x" lang="en"><span aria-hidden="true">✗</span> not a valid spelling</span></div>
      </div>
    );
  }
  return (
    <div class="uw-card" data-testid="word-card">
      <div class="uw-word">
        <a class="hangul uw-w" lang="ko" href={href(usagePath(w.entry, w.word))}>{w.word}</a>
        <a class="uw-open" href={href(usagePath(w.entry, w.word))} aria-label={`${w.entry ? 'Dictionary entry' : 'Search'}: ${w.word}`}>{w.entry ? 'Entry' : 'Search'} <Icon name="chev" size={16} /></a>
      </div>
      <div class="uw-ex">
        <div class="uw-ex-main">
          <div class="ex-ko hangul" lang="ko" {...TAP}>{w.ko}</div>
          <div class="ex-en" lang="en">{w.en}</div>
        </div>
        <button type="button" class="ex-say" aria-label="Listen" title="Listen" onClick={() => speak(w.ko)}><Icon name="speaker" size={18} /></button>
      </div>
    </div>
  );
}
