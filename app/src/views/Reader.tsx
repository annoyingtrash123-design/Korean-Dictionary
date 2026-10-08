import { useEffect, useState } from 'preact/hooks';
import { cachedManifest, db, fetchManifest, packStatus$, packsHint, refreshStatus, resetTexts } from '../db/client';
import { useAsync } from '../lib/useAsync';
import { useStore } from '../lib/store';
import { fmtBytes } from '../lib/app-state';
import { PackInstaller } from '../components/PackInstaller';
import { Empty } from '../components/common';
import { href, readerPath } from '../lib/router';
import { createStore } from '../lib/store';
import { readIds } from '../lib/reader-prefs';
import {
  LENGTHS, LEVELS, NO_FILTERS, PERIODS, SCRIPTS, SHELVES, THEMES, activeFilterCount, byPeriod, byShelf, byline, fmtChars, fmtYear, matches,
  periodId, type Filters,
} from '../lib/reader-library';
import type { Manifest, PendingText, TextSummary } from '../lib/types';

type View = 'shelves' | 'timeline' | 'todo';
const VIEW_KEY = 'kd.reader.view';
const loadView = (): View => { try { const v = localStorage.getItem(VIEW_KEY); return v === 'timeline' || v === 'todo' ? v : 'shelves'; } catch { return 'shelves'; } };
const view$ = createStore<View>(loadView());
view$.subscribe((v) => { try { localStorage.setItem(VIEW_KEY, v); } catch { /* */ } });
const filters$ = createStore<Filters>({ ...NO_FILTERS });
/** Collapsed shelf / period sections (`shelf:<id>`, `period:<id>`), remembered per device. */
const FOLD_KEY = 'kd.reader.collapsed';
const loadFolds = (): string[] => { try { const v = JSON.parse(localStorage.getItem(FOLD_KEY) || '[]'); return Array.isArray(v) ? v.filter((x) => typeof x === 'string') : []; } catch { return []; } };
const folds$ = createStore<string[]>(loadFolds());
folds$.subscribe((v) => { try { localStorage.setItem(FOLD_KEY, JSON.stringify(v)); } catch { /* */ } });
const toggleFold = (k: string) => folds$.set((p) => (p.includes(k) ? p.filter((x) => x !== k) : [...p, k]));

/** Is the `texts` pack usable (installed)? `undefined` while the engine is still starting. */
export function useTextsReady(): boolean | undefined {
  const st = useStore(packStatus$);
  if (!st) return packsHint().includes('texts') ? true : undefined;
  return !!st.packs.texts?.installed;
}

function TextsPrompt() {
  const [manifest, setManifest] = useState<Manifest | null>(cachedManifest());
  const [offline, setOffline] = useState(false);
  useEffect(() => { fetchManifest().then(setManifest, () => setOffline(true)); }, []);
  const pack = manifest?.packs.find((p) => p.id === 'texts');
  return (
    <div class="rd-prompt">
      <div class="rd-prompt-mark" aria-hidden="true">읽</div>
      <h1>Reader library</h1>
      <p class="muted">Classic prose and verse, hanmun, historical documents, modern poetry and fiction, plus graded readers — with tap-to-look-up and parallel English. Read fully offline.</p>
      {pack && manifest ? (
        <>
          <p class="small muted">Download size {fmtBytes(pack.gz_bytes)} · about {fmtBytes(pack.bytes)} on your device.</p>
          <PackInstaller manifest={manifest} packIds={['texts']} label={`Download Reader library (${fmtBytes(pack.gz_bytes)})`} onDone={() => { resetTexts(); void refreshStatus(); }} />
        </>
      ) : offline && !manifest ? <p class="error" role="alert">You need to be online to download the Reader library.</p>
        : manifest ? <p class="muted small">The Reader library is not published with this version of the dictionary yet.</p>
        : <div class="spinner" aria-label="Loading" />}
    </div>
  );
}

function Chips({ t }: { t: TextSummary }) {
  return (
    <div class="rd-chips">
      {t.level && <span class="chip-lvl">{t.level}</span>}
      <span class="rd-len">{fmtChars(t.chars)}</span>
      {t.labels?.translation === 'ai' && <span class="chip-quiet">AI translation</span>}
      {t.excerpt && <span class="chip-quiet">excerpt</span>}
    </div>
  );
}

