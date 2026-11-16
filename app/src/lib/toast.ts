import { createStore } from './store';

export interface ToastAction { label: string; run: () => void }
export interface Toast { id: number; text: string; actions: ToastAction[] }

/** The one short message at the bottom of the screen ("Saved to … · Change"). A new one replaces the old. */
export const toast$ = createStore<Toast | null>(null);
let seq = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

const hideLater = (id: number, ms: number) => { clearTimeout(timer); timer = setTimeout(() => { if (toast$.get()?.id === id) toast$.set(null); }, ms); };

export function showToast(text: string, actions: ToastAction[] = [], ms = 5000): void {
  const id = ++seq;
  toast$.set({ id, text, actions });
  hideLater(id, ms);
}
export const hideToast = () => { clearTimeout(timer); toast$.set(null); };
/** Keep the toast up while the pointer or focus is on it (someone is about to press Change or Undo). */
export const holdToast = () => clearTimeout(timer);
export const releaseToast = (ms = 2500) => { const t = toast$.get(); if (t) hideLater(t.id, ms); };
