# Korean Dictionary — working agreement

Offline Pleco-style Korean dictionary PWA. Rust pipeline (`pipeline/`) builds SQLite packs in CI,
Rust→WASM engine (`core/`) searches them in the browser, Preact UI (`app/`). Specs:
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
- Sandbox network: GitHub, crates.io, npm, PyPI only. Wikisource, kaikki, Tatoeba, Unicode,
  korea.kr are reachable from GitHub Actions only — fetch there, test parsers with fixtures here.
- Public domain in Korea: authors who died before 1963. Laws/court rulings are unprotected.
- wasm-opt: `export PATH=/tmp/claude-0/binaryen/bin:$PATH` in this sandbox before `npm run build:core`.
- Local data for e2e: `cp -r pipeline/out/site-data app/public/data` after a pipeline build.