function Row({ t, current, read, compact }: { t: TextSummary; current: boolean; read: boolean; compact?: boolean }) {
  return (
    <a class={`rd-row${compact ? ' compact' : ''}`} href={href(readerPath(t.id))} aria-current={current ? 'true' : undefined} data-id={t.id}>
      <div class="rd-row-main">
        <div class="rd-row-title"><span lang="ko">{t.title_ko}</span>{read && <span class="rd-read" title="Started" aria-label="Started" />}</div>
        {t.title_en && <div class="rd-row-en">{t.title_en}</div>}
        <div class="rd-row-by">{byline(t)}</div>
        <Chips t={t} />
      </div>
    </a>
  );
}

function Select({ label, value, onChange, options }: { label: string; value: string; onChange: (v: string) => void; options: { id: string; label: string }[] }) {
  return (
    <label class="rd-sel"><span>{label}</span>
      <select value={value} aria-label={label} onChange={(e) => onChange((e.currentTarget as HTMLSelectElement).value)}>
        <option value="">All</option>
        {options.map((o) => <option key={o.id} value={o.id}>{o.label}</option>)}
      </select>
    </label>
  );
}

const PENDING_STATUS: Record<PendingText['status'], { label: string; ko: string }> = {
  preparing: { label: 'Notes and translation in preparation', ko: '해설·번역 준비 중' },
  repair: { label: 'Source text needs repair', ko: '원문 정리 필요' },
  held: { label: 'On hold: source or copyright check', ko: '출처·저작권 확인 중' },
  source: { label: 'Looking for a public-domain source', ko: '원문 찾는 중' },
};

