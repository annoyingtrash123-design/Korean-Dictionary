// Pack installs always run against the live manifest. Chunk files have fixed names, so a deploy
// that lands mid-download swaps them under a manifest the client already holds: the download then
// fails its checksum/size check (or 404s). A retry with the same stale manifest would fail forever.
import type { Manifest } from './types';

type Pack = Manifest['packs'][number];

/** Install `ids` from the live manifest; if an install fails and the live version has changed
 *  meanwhile, start once more with the new manifest. Returns the manifest that was installed. */
export async function installLive(
  start: Manifest, ids: string[], fetchLive: () => Promise<Manifest>, install: (p: Pack, version: string) => Promise<unknown>,
): Promise<Manifest> {
  const live = async (fallback: Manifest) => { try { return await fetchLive(); } catch { return fallback; } };
  let m = await live(start);
  for (let attempt = 0; ; attempt++) {
    try {
      for (const p of m.packs.filter((x) => ids.includes(x.id))) await install(p, m.version);
      return m;
    } catch (e) {
      if (attempt > 0) throw e;
      const next = await live(m);
      if (next.version === m.version) throw e;
      m = next;
    }
  }
}
