# Korean Dictionary — app

Offline Korean–English dictionary, the UI of the Android app (Vite + Preact + TypeScript, packaged with Capacitor). See `../docs/SCOPE.md` for the data contract.

## Develop

```bash
npm install
npm run fixture      # python3 scripts/make-fixture.py -> public/data/ (small contract-conformant test packs)
npm run dev          # http://localhost:5173/  (VITE_DATA_BASE=/ to use the local public/data instead of the Pages data host)
```

Real data: copy the pipeline output (`manifest.json`, `*.sqlite.gz.NNN`, `ATTRIBUTION.md`) into `public/data/`
(git-ignored). The app fetches `data/manifest.json` from `DATA_BASE` (`src/lib/native.ts`): the GitHub Pages
data host by default, or the local server when `VITE_DATA_BASE=/` is set (the e2e build).

## Build, test

```bash
npm run build        # tsc --noEmit + vite build; then `npx cap sync android` copies dist/ into the app
npm test             # vitest unit tests (search-mode, merge/grouping, theme, bookmarks)
npm run e2e          # Playwright: builds, serves `vite preview` on :4173, runs e2e/app.spec.ts
npm run icons        # regenerate PNG icons from scripts/make-icons.mjs (needs sharp)
```

Playwright uses the chromium found under `$PLAYWRIGHT_BROWSERS_PATH` (override with `CHROMIUM_PATH`).
The e2e run also writes screenshots to `../docs/screenshots/`.

## Layout

- `src/db/engine.ts` — the only file that touches SQLite; implements the `Engine` interface from `src/db/types.ts`.
  Currently a temporary Rust/wasm engine implementation (`queries.ts`, `temp-deconj.ts`);
  the Rust/wasm engine (`npm run build:core`) replaces it behind the same interface.
- `src/db/worker.ts` — dedicated worker, message RPC (`rpc.ts`) + download manager (`download.ts`).
- `src/db/client.ts` — main-thread proxy (`db.*`), progress and pack-status stores.
- `src/lib/` — router, settings/theme, bookmarks/history (IndexedDB via idb-keyval), search-mode detection, result grouping.
- `src/views/`, `src/components/` — UI. Styling is hand-written CSS custom properties in `src/styles.css`.

## Data download

Chunks (`<pack>.sqlite.gz.NNN`) are fetched sequentially into OPFS files (`kd-dl/<pack>/`); the index of
the last finished chunk is kept in IndexedDB so an interrupted download resumes. After all chunks are present
they are streamed through one `DecompressionStream('gzip')` into the engine (`beginImport/writeChunk/finishImport`)
without holding the DB in memory. The decompressed byte count is verified against the manifest.
