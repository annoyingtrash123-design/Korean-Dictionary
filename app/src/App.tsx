import { useEffect, useRef } from 'preact/hooks';
import { SearchBar } from './components/SearchBar';
import { TabBar } from './components/TabBar';
import { Home } from './views/Home';
import { Results } from './views/Results';
import { EntryView } from './views/Entry';
import { HanjaPage } from './views/HanjaPage';
import { GrammarView } from './views/Grammar';
import { BookmarksView } from './views/Bookmarks';
import { SettingsView, checkForUpdate } from './views/Settings';
import { FirstRun } from './views/FirstRun';
import { packStatus$, packsHint, refreshStatus } from './db/client';
import { useStore } from './lib/store';
import { useRoute, type Route } from './lib/router';
import { settings } from './lib/settings';
import { applyTheme } from './lib/theme';
import { loadBookmarks } from './lib/bookmarks';
import { loadHistory } from './lib/history';
import { Empty } from './components/common';

const scrollMemo = new Map<string, number>();

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
    refreshStatus().then(() => { if (navigator.onLine) checkForUpdate(); }).catch(() => undefined);
  }, []);
  // Remember scroll per route so Back restores the list position (content is cached, so it paints at once).
  useEffect(() => {
    const el = main.current; if (!el) return;
    const key = route.raw;
    const saved = scrollMemo.get(key) ?? 0;
    el.scrollTop = saved;
    if (saved) requestAnimationFrame(() => { el.scrollTop = saved; });
    const on = () => scrollMemo.set(key, el.scrollTop);
    el.addEventListener('scroll', on, { passive: true });
    return () => el.removeEventListener('scroll', on);
  }, [route.raw]);

  const hinted = packsHint().includes('core');
  if (!status && !hinted) return <div class="splash" aria-busy="true"><div class="spinner" /></div>;
  if (status?.error) return <div class="firstrun"><Empty title="Could not open the dictionary storage">{status.error} — If the app is open in another tab, close it and reload.</Empty></div>;
  if (status && !status.packs.core?.installed) return <FirstRun />;

  return (
    <div class="app">
      <header class="topbar"><SearchBar /></header>
      <main ref={main} id="main" tabIndex={-1}>{view(route)}</main>
      <TabBar />
    </div>
  );
}

function view(r: Route) {
  switch (r.name) {
    case 'search': { const q = r.params.get('q') ?? ''; return q.trim() ? <Results q={q} /> : <Home />; }
    case 'entry': return <EntryView key={r.raw} source={r.parts[0]} id={Number(r.parts[1])} hw={r.params.get('hw') ?? undefined} />;
    case 'word': return <EntryView key={r.raw} word={r.parts[0]} />;
    case 'hanja': return <HanjaPage key={r.parts[0]} ch={r.parts[0]} />;
    case 'grammar': return <GrammarView key={r.params.get('q') ?? ''} initialQ={r.params.get('q') ?? ''} />;
    case 'bookmarks': return <BookmarksView />;
    case 'settings': return <SettingsView />;
    default: return <Home />;
  }
}
