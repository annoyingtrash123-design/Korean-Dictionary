import { useEffect, useRef } from 'preact/hooks';
import { Icon } from './Icons';
import { navigate, searchPath, useRoute } from '../lib/router';
import { query } from '../lib/app-state';
import { useStore } from '../lib/store';
import { db } from '../db/client';
import { FIRST_PAGE } from '../views/Results';
import { useWide } from '../lib/layout';

export const MAX_QUERY = 100;

export function SearchBar() {
  const q = useStore(query);
  const route = useRoute();
  const input = useRef<HTMLInputElement>(null);
  const wide = useWide();

  // Back/forward or links to #/search?q=… update the text.
  useEffect(() => {
    if (route.name === 'home') { if (query.get()) query.set(''); }
    else if (route.name === 'search') { const rq = route.params.get('q') ?? ''; if (rq !== query.get()) query.set(rq); }
  }, [route.raw]);

  // Fire on every keystroke: results components drop stale responses, the client coalesces to the newest query.
  const go = (v: string) => {
    // Two-pane: typing while an entry is open refreshes the left list and leaves the right pane alone.
    if (wide && ['entry', 'word', 'hanja'].includes(currentName())) return;
    const onSearch = currentName() === 'search';
    if (!v.trim()) navigate('/', onSearch);
    else navigate(searchPath(v), onSearch);
  };
  return (
    <form class="searchbar" role="search" onSubmit={(e) => {
      e.preventDefault();
      const pick = wide ? document.querySelector<HTMLElement>('.pane-left .row.kbd') : null;
      if (pick) { pick.click(); return; }
      go(q.slice(0, MAX_QUERY)); input.current?.blur();
    }}>
      <Icon name="search" size={20} />
      <input ref={input} type="search" enterkeyhint="search" inputMode="search" autocomplete="off" autocapitalize="off" autocorrect="off" spellcheck={false} maxLength={MAX_QUERY}
        placeholder="Search Korean, English or 漢字" aria-label="Search" value={q}
        onKeyDown={(e) => { if (wide && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) moveSelection(e.key === 'ArrowDown' ? 1 : -1, e); }}
        onInput={(e) => {
          clearKbd();
          const el = e.currentTarget as HTMLInputElement;
          const v = el.value.slice(0, MAX_QUERY);   // truncate pasted text (maxlength also guards typing)
          if (v !== el.value) el.value = v;
          if (v.trim()) void db.search(v, { limit: FIRST_PAGE }); // start now; Results joins this request
          query.set(v); go(v);
        }} />
      {q && (
        <button type="button" class="clear" aria-label="Clear search" onClick={() => { query.set(''); go(''); input.current?.focus(); }}>
          <Icon name="x" size={16} />
        </button>
      )}
    </form>
  );
}
const currentName = () => (location.hash.replace(/^#\/?/, '').split(/[/?]/)[0] || 'home');

const clearKbd = () => document.querySelectorAll('.pane-left .row.kbd').forEach((r) => r.classList.remove('kbd'));
/** Arrow keys walk the left list (rows are plain links); Enter then opens the marked row on the right. */
function moveSelection(dir: 1 | -1, e: KeyboardEvent) {
  const rows = Array.from(document.querySelectorAll<HTMLElement>('.pane-left .list .row'));
  if (!rows.length) return;
  e.preventDefault();
  let i = rows.findIndex((r) => r.classList.contains('kbd'));
  if (i < 0) i = rows.findIndex((r) => r.getAttribute('aria-current') === 'true');
  const next = Math.max(0, Math.min(rows.length - 1, i < 0 ? (dir > 0 ? 0 : rows.length - 1) : i + dir));
  clearKbd();
  rows[next].classList.add('kbd');
  rows[next].scrollIntoView({ block: 'nearest' });
}
