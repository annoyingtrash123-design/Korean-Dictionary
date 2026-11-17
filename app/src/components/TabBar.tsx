import { Icon } from './Icons';
import { href, searchPath, useRoute } from '../lib/router';
import { listTab, query } from '../lib/app-state';
import { useStore } from '../lib/store';
import { useWide } from '../lib/layout';

const TABS = [
  { name: 'home', icon: 'home', label: 'Home', match: ['home'] },
  { name: 'search', icon: 'search', label: 'Search', match: ['search', 'entry', 'word', 'hanja'] },
  { name: 'reader', icon: 'book', label: 'Reader', match: ['reader'] },
  { name: 'games', icon: 'dice', label: 'Games', match: ['games'] },
  { name: 'lists', icon: 'lists', label: 'Lists', match: ['lists', 'bookmarks'] },
  { name: 'settings', icon: 'gear', label: 'Settings', match: ['settings'] },
];

export function TabBar() {
  const r = useRoute();
  const wide = useWide();
  const list = useStore(listTab);
  return (
    <nav class="tabbar" aria-label="Main">
      {TABS.map((t) => {
        const entryish = ['entry', 'word', 'hanja'].includes(r.name);
        // an open entry belongs to the tab it was opened from (phone: only Home is told apart, as before)
        const active = entryish && (wide || list === 'home') ? t.name === list : t.match.includes(r.name);
        const to = t.name === 'search' ? (query.get().trim() ? searchPath(query.get()) : '/search') : t.name === 'home' ? '/' : '/' + t.name;
        return (
          <a key={t.name} href={href(to)} class={active ? 'tab on' : 'tab'} aria-current={active ? 'page' : undefined}>
            <Icon name={t.icon} size={23} />
            <span>{t.label}</span>
          </a>
        );
      })}
    </nav>
  );
}
