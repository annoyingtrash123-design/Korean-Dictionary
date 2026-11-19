import { useStore } from '../lib/store';
import { hideToast, holdToast, releaseToast, toast$ } from '../lib/toast';

/** Short notice with actions ("Saved to Saved words · Change"), announced politely to screen readers. */
export function Toast() {
  const t = useStore(toast$);
  return (
    <div class="toast-host" role="status" aria-live="polite">
      {t && (
        <div key={t.id} class="toast app-toast" onPointerEnter={(e) => { if (e.pointerType === 'mouse') holdToast(); }} onPointerLeave={(e) => { if (e.pointerType === 'mouse') releaseToast(); }} onFocusIn={holdToast} onFocusOut={() => releaseToast()}>
          <span class="toast-text">{t.text}</span>
          {t.actions.map((a) => <button key={a.label} type="button" class="link" onClick={() => { hideToast(); a.run(); }}>{a.label}</button>)}
          <button type="button" class="link dim" aria-label="Dismiss" onClick={hideToast}>×</button>
        </div>
      )}
    </div>
  );
}
