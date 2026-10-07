// Engine adapter backed by the Rust/WASM core (core/ -> src/core-wasm, built with `npm run build:core`).
// To switch: in engine.ts replace the body with `export { createEngine } from './engine.wasm';`
// (worker.ts keeps calling createEngine()).
import init, { Engine as WasmEngine } from '../core-wasm/kdict_core.js';
import wasmUrl from '../core-wasm/kdict_core_bg.wasm?url';
import type { Engine } from './types';

export function createEngine(): Engine {
  let eng: WasmEngine | undefined;
  const e = () => {
    if (!eng) throw new Error('engine not initialised');
    return eng;
  };
  const optional = (name: string) => {
    const f = (e() as unknown as Record<string, ((...a: unknown[]) => Promise<any>) | undefined>)[name];
    if (typeof f !== 'function') throw new Error(`This engine build has no ${name}() (texts pack not supported yet)`);
    return (...a: unknown[]) => f.apply(e(), a);
  };
  return {
    async init() {
      if (eng) return;
      await init({ module_or_path: wasmUrl });
      const w = new WasmEngine();
      await w.init();
      eng = w;
    },
    installedPacks: () => e().installedPacks(),
    beginImport: (id, total) => e().beginImport(id, total),
    writeChunk: (id, bytes) => e().writeChunk(id, bytes),
    // the generated typings may predate the 3-arg signature; extra args are harmless on an older wasm build
    finishImport: (id, version, sha256) => (e() as unknown as { finishImport(a: string, b: string, c?: string | null): Promise<void> }).finishImport(id, version, sha256 ?? null),
    deletePack: (id) => e().deletePack(id),
    search: (q, opts) => e().search(q, opts),
    entriesByHeadword: (hw, packs) => e().entriesByHeadword(hw, packs),
    entry: (source, id) => e().entry(source, id),
    hanjaChar: (ch) => e().hanjaChar(ch),
    wordsWithHanja: (ch, limit, offset) => e().wordsWithHanja(ch, limit, offset),
    sentences: (text, limit) => e().sentences(text, limit),
    grammarList: () => e().grammarList(),
    wordOfDay: (date) => e().wordOfDay(date),
    lookupInText: (text, offset, opts) => e().lookupInText(text, offset, opts),
    // listTexts/getText arrive with the `texts` pack; older wasm builds don't have them yet
    listTexts: () => optional('listTexts')(),
    getText: (id) => optional('getText')(id),
    warm: (step, packs) => e().warm(step, packs),
    diagnostics: async () => e().diagnostics(),
  };
}
