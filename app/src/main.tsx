import { render } from 'preact';
import { registerSW } from 'virtual:pwa-register';
import { App } from './App';
import { setApplyUpdate, swWaiting } from './lib/sw-update';
import './styles.css';

render(<App />, document.getElementById('app')!);
if (import.meta.env.PROD) {
  // registerType 'prompt': a new service worker waits until the user taps "Reload" in the toast (never mid-install).
  const updateSW = registerSW({ immediate: true, onNeedRefresh: () => swWaiting.set(true) });
  setApplyUpdate(() => { void updateSW(true); });
}
