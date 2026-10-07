import { useEffect, useRef } from 'preact/hooks';
import { SearchBar } from './components/SearchBar';
import { TabBar } from './components/TabBar';
import { Home } from './views/Home';
import { Results } from './views/Results';
import { EntryView } from './views/Entry';
import { HanjaPage } from './views/HanjaPage';
import { BookmarksView } from './views/Bookmarks';
import { SettingsView, checkForUpdate } from './views/Settings';
import { FirstRun } from './views/FirstRun';
import { ReaderLibrary, ReaderEmpty } from './views/Reader';
import { TextView } from './views/TextView';
import { db, packStatus$, packsHint, refreshStatus, restartWorker } from './db/client';
import { useStore } from './lib/store';
import { useRoute, type Route } from './lib/router';
import { useWide, useScrollMemo } from './lib/layout';
import { listTab, query } from './lib/app-state';
import { settings } from './lib/settings';
import { applyTheme } from './lib/theme';
import { loadBookmarks, upgradeBookmarks } from './lib/bookmarks';
import { loadHistory, upgradeHistory } from './lib/history';
import { UpdateToast } from './components/UpdateToast';
import { refFromParams } from './lib/entry-key';
import { Empty } from './components/common';

export function App() {
  const status = useStore(packStatus$);
  const s = useStore(settings);
  const route = useRoute();
  const main = useRef<HTMLElement>(null);

  useEffect(() => { applyTheme(s); }, [s.theme, s.colors, s.fontSize]);
  useEffect(() => {
    if (s.theme !== 'system' || !matchMedia) return;
    const mq = matchMedia('(prefers-color-scheme: dark)');
    const f = () => applyTheme(settings.get());
    mq.addEventListener('change', f);
    return () => mq.removeEventListener('change', f);
  }, [s.theme]);
  useEffect(() => {
    loadBookmarks(); loadHistory();
    refreshStatus().then((st) => { if (st.packs.core?.installed) { const lookup = (src: string, id: number) => db.getEntry(src, id); void upgradeBookmarks(lookup); void upgradeHistory(lookup); } if (navigator.onLine) checkForUpdate(); }).catch(() => undefined);
  }, []);
  const wide = useWide();
  const list = useStore(listTab);
  const q = useStore(query);
  const entryRoute = ENTRY_ROUTES.includes(route.name);
  const readerRoute = route.name === 'reader';
  const leftKind: 'search' | 'bookmarks' | 'reader' = readerRoute ? 'reader' : route.name === 'bookmarks' ? 'bookmarks' : entryRoute ? list : 'search';
  const leftQ = route.name === 'search' ? (route.params.get('q') ?? '') : q;
  const left = useRef<HTMLElement>(null);
  // Remember which list tab the open entry came from, so the left pane keeps showing it.
  useEffect(() => { if (route.name === 'bookmarks') listTab.set('bookmarks'); else if (readerRoute) listTab.set('reader'); else if (!entryRoute && route.name !== 'settings') listTab.set('search'); }, [route.name]);
  // Remember scroll per route so Back restores the list position (content is cached, so it paints at once).
  useScrollMemo(main, wide ? route.raw + '|w' : route.raw);
  useScrollMemo(left, wide ? `L:${leftKind}:${leftKind === 'search' ? leftQ : ''}` : 'unused');
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement | null;
      const typing = !!t && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
      const cmdK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k';
      if (cmdK || (e.key === '/' && !typing && !e.metaKey && !e.ctrlKey && !e.altKey)) {
        const el = document.querySelector<HTMLInputElement>('.searchbar input');
        if (el) { e.preventDefault(); el.focus(); el.select(); }
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const hinted = packsHint().includes('core');
  if (!status && !hinted) return <div class="splash" aria-busy="true"><div class="spinner" /></div>;
  if (status?.error) return (
    <div class="firstrun">
      <Empty title="Could not open the dictionary storage">{status.error} — If the app is open in another tab or window, close it, then retry.</Empty>
      <button type="button" class="btn primary" onClick={() => { restartWorker(); refreshStatus().catch(() => undefined); }}>Retry</button>
    </div>
  );
  if (status && !status.packs.core?.installed) return <FirstRun />;

  if (wide) {
    const single = route.name === 'settings';
    return (
      <div class="app wide">
        <TabBar />
        {!single && (
          <section class="pane-left" aria-label={leftKind === 'bookmarks' ? 'Bookmarks' : leftKind === 'reader' ? 'Reader library' : 'Search'}>
            {leftKind !== 'reader' && <header class="topbar"><SearchBar /></header>}
            <main ref={left} class="pane-scroll">
              {leftKind === 'reader' ? <ReaderLibrary currentId={readerRoute ? route.parts[0] : undefined} /> : leftKind === 'bookmarks' ? <BookmarksView /> : leftQ.trim() ? <Results q={leftQ} /> : <Home />}
            </main>
          </section>
        )}
        <main ref={main} id="main" tabIndex={-1} class={single ? 'pane-right single' : readerRoute && route.parts[0] ? 'pane-right reader-host' : 'pane-right'}>
          {readerRoute ? (route.parts[0] ? <TextView key={route.parts[0]} id={route.parts[0]} /> : <ReaderEmpty />) : entryRoute || single ? view(route) : <PaneEmpty bookmarks={leftKind === 'bookmarks'} />}
        </main>
        <UpdateToast />
      </div>
    );
  }
  return (
    <div class="app">
      {!readerRoute && <header class="topbar"><SearchBar /></header>}
      <main ref={main} id="main" tabIndex={-1} class={readerRoute && route.parts[0] ? 'reader-host' : undefined}>{view(route)}</main>
      <TabBar />
      <UpdateToast />
    </div>
  );
}

const ENTRY_ROUTES = ['entry', 'word', 'hanja'];

function PaneEmpty({ bookmarks }: { bookmarks: boolean }) {
  return (
    <div class="pane-empty">
      <div class="pane-empty-mark" aria-hidden="true">한</div>
      <div class="empty-title">{bookmarks ? 'Pick a saved word' : 'Search or pick a word'}</div>
      <div class="muted small">{bookmarks ? 'Choose a bookmark on the left to read it here.' : 'Type in the search box, or choose a recent word. Press / to jump to search.'}</div>
    </div>
  );
}

// ?tab= changes must not remount the entry (it would reload and flash)
const entryKey = (r: Route) => { const p = new URLSearchParams(r.params); p.delete('tab'); return `${r.name}/${r.parts.join('/')}?${p}`; };

function view(r: Route) {
  switch (r.name) {
    case 'search': { const q = r.params.get('q') ?? ''; return q.trim() ? <Results q={q} /> : <Home />; }
    case 'entry': return <EntryView key={entryKey(r)} source={r.parts[0]} id={Number(r.parts[1])} hw={r.params.get('hw') ?? undefined} />;
    case 'word': return <EntryView key={entryKey(r)} word={r.parts[0]} pref={r.params.has('s') ? refFromParams(r.params) : undefined} />;
    case 'hanja': return <HanjaPage key={r.parts[0]} ch={r.parts[0]} />;
    case 'bookmarks': return <BookmarksView />;
    case 'settings': return <SettingsView />;
    case 'reader': return r.parts[0] ? <TextView key={r.parts[0]} id={r.parts[0]} /> : <ReaderLibrary />;
    default: return <Home />;
  }
}
