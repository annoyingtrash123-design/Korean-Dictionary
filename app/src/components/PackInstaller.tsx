import { useState } from 'preact/hooks';
import { db, MANIFEST_URL, progress } from '../db/client';
import { fmtBytes } from '../lib/app-state';
import { useStore } from '../lib/store';
import type { Manifest } from '../lib/types';

import { packLabel } from '../lib/packs';

export function ProgressBar({ value, label }: { value: number; label: string }) {
  return (
    <div class="progress" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(value * 100)} aria-label={label}>
      <div style={{ width: `${Math.max(0, Math.min(1, value)) * 100}%` }} />
    </div>
  );
}

/** Downloads & installs the given packs one after another; shows a byte progress bar per pack. */
export function PackInstaller({ manifest, packIds, label, onDone }: { manifest: Manifest; packIds: string[]; label: string; onDone?: () => void }) {
  const prog = useStore(progress);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string>();
  const [started, setStarted] = useState(false);
  const packs = manifest.packs.filter((p) => packIds.includes(p.id));

  const run = async () => {
    setBusy(true); setErr(undefined); setStarted(true);
    try { await navigator.storage?.persist?.(); } catch { /* optional */ }
    try {
      for (const p of packs) await db.install(p, MANIFEST_URL(), manifest.version);
      await db.packStatus();
      onDone?.();
    } catch (e) { setErr(String((e as Error)?.message ?? e)); }
    setBusy(false);
  };
  return (
    <div class="installer">
      {started && packs.map((p) => {
        const pr = prog[p.id];
        const phase = !pr ? 'Waiting…' : pr.phase === 'download' ? `Downloading ${fmtBytes(pr.done)} / ${fmtBytes(pr.total)}` : pr.phase === 'install' ? `Installing ${fmtBytes(pr.done)} / ${fmtBytes(pr.total)}` : 'Done';
        const v = !pr ? 0 : pr.phase === 'done' ? 1 : pr.total ? pr.done / pr.total : 0;
        return (
          <div key={p.id} class="pack-prog">
            <div class="pack-prog-head"><span>{packLabel(p.id)}</span><span class="muted small">{phase}</span></div>
            <ProgressBar value={v} label={`${packLabel(p.id)} progress`} />
          </div>
        );
      })}
      {err && <p class="error" role="alert">{err} <span class="muted">Progress is kept — you can resume.</span></p>}
      {!busy && <button type="button" class="btn primary" onClick={run}>{err ? 'Resume download' : label}</button>}
    </div>
  );
}
