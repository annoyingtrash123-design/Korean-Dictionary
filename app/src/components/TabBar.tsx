import { Icon } from './Icons';
import { href, searchPath, useRoute } from '../lib/router';
import { query } from '../lib/app-state';

const TABS = [
  { name: 'search', icon: 'search', label: 'Search', match: ['home', 'search', 'entry', 'word', 'hanja'] },
  { name: 'grammar', icon: 'book', label: 'Grammar', match: ['grammar'] },
  { name: 'bookmarks', icon: 'star', label: 'Bookmarks', match: ['bookmarks'] },
  { name: 'settings', icon: 'gear', label: 'Settings', match: ['settings'] },
];

export function TabBar() {
  const r = useRoute();
  return (
    <nav class="tabbar" aria-label="Main">
      {TABS.map((t) => {
        const active = t.match.includes(r.name);
        const to = t.name === 'search' ? (query.get().trim() ? searchPath(query.get()) : '/') : '/' + t.name;
        return (
          <a key={t.name} href={href(to)} class={active ? 'tab on' : 'tab'} aria-current={active ? 'page' : undefined}>
            <Icon name={t.icon} size={23} fill={active && t.icon === 'star'} />
            <span>{t.label}</span>
          </a>
        );
      })}
    </nav>
  );
}
