import { update } from '../lib/app-state';
import { useStore } from '../lib/store';
import { href } from '../lib/router';
import { packLabel } from '../lib/packs';

export function UpdateBanner() {
  const u = useStore(update);
  if (!u.available) return null;
  return (
    <a class="banner" href={href('/settings')}>
      <strong>Dictionary update available</strong>
      <span>{u.stale.length ? `${u.stale.map(packLabel).join(', ')} · ` : ''}version {u.manifest?.version} — tap to update in Settings.</span>
    </a>
  );
}