/** The "To add" tab: catalogue texts that are planned but not in the library yet. */
function ToAdd() {
  const list = useAsync(() => db.pendingTexts(), []);
  const all = list.data ?? [];
  if (list.loading) return <div class="skeleton" />;
  if (!all.length) return <Empty title="Nothing waiting">Every planned text is in the library.</Empty>;
  const order: PendingText['status'][] = ['preparing', 'repair', 'held', 'source'];
  return (
    <div class="rd-todo">
      <p class="small muted">Planned texts that are not ready yet. They join the library once their source is checked and their notes are reviewed.</p>
      {order.map((st) => {
        const texts = all.filter((t) => t.status === st).sort((a, b) => a.year - b.year);
        if (!texts.length) return null;
        const s = PENDING_STATUS[st];
        return (
          <section key={st} class="rd-shelf" aria-label={s.label}>
            <h2 class="rd-shelf-h"><span>{s.label}</span><span class="rd-shelf-ko" lang="ko">{s.ko}</span><span class="rd-count">{texts.length}</span></h2>
            <ul class="plain rd-rows">
              {texts.map((t) => (
                <li key={t.id} class="rd-row rd-todo-row" data-id={t.id}>
                  <div class="rd-row-main">
                    <div class="rd-row-title"><span lang="ko">{t.title_ko}</span></div>
                    {t.title_en && <div class="rd-row-en">{t.title_en}</div>}
                    <div class="rd-row-by"><span lang="ko">{t.author_ko}</span>{t.date ? ` · ${t.date}` : ''}</div>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}

export function ReaderLibrary({ currentId }: { currentId?: string }) {
  const ready = useTextsReady();
  const view = useStore(view$);
  const f = useStore(filters$);
  const folds = new Set(useStore(folds$));
  const [open, setOpen] = useState(false);
  const list = useAsync(() => (ready ? db.listTexts() : Promise.resolve([] as TextSummary[])), [ready]);
  const read = new Set(readIds());
  const set = (k: keyof Filters) => (v: string) => filters$.set((p) => ({ ...p, [k]: v }));

  if (ready === undefined) return <div class="rd-lib"><div class="skeleton" /></div>;
  if (!ready) return <div class="rd-lib"><TextsPrompt /></div>;
  const all = list.data ?? [];
  const n = activeFilterCount(f);
  const shown = all.filter((t) => matches(t, f));
  const periodsPresent = PERIODS.filter((p) => all.some((t) => periodId(t) === p.id));
  const shelves = SHELVES.filter((s) => all.some((t) => t.shelf === s.id));
  return (
    <div class="rd-lib">
      <header class="rd-lib-head">
        <div class="rd-lib-title"><h1><span lang="ko">읽기</span> <span class="rd-h-en">Reader</span></h1><span class="muted small" aria-live="polite">{list.loading ? '' : n ? `${shown.length} of ${all.length} texts` : `${all.length} texts`}</span></div>
        <div class="rd-lib-tools">
          <div class="seg inline" role="tablist" aria-label="Library view">
            <button type="button" role="tab" aria-selected={view === 'shelves'} class={view === 'shelves' ? 'on' : ''} onClick={() => view$.set('shelves')}>Shelves</button>
            <button type="button" role="tab" aria-selected={view === 'timeline'} class={view === 'timeline' ? 'on' : ''} onClick={() => view$.set('timeline')}>Timeline</button>
            <button type="button" role="tab" aria-selected={view === 'todo'} class={view === 'todo' ? 'on' : ''} onClick={() => view$.set('todo')}>To add</button>
          </div>
          {view !== 'todo' && <button type="button" class={`pill${open || n ? ' on' : ''}`} aria-expanded={open} onClick={() => setOpen(!open)}>Filters{n ? ` · ${n}` : ''}</button>}
        </div>
        {open && view !== 'todo' && (
          <div class="rd-filters">
            <Select label="Shelf" value={f.shelf} onChange={set('shelf')} options={shelves.map((s) => ({ id: s.id, label: s.label }))} />
            <Select label="Period" value={f.period} onChange={set('period')} options={periodsPresent.map((p) => ({ id: p.id, label: p.label }))} />
            <Select label="Theme" value={f.theme} onChange={set('theme')} options={THEMES} />
            <Select label="Level" value={f.level} onChange={set('level')} options={LEVELS} />
            <Select label="Length" value={f.length} onChange={set('length')} options={LENGTHS} />
            <Select label="Script" value={f.script} onChange={set('script')} options={SCRIPTS} />
            {n > 0 && <button type="button" class="link rd-clear" onClick={() => filters$.set({ ...NO_FILTERS })}>Clear filters</button>}
          </div>
        )}
      </header>
      {list.error && <Empty title="Could not open the library">{list.error}</Empty>}
      {view === 'todo' && <ToAdd />}
      {view !== 'todo' && !list.error && !list.loading && !shown.length && <Empty title="No texts match">{n ? 'Try removing a filter.' : 'The library is empty.'}</Empty>}
      {view === 'shelves' && byShelf(all, f).map(({ shelf, texts, total }) => (
        <section key={shelf.id} class="rd-shelf" aria-label={shelf.label}>
          <h2 class="rd-shelf-h"><button type="button" class="rd-fold" aria-expanded={!folds.has(`shelf:${shelf.id}`)} onClick={() => toggleFold(`shelf:${shelf.id}`)}>
            <span class="rd-chev" aria-hidden="true" /><span>{shelf.label}</span><span class="rd-shelf-ko" lang="ko">{shelf.ko}</span><span class="rd-count">{texts.length === total ? total : `${texts.length}/${total}`}</span></button></h2>
          {!folds.has(`shelf:${shelf.id}`) && <div class="rd-rows">{texts.map((t) => <Row key={t.id} t={t} current={t.id === currentId} read={read.has(t.id)} />)}</div>}
        </section>
      ))}
      {view === 'timeline' && (
        <div class="tl" role="list" aria-label="Timeline, Gojoseon to today">
          {byPeriod(all, f).map(({ period, texts }) => (
            <section key={period.id} class="tl-band" aria-label={period.label}>
              <h2 class="tl-h"><button type="button" class="rd-fold" aria-expanded={!folds.has(`period:${period.id}`)} onClick={() => toggleFold(`period:${period.id}`)}>
                <span class="rd-chev" aria-hidden="true" /><span class="tl-name">{period.label}</span><span class="tl-ko" lang="ko">{period.ko}</span><span class="tl-span">{period.span}</span><span class="rd-count">{texts.length}</span></button></h2>
              {!folds.has(`period:${period.id}`) && texts.map((t) => (
                <div key={t.id} class="tl-item" role="listitem">
                  <div class="tl-year">{t.year != null ? fmtYear(t.year) : ''}</div>
                  <Row t={t} current={t.id === currentId} read={read.has(t.id)} compact />
                </div>
              ))}
            </section>
          ))}
        </div>
      )}
      {!list.loading && all.length > 0 && <p class="small muted rd-foot">Texts are public domain or openly licensed. AI-written notes and translations are labelled.</p>}
    </div>
  );
}

export function ReaderEmpty() {
  return (
    <div class="pane-empty">
      <div class="pane-empty-mark" aria-hidden="true">읽</div>
      <div class="empty-title">Choose a text</div>
      <div class="muted small">Pick something from the library on the left. Tap any word while reading to look it up.</div>
    </div>
  );
}
