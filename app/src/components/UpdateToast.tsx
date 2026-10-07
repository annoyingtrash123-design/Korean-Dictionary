import { useStore } from '../lib/store';
import { applyUpdate, installActive, swWaiting } from '../lib/sw-update';

/** Small, non-blocking notice shown when a new app version has been downloaded. */
export function UpdateToast() {
  const waiting = useStore(swWaiting);
  const busy = useStore(installActive);
  if (!waiting) return null;
  return (
    <div class="toast" role="status">
      <span>{busy ? 'App updated — reload after the download finishes' : 'App updated'}</span>
      <button type="button" class="link" disabled={busy} onClick={() => applyUpdate()}>Reload</button>
      <button type="button" class="link dim" aria-label="Dismiss" onClick={() => swWaiting.set(false)}>×</button>
    </div>
  );
}
