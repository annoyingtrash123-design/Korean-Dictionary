import { render } from 'preact';
import { registerSW } from 'virtual:pwa-register';
import { App } from './App';
import { setApplyUpdate, swWaiting } from './lib/sw-update';
import { installErrorCapture } from './lib/errlog';
import './styles.css';

installErrorCapture();
render(<App />, document.getElementById('app')!);
import { IS_NATIVE, installBackButton } from './lib/native';
void installBackButton();
if (import.meta.env.PROD && !IS_NATIVE) {
  // registerType 'prompt': a new service worker waits until the user taps "Reload" in the toast (never mid-install).
  const updateSW = registerSW({ immediate: true, onNeedRefresh: () => swWaiting.set(true) });
  setApplyUpdate(() => { void updateSW(true); });
}
