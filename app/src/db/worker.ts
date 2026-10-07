/// <reference lib="webworker" />
import { createEngine } from './engine';
import { forgetPack, getInstalled, installPack } from './download';
import type { Evt, Req, Res } from './rpc';
import type { Engine, ManifestPack, PackStatus } from './types';

const engine: Engine = createEngine();
const post = (m: Res | Evt) => (self as unknown as Worker).postMessage(m);
let initError: string | undefined;
const ready = engine.init().catch((e) => { initError = String(e?.message ?? e); });
let installing = false;

async function packStatus(): Promise<PackStatus> {
  await ready;
  const packs: PackStatus['packs'] = {};
  for (const p of await engine.installedPacks()) {
    const rec = await getInstalled(p.id);
    packs[p.id] = { id: p.id, installed: true, version: rec?.version ?? p.version, bytes: p.bytes, installedAt: rec?.installedAt };
  }
  return { ready: !initError, packs, error: initError };
}

const methods: Record<string, (...a: any[]) => Promise<unknown>> = {
  packStatus,
  async install(pack: ManifestPack, manifestUrl: string, version: string) {
    await ready;
    if (installing) throw new Error('An install is already running');
    installing = true;
    try { await installPack(engine, pack, manifestUrl, version, (p) => post({ event: 'progress', payload: p })); }
    finally { installing = false; }
    return packStatus();
  },
  async removePack(id: string) { await ready; await engine.deletePack(id); await forgetPack(id); return packStatus(); },
  async search(q: string, o: { packs: string[]; limit?: number }) { await ready; return engine.search(q, o); },
  async entriesByHeadword(hw: string, packs: string[]) { await ready; return engine.entriesByHeadword(hw, packs); },
  async entry(s: string, id: number) { await ready; return engine.entry(s, id); },
  async hanjaChar(ch: string) { await ready; return engine.hanjaChar(ch); },
  async wordsWithHanja(ch: string, l: number, o: number) { await ready; return engine.wordsWithHanja(ch, l, o); },
  async sentences(t: string, l: number) { await ready; return engine.sentences(t, l); },
  async grammarList() { await ready; return engine.grammarList(); },
  async wordOfDay(d: string) { await ready; return engine.wordOfDay(d); },
};

self.onmessage = async (ev: MessageEvent<Req>) => {
  const { id, method, args } = ev.data;
  try {
    const fn = methods[method];
    if (!fn) throw new Error('Unknown method ' + method);
    post({ id, result: await fn(...args) });
  } catch (e) {
    post({ id, error: String((e as Error)?.message ?? e) });
  }
};
