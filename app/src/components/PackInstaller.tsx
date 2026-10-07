import { useEffect, useRef, useState } from 'preact/hooks';
import { db, MANIFEST_URL, progress } from '../db/client';
import { fmtBytes } from '../lib/app-state';
import { useStore } from '../lib/store';
import type { Manifest } from '../lib/types';

import { packLabel } from '../lib/packs';
import { requestPersist } from '../lib/persist';
import { bytesNeeded, freeSpaceProblem, isFatalInstallMessage, isQuotaError, noSpaceMessage } from '../lib/storage';

type WakeLockSentinelLike = { release(): Promise<void> };

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

  const lock = useRef<WakeLockSentinelLike | null>(null);
  const running = useRef(false);
  const hiddenDuringRun = useRef(false);
  const autoResumed = useRef(false);

  const takeLock = async () => {
    try { lock.current = (await (navigator as unknown as { wakeLock?: { request(t: 'screen'): Promise<WakeLockSentinelLike> } }).wakeLock?.request('screen')) ?? null; } catch { /* optional */ }
  };
  const dropLock = () => { void lock.current?.release().catch(() => undefined); lock.current = null; };

  const run = async () => {
    if (running.current) return;
    running.current = true; hiddenDuringRun.current = false;
    setBusy(true); setErr(undefined); setStarted(true);
    try {
      // Refuse to start (rather than fail half-way) when the device clearly lacks space.
      const est = await navigator.storage?.estimate?.().catch(() => undefined);
      const problem = freeSpaceProblem(est, bytesNeeded(packs));
      if (problem) throw new Error(problem);
      await requestPersist();
      await takeLock();
      for (const p of packs) await db.install(p, MANIFEST_URL(), manifest.version);
      await db.packStatus();
      dropLock();
      running.current = false;
      onDone?.();
    } catch (e) {
      dropLock();
      running.current = false;
      const m = String((e as Error)?.message ?? e);
      setErr(isQuotaError(e) && !isFatalInstallMessage(m) ? noSpaceMessage(bytesNeeded(packs)) : m);
    }
    setBusy(false);
  };
  // Mobile browsers suspend a backgrounded download: remember that, and resume once when the app is visible again.
  const runRef = useRef(run); runRef.current = run;
  const errRef = useRef<string>();
  errRef.current = err;
  useEffect(() => {
    const on = () => {
      if (document.visibilityState === 'hidden') { if (running.current) hiddenDuringRun.current = true; return; }
      if (running.current) void takeLock(); // wake locks are released when the page is hidden
      else if (errRef.current && !isFatalInstallMessage(errRef.current) && hiddenDuringRun.current && !autoResumed.current) { autoResumed.current = true; void runRef.current(); }
    };
    document.addEventListener('visibilitychange', on);
    return () => { document.removeEventListener('visibilitychange', on); dropLock(); };
  }, []);
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
      {err && <p class="error" role="alert">{err}{!isFatalInstallMessage(err) && <> <span class="muted">Progress is kept — you can resume.</span></>}</p>}
      {!busy && <button type="button" class="btn primary" onClick={run}>{err ? (isFatalInstallMessage(err) ? 'Try again' : 'Resume download') : label}</button>}
    </div>
  );
}
