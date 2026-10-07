/// <reference lib="webworker" />
import { createEngine } from './engine';
import { forgetPack, getInstalled, installPack } from './download';
import type { Evt, Req, Res } from './rpc';
import type { Engine, ManifestPack, PackStatus } from './types';

const engine: Engine = createEngine();
const post = (m: Res | Evt) => (self as unknown as Worker).postMessage(m);
let initError: string | undefined;
// After a reload the previous worker can still hold the OPFS file handles for a moment, so a
// first init attempt may fail with a lock error: retry for ~10 s before giving up.
async function initWithRetry(): Promise<void> {
  let last: unknown;
  for (let attempt = 0; attempt < 20; attempt++) {
    try { await engine.init(); initError = undefined; return; } catch (e) { last = e; }
    await new Promise((r) => setTimeout(r, Math.min(1000, 150 * (attempt + 1))));
  }
  initError = String((last as Error)?.message ?? last);
}
const ready = initWithRetry();
let installing = false;
let queue: Promise<void> = Promise.resolve();

let lastStatus: PackStatus | null = null;
/** While an install runs the engine's VFS is busy, so answer from the last known status + IndexedDB install records. */
async function statusDuringInstall(): Promise<PackStatus> {
  const packs: PackStatus['packs'] = { ...(lastStatus?.packs ?? {}) };
  for (const id of Object.keys(packs)) {
    const rec = await getInstalled(id);
    if (rec) packs[id] = { ...packs[id], version: rec.version, installedAt: rec.installedAt };
  }
  return { ready: !initError, packs, error: initError };
}
async function packStatus(): Promise<PackStatus> {
  await ready;
  if (installing) return statusDuringInstall();
  const packs: PackStatus['packs'] = {};
  for (const p of await engine.installedPacks()) {
    const rec = await getInstalled(p.id);
    packs[p.id] = { id: p.id, installed: true, version: rec?.version ?? p.version, bytes: p.bytes, installedAt: rec?.installedAt };
  }
  return (lastStatus = { ready: !initError, packs, error: initError });
}

const methods: Record<string, (...a: any[]) => Promise<unknown>> = {
  packStatus,
  async install(pack: ManifestPack, manifestUrl: string, version: string) {
    await ready;
    if (installing) throw new Error('An install is already running');
    if (!lastStatus) await packStatus(); // remember what is installed before the engine gets busy
    installing = true;
    try { await installPack(engine, pack, manifestUrl, version, (p) => post({ event: 'progress', payload: p })); }
    finally { installing = false; }
    return packStatus();
  },
  async removePack(id: string) { await ready; if (installing) throw new Error('An install is running — try again when it finishes'); await engine.deletePack(id); await forgetPack(id); return packStatus(); },
  async search(q: string, o: { packs: string[]; limit?: number }) {
    await ready;
    const t = performance.now();
    const r = await engine.search(q, o);
    return { ...r, engineMs: Math.round((performance.now() - t) * 10) / 10 };
  },
  async entriesByHeadword(hw: string, packs: string[]) { await ready; return engine.entriesByHeadword(hw, packs); },
  async entry(s: string, id: number) { await ready; return engine.entry(s, id); },
  async hanjaChar(ch: string) { await ready; return engine.hanjaChar(ch); },
  async wordsWithHanja(ch: string, l: number, o: number) { await ready; return engine.wordsWithHanja(ch, l, o); },
  async sentences(t: string, l: number) { await ready; return engine.sentences(t, l); },
  async grammarList() { await ready; return engine.grammarList(); },
  async lookupInText(t: string, o: number, opts: { packs: string[]; limit?: number }) { await ready; return engine.lookupInText(t, o, opts); },
  async listTexts() { await ready; return engine.listTexts(); },
  async getText(id: string) { await ready; return engine.getText(id); },
  async diagnostics() { await ready; return engine.diagnostics(); },
  async wordOfDay(d: string) { await ready; return engine.wordOfDay(d); },
};

// Background warm-up: after start (and after installs), read the search indexes into SQLite's
// page cache one small step at a time, only while no request is waiting, so first-time queries
// don't pay for cold storage reads.
let pending = 0;
let warmPacks: string[] = [];
let warmStep = 0;
let warmTimer: ReturnType<typeof setTimeout> | undefined;
function scheduleWarm(delay = 50) {
  clearTimeout(warmTimer);
  warmTimer = setTimeout(async () => {
    if (pending > 0 || installing) return scheduleWarm(200);
    const run = queue.then(() => engine.warm(warmStep, warmPacks));
    queue = run.then(() => undefined, () => undefined);
    const more = await run.catch(() => false);
    warmStep++;
    if (more) scheduleWarm(0);
  }, delay);
}
async function startWarm() {
  await ready;
  if (initError) return;
  const ids = (await engine.installedPacks()).map((p) => p.id).filter((id) => id !== 'opendict');
  if (!ids.length) return;
  warmPacks = ids;
  warmStep = 0;
  scheduleWarm(300);
}
void startWarm();

self.onmessage = async (ev: MessageEvent<Req>) => {
  const { id, method, args } = ev.data;
  try {
    const fn = methods[method];
    if (!fn) throw new Error('Unknown method ' + method);
    // The engine is not re-entrant (overlapping calls give 'disk I/O error'), so queries run one at a time.
    // install() runs outside the queue because it makes many engine calls of its own.
    if (method === 'install') { post({ id, result: await fn(...args) }); void startWarm(); return; }
    pending++;
    const run = queue.then(() => fn(...args));
    queue = run.then(() => undefined, () => undefined);
    try { post({ id, result: await run }); } finally { pending--; }
  } catch (e) {
    post({ id, error: String((e as Error)?.message ?? e) });
  }
};
