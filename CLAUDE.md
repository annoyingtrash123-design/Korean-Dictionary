# Korean Dictionary — working agreement

Offline Pleco-style Korean dictionary, an Android app only (Capacitor WebView; no website, no PWA).
Rust pipeline (`pipeline/`) builds SQLite packs in CI, Rust→WASM engine (`core/`) searches them in
the app's WebView, Preact UI (`app/`). Specs:
`docs/SCOPE.md` (architecture + DB contract), `docs/READER.md` (Reader add-on).

## Process (skills are process written down — see `.claude/skills/`)
- **Verify before every push**: `scripts/verify.sh` (or `--fast` while iterating). Never push red.
  `git config core.hooksPath .githooks` once per clone: the pre-push hook runs the fast gates.
  `main` auto-deploys to GitHub Pages; CI re-runs the same gates and only deploys when green.
- **Every recurring mistake becomes a check**, not a note: add a guard to `scripts/verify.sh`,
  a test, or a lint — then fix the instance.
- **Verification beats review**: prefer a test/script/screenshot that proves the change over
  reading code. UI changes need a Playwright screenshot at 390x844 (phone) and 1180x820 (iPad)
  that you have actually looked at.
- **AI-written content** (translations, notes, modernised spelling, graded readers) ships only
  after an independent verifier pass (`content-review` skill) and is always labelled in the UI.
- Small, focused commits. Work-in-progress pushes use `[skip ci]`.
  GitHub skips CI for a whole push when the marker appears *anywhere* in the head commit's message
  (even in the body: a real commit once mentioned it in a sentence and never deployed) — never end a
  push that carries real changes with such a commit (or dispatch "Build data & deploy app" by hand).
  `.githooks/commit-msg` rejects the marker anywhere but the very end of the subject line.
- Never write to `/dev`, never create symlinks outside the repo or the scratchpad.

## Facts worth knowing
- The owner is an advanced learner (TOPIK 5–6). Defaults that depend on level (games, word of the
  day, which words get extra content first) start from 고급; ask before assuming a "typical" user.
- Sandbox network: GitHub, crates.io, npm, PyPI only. Wikisource, kaikki, Tatoeba, Unicode,
  korea.kr are reachable from GitHub Actions only — fetch there, test parsers with fixtures here.
- **Hanja: if in doubt, none (owner's rule).** Show hanja only when a Korean dictionary (krdict, 표준국어대사전,
  우리말샘) confirms it for that word and meaning. Never guess hanja from Chinese-only evidence, definition
  similarity or a homograph; a native word or an uncertain case gets no hanja.
- Public domain in Korea: authors who died before 1963. Laws/court rulings are unprotected.
- wasm-opt: `export PATH=/tmp/claude-0/binaryen/bin:$PATH` in this sandbox before `npm run build:core`.
- Local data for e2e: `cp -r pipeline/out/site-data app/public/data` after a pipeline build.
- **App only (owner's decision, 2026-11):** the website and PWA are gone (removed 2026-11). GitHub Pages
  is only the app's data host: the app downloads packs from `.../Korean-Dictionary/data/`, so the data
  deploy must stay green and those URLs/file names must not change.
- Android app (the owner's only install): Capacitor 8 in `app/android/`; the plugin wrappers and the data
  host URL live in `app/src/lib/native.ts`. The e2e specs run the same bundle in desktop Chromium against
  `vite preview`: `VITE_DATA_BASE=/` (playwright.config.ts) points it at the local `app/public/data`, and the
  Capacitor plugins fall back to their web implementations. `.github/workflows/android.yml` builds a signed APK on
  pushes touching app/ or core/ and publishes it to the `android-latest` release. The signing key lives
  only in repo secrets ANDROID_KEYSTORE_B64 / ANDROID_KEYSTORE_PASSWORD (alias `hanppuri`); never commit
  it, and never change it (updates would stop installing over the app).
