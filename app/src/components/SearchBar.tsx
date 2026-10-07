import { useEffect, useRef } from 'preact/hooks';
import { Icon } from './Icons';
import { navigate, searchPath, useRoute } from '../lib/router';
import { query } from '../lib/app-state';
import { useStore } from '../lib/store';

export function SearchBar() {
  const q = useStore(query);
  const route = useRoute();
  const input = useRef<HTMLInputElement>(null);

  // Back/forward or links to #/search?q=… update the text.
  useEffect(() => {
    if (route.name === 'home') { if (query.get()) query.set(''); }
    else if (route.name === 'search') { const rq = route.params.get('q') ?? ''; if (rq !== query.get()) query.set(rq); }
  }, [route.raw]);

  // Fire on every keystroke: results components drop stale responses, the client coalesces to the newest query.
  const go = (v: string) => {
    const onSearch = currentName() === 'search';
    if (!v.trim()) navigate('/', onSearch);
    else navigate(searchPath(v), onSearch);
  };
  return (
    <form class="searchbar" role="search" onSubmit={(e) => { e.preventDefault(); go(q); input.current?.blur(); }}>
      <Icon name="search" size={20} />
      <input ref={input} type="search" enterkeyhint="search" inputMode="search" autocomplete="off" autocapitalize="off" autocorrect="off" spellcheck={false}
        placeholder="Search Korean, English or 漢字" aria-label="Search" value={q}
        onInput={(e) => { const v = (e.currentTarget as HTMLInputElement).value; query.set(v); go(v); }} />
      {q && (
        <button type="button" class="clear" aria-label="Clear search" onClick={() => { query.set(''); go(''); input.current?.focus(); }}>
          <Icon name="x" size={16} />
        </button>
      )}
    </form>
  );
}
const currentName = () => (location.hash.replace(/^#\/?/, '').split(/[/?]/)[0] || 'home');
