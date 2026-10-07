import { useEffect, useState } from 'preact/hooks';
import { cachedManifest, fetchManifest, refreshStatus } from '../db/client';
import { fmtBytes } from '../lib/app-state';
import { PackInstaller, PACK_LABEL } from '../components/PackInstaller';
import { settings, updateSettings } from '../lib/settings';
import { useStore } from '../lib/store';
import type { Manifest } from '../lib/types';

export function FirstRun() {
  const s = useStore(settings);
  const [manifest, setManifest] = useState<Manifest | null>(cachedManifest());
  const [err, setErr] = useState<string>();
  const [tick, setTick] = useState(0);
  useEffect(() => {
    fetchManifest().then((m) => { setManifest(m); setErr(undefined); }, (e) => setErr(String(e?.message ?? e)));
  }, [tick]);

  const core = manifest?.packs.find((p) => p.id === 'core');
  const std = manifest?.packs.find((p) => p.id === 'stdict');
  const ids = ['core', ...(std && s.stdict ? ['stdict'] : [])];
  const dl = [core, std && s.stdict ? std : undefined].reduce((a, p) => a + (p?.gz_bytes ?? 0), 0);
  const disk = [core, std && s.stdict ? std : undefined].reduce((a, p) => a + (p?.bytes ?? 0), 0);

  return (
    <div class="firstrun">
      <div class="logo" aria-hidden="true">한</div>
      <h1>Korean Dictionary</h1>
      <p class="lead">A fast, private Korean–English dictionary that works offline. It needs a one-time download of the dictionary data.</p>
      {err && !manifest && (
        <div class="card"><p class="error" role="alert">{err}</p><p class="muted small">The first download needs an internet connection.</p>
          <button type="button" class="btn" onClick={() => setTick(tick + 1)}>Try again</button></div>
      )}
      {manifest && core && (
        <div class="card">
          <div class="pack-line"><div><strong>{PACK_LABEL.core}</strong><div class="muted small">English definitions, hanja, examples, grammar</div></div><span class="size">{fmtBytes(core.gz_bytes)}</span></div>
          {std && (
            <label class="pack-line toggle">
              <div><strong>{PACK_LABEL.stdict}</strong><div class="muted small">Optional. Korean definitions for 400k+ words</div></div>
              <span class="size">{fmtBytes(std.gz_bytes)}</span>
              <input type="checkbox" role="switch" checked={s.stdict} onChange={(e) => updateSettings({ stdict: (e.currentTarget as HTMLInputElement).checked })} aria-label="Include the standard Korean dictionary" />
            </label>
          )}
          <p class="muted small total">Download {fmtBytes(dl)} · uses about {fmtBytes(disk)} on this device. Wi-Fi recommended. You can add or remove the optional pack later in Settings.</p>
          <PackInstaller manifest={manifest} packIds={ids} label="Download" onDone={() => { refreshStatus(); }} />
        </div>
      )}
    </div>
  );
}
