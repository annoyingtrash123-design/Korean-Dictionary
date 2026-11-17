import { useEffect, useRef, useState } from 'preact/hooks';
import { SearchBar } from './components/SearchBar';
import { TabBar } from './components/TabBar';
import { Home, HomeHeader } from './views/Home';
import { Recent } from './views/Recent';
import { Results } from './views/Results';
import { EntryView } from './views/Entry';
import { HanjaPage } from './views/HanjaPage';
import { ListsView } from './views/Lists';
import { SettingsView } from './views/Settings';
import { GamesView } from './views/Games';
import { FirstRun } from './views/FirstRun';
import { ReaderLibrary, ReaderEmpty } from './views/Reader';
import { TextView } from './views/TextView';
import { TapLookup } from './components/TapLookup';
import { db, packStatus$, packsHint, refreshStatus, restartWorker } from './db/client';
import { useStore } from './lib/store';
import { navigate, useRoute, type Route } from './lib/router';
import { useWide, useScrollMemo } from './lib/layout';
import { listTab, openList, query } from './lib/app-state';
import { settings } from './lib/settings';
import { applyTheme } from './lib/theme';
import { loadLists, upgradeLists } from './lib/lists';
import { loadHistory, upgradeHistory } from './lib/history';
import { UpdateToast } from './components/UpdateToast';
import { Toast } from './components/Toast';
import { ListPicker } from './components/SaveButton';
import { refFromParams } from './lib/entry-key';
import { Empty } from './components/common';
import { LogoMark } from './components/Logo';
import { requestPersist } from './lib/persist';

export function App() {
  const status = useStore(packStatus$);
  const s = useStore(settings);
  const route = useRoute();
  const main = useRef<HTMLElement>(null);

  useEffect(() => { applyTheme(s); }, [s.theme, s.colors, s.fontSize, s.font]);
  useEffect(() => {
    if (s.theme !== 'system' || !matchMedia) return;
    const mq = matchMedia('(prefers-color-scheme: dark)');
    const f = () => applyTheme(settings.get());
    mq.addEventListener('change', f);
    return () => mq.removeEventListener('change', f);
  }, [s.theme]);
  useEffect(() => {
    void loadLists(); loadHistory();
    // ask on every launch: browsers grant persistent storage once the app is installed, not before
    void requestPersist();
    refreshStatus().then((st) => { if (st.packs.core?.installed) { const lookup = (src: string, id: number) => db.getEntry(src, id); void upgradeLists(lookup); void upgradeHistory(lookup); } /* dictionary updates are manual: Settings → Check for update */ }).catch(() => undefined);
  }, []);
  const wide = useWide();
  const kb = useKeyboardOpen();
  const list = useStore(listTab);
  const q = useStore(query);
  const entryRoute = ENTRY_ROUTES.includes(route.name);
  const readerRoute = route.name === 'reader';
  const listsRoute = route.name === 'lists';
  const openL = useStore(openList);
  const homeRoute = route.name === 'home';
  const leftKind: 'home' | 'search' | 'lists' | 'reader' = readerRoute ? 'reader' : listsRoute ? 'lists' : homeRoute ? 'home' : entryRoute ? list : 'search';
  const leftQ = route.name === 'search' ? (route.params.get('q') ?? '') : q;
  const left = useRef<HTMLElement>(null);
  // Remember which list tab the open entry came from, so the left pane keeps showing it.
  useEffect(() => { if (listsRoute) { listTab.set('lists'); openList.set(route.parts[0]); } else if (readerRoute) listTab.set('reader'); else if (!entryRoute && route.name !== 'settings' && route.name !== 'games') listTab.set(homeRoute ? 'home' : 'search'); }, [route.name, route.parts[0]]);
  // The Bookmarks tab became Lists: old links and history entries land there.
  useEffect(() => { if (route.name === 'bookmarks') navigate('/lists', true); }, [route.name]);
  // Remember scroll per route so Back restores the list position (content is cached, so it paints at once).
  useScrollMemo(main, wide ? route.raw + '|w' : route.raw);
  useScrollMemo(left, wide ? `L:${leftKind}:${leftKind === 'search' ? leftQ : leftKind === 'lists' ? openL ?? '' : ''}` : 'unused');
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
    const single = route.name === 'settings' || route.name === 'games';
    return (
      <div class="app wide">
        <TabBar />
        {!single && (
          <section class="pane-left" aria-label={leftKind === 'lists' ? 'Lists' : leftKind === 'reader' ? 'Reader library' : leftKind === 'home' ? 'Home' : 'Search'}>
            {leftKind === 'home' ? <HomeHeader /> : leftKind !== 'reader' && <header class="topbar"><SearchBar /></header>}
            <main ref={left} class="pane-scroll">
              {leftKind === 'reader' ? <ReaderLibrary currentId={readerRoute ? route.parts[0] : undefined} /> : leftKind === 'lists' ? <ListsView listId={listsRoute ? route.parts[0] : openL} /> : leftKind === 'home' ? <Home /> : leftQ.trim() ? <Results q={leftQ} /> : <Recent />}
            </main>
          </section>
        )}
        <main ref={main} id="main" tabIndex={-1} class={single ? `pane-right single${route.name === 'games' ? ' flush' : ''}` : readerRoute && route.parts[0] ? 'pane-right reader-host' : 'pane-right'}>
          {readerRoute ? (route.parts[0] ? <TextView key={route.parts[0]} id={route.parts[0]} /> : <ReaderEmpty />) : entryRoute || single ? view(route) : <PaneEmpty kind={leftKind} />}
        </main>
        <UpdateToast />
        <Toast />
        <ListPicker />
        <TapLookup />
      </div>
    );
  }
  // Phone: the search bar sits at the bottom (one-handed), just above the tab bar; Results and Recent scroll above it.
  const dock = route.name !== 'home' && !readerRoute && route.name !== 'games';
  return (
    <div class={kb ? 'app kb' : 'app'}>
      {route.name === 'home' && <HomeHeader />}
      <main ref={main} id="main" tabIndex={-1} class={readerRoute && route.parts[0] ? 'reader-host' : undefined}>{view(route)}</main>
      {dock && <footer class="topbar dock"><SearchBar /></footer>}
      <TabBar />
      <UpdateToast />
      <Toast />
      <ListPicker />
      <TapLookup />
    </div>
  );
}

