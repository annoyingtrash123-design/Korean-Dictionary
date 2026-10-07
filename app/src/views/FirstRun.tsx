import { useEffect, useState } from 'preact/hooks';
import { dismissHint, hintDismissed, isStandalone, needsHomeScreenHint, persisted$, requestPersist } from '../lib/persist';
import { bytesNeeded, fmtMB } from '../lib/storage';
import { cachedManifest, fetchManifest, refreshStatus } from '../db/client';
import { fmtBytes } from '../lib/app-state';
import { PackInstaller } from '../components/PackInstaller';
import { optionalPacks, packDesc, packEnabled, packLabel } from '../lib/packs';
import { settings, updateSettings } from '../lib/settings';
import { useStore } from '../lib/store';
import type { Manifest } from '../lib/types';

export function FirstRun() {
  const s = useStore(settings);
  const [manifest, setManifest] = useState<Manifest | null>(cachedManifest());
  const [err, setErr] = useState<string>();
  const [tick, setTick] = useState(0);
  const persisted = useStore(persisted$);
  const [dismissed, setDismissed] = useState(hintDismissed());
  useEffect(() => { void requestPersist(); }, []);
  useEffect(() => {
    fetchManifest().then((m) => { setManifest(m); setErr(undefined); }, (e) => setErr(String(e?.message ?? e)));
  }, [tick]);

  const core = manifest?.packs.find((p) => p.id === 'core');
  const opt = optionalPacks(manifest);
  const chosen = opt.filter((p) => packEnabled(s, p.id));
  const ids = ['core', ...chosen.map((p) => p.id)];
  const sel = [core, ...chosen];
  const dl = sel.reduce((a, p) => a + (p?.gz_bytes ?? 0), 0);
  const disk = sel.reduce((a, p) => a + (p?.bytes ?? 0), 0);
  const need = bytesNeeded(sel.filter((p): p is NonNullable<typeof p> => !!p));

  return (
    <div class="firstrun">
      <div class="logo" aria-hidden="true">한</div>
      <h1>Korean Dictionary</h1>
      <p class="lead">A fast, private Korean–English dictionary that works offline. It needs a one-time download of the dictionary data.</p>
      {needsHomeScreenHint(persisted, isStandalone(), dismissed) && (
        <div class="notice a2hs" role="note">
          <strong>Tip: Add to Home Screen so iOS keeps your dictionary data</strong>
          <ul class="plain small">
            <li><b>iPhone / iPad:</b> tap Share, then “Add to Home Screen”.</li>
            <li><b>Android:</b> tap ⋮, then “Install app” (or “Add to Home screen”).</li>
          </ul>
          <button type="button" class="link" onClick={() => { dismissHint(); setDismissed(true); }}>Got it</button>
        </div>
      )}
      {err && !manifest && (
        <div class="card"><p class="error" role="alert">{err}</p><p class="muted small">The first download needs an internet connection.</p>
          <button type="button" class="btn" onClick={() => setTick(tick + 1)}>Try again</button></div>
      )}
      {manifest && core && (
        <div class="card">
          <div class="pack-line"><div><strong>{packLabel('core')}</strong><div class="muted small">{packDesc('core')}</div></div><span class="size">{fmtBytes(core.gz_bytes)}</span></div>
          {opt.map((p) => (
            <label key={p.id} class="pack-line toggle">
              <div><strong>{packLabel(p.id)}</strong><div class="muted small">Optional. {packDesc(p.id)}</div></div>
              <span class="size">{fmtBytes(p.gz_bytes)}</span>
              <input type="checkbox" role="switch" checked={packEnabled(s, p.id)} aria-label={`Include ${packLabel(p.id)}`}
                onChange={(e) => updateSettings({ packs: { ...s.packs, [p.id]: (e.currentTarget as HTMLInputElement).checked } })} />
            </label>
          ))}
          <p class="muted small total">Download {fmtBytes(dl)} · uses about {fmtBytes(disk)} on this device (needs about {fmtMB(need)} free during install). Wi-Fi recommended. You can add or remove the optional pack later in Settings.</p>
          <PackInstaller manifest={manifest} packIds={ids} label="Download" onDone={() => { refreshStatus(); }} />
        </div>
      )}
    </div>
  );
}
