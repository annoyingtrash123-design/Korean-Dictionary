import { createStore } from './store';

const KEY = 'kd.hint.a2hs';
/** Result of the last navigator.storage.persist() call (undefined = not asked yet). */
export const persisted$ = createStore<boolean | undefined>(undefined);

export const isStandalone = (): boolean =>
  (typeof matchMedia === 'function' && matchMedia('(display-mode: standalone)').matches) ||
  (navigator as unknown as { standalone?: boolean }).standalone === true;

/** Ask the browser to keep our storage; false when denied or unsupported. */
export async function requestPersist(): Promise<boolean> {
  let ok = false;
  try { ok = (await navigator.storage?.persisted?.()) || (await navigator.storage?.persist?.()) || false; } catch { /* unsupported */ }
  persisted$.set(ok);
  return ok;
}
/** Show the "Add to Home Screen" tip when storage is not persistent and the app runs in a plain browser tab. */
export const needsHomeScreenHint = (persisted: boolean | undefined, standalone: boolean, dismissed: boolean) =>
  persisted === false && !standalone && !dismissed;

export const hintDismissed = (): boolean => { try { return localStorage.getItem(KEY) === '1'; } catch { return false; } };
export const dismissHint = () => { try { localStorage.setItem(KEY, '1'); } catch { /* */ } };