/** The on-screen keyboard is up (the viewport got much shorter than it has been): the tab bar then gets out of the way
 *  so the search bar stays right above the keyboard. Also keeps the app above the keyboard when the WebView does not
 *  resize its layout for it (`--kb` = the part of the layout viewport the keyboard covers). */
function useKeyboardOpen(): boolean {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    let base = window.innerHeight, w = window.innerWidth;
    const f = () => {
      if (window.innerWidth !== w) { w = window.innerWidth; base = window.innerHeight; }
      base = Math.max(base, window.innerHeight);
      setOpen(window.innerHeight < base - 150);
      const vv = window.visualViewport;
      document.documentElement.style.setProperty('--kb', `${vv ? Math.max(0, Math.round(window.innerHeight - vv.height - vv.offsetTop)) : 0}px`);
    };
    window.addEventListener('resize', f);
    window.visualViewport?.addEventListener('resize', f);
    return () => { window.removeEventListener('resize', f); window.visualViewport?.removeEventListener('resize', f); };
  }, []);
  return open;
}

const ENTRY_ROUTES = ['entry', 'word', 'hanja'];

function PaneEmpty({ kind }: { kind: string }) {
  const lists = kind === 'lists', home = kind === 'home';
  return (
    <div class="pane-empty">
      <div class="pane-empty-mark" aria-hidden="true"><LogoMark size={64} /></div>
      <div class="empty-title">{lists ? 'Pick a saved word' : home ? 'Pick a card' : 'Search or pick a word'}</div>
      <div class="muted small">{lists ? 'Choose a list on the left, then a word in it, to read it here.' : home ? 'Open a card on the left to read it here.' : 'Type in the search box, or choose a recent word. Press / to jump to search.'}</div>
    </div>
  );
}

// ?tab= changes must not remount the entry (it would reload and flash)
const entryKey = (r: Route) => { const p = new URLSearchParams(r.params); p.delete('tab'); return `${r.name}/${r.parts.join('/')}?${p}`; };

function view(r: Route) {
  switch (r.name) {
    case 'search': { const q = r.params.get('q') ?? ''; return q.trim() ? <Results q={q} /> : <Recent />; }
    case 'entry': return <EntryView key={entryKey(r)} source={r.parts[0]} id={Number(r.parts[1])} hw={r.params.get('hw') ?? undefined} />;
    case 'word': return <EntryView key={entryKey(r)} word={r.parts[0]} pref={r.params.has('s') ? refFromParams(r.params) : undefined} />;
    case 'hanja': return <HanjaPage key={r.parts[0]} ch={r.parts[0]} />;
    case 'lists': case 'bookmarks': return <ListsView listId={r.name === 'lists' ? r.parts[0] : undefined} />;
    case 'settings': return <SettingsView />;
    case 'games': return <GamesView />;
    case 'reader': return r.parts[0] ? <TextView key={r.parts[0]} id={r.parts[0]} /> : <ReaderLibrary />;
    case 'home': default: return <Home />;
  }
}
