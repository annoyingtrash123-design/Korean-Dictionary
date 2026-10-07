import { createStore } from './store';

/** True once a new app version is installed and waiting; `applyUpdate` activates it and reloads. */
export const swWaiting = createStore(false);
/** True while a dictionary install is running (the app must not reload then). */
export const installActive = createStore(false);
let apply: () => void = () => location.reload();
export const setApplyUpdate = (f: () => void) => { apply = f; };
/** Reload into the new version, unless an install is running. Returns whether it went ahead. */
export function applyUpdate(): boolean {
  if (installActive.get()) return false;
  apply();
  return true;
}
