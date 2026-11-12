#!/usr/bin/env bash
# One command that says whether a change is shippable. Used locally (before every push) and by CI.
#   scripts/verify.sh            full: rust tests + clippy, wasm build, typecheck, unit, e2e
#   scripts/verify.sh --fast     skip wasm rebuild and e2e
set -euo pipefail
cd "$(dirname "$0")/.."
FAST=0; [[ "${1:-}" == "--fast" ]] && FAST=1
step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

# Guard rails that come from past incidents (each one is a rule now, not a memory).
step "guard: /dev/null is a character device"
[[ -c /dev/null ]] || { echo "/dev/null is not a device — something replaced it"; exit 1; }
step "guard: no generated/large files staged"
if git diff --cached --name-only | grep -E '\.(sqlite|sqlite3)$|app/public/data/|core-wasm/' ; then echo "generated files staged"; exit 1; fi
# downloaded packages/archives must never land in the repo (tracked, staged or about to be added)
if { git ls-files; git ls-files --others --exclude-standard; } | grep -E '\.(whl|tar\.gz|tgz|zip)$' ; then echo "package archives in the working tree: remove them"; exit 1; fi

step "rust: tests"
# one run: keep its output so a failure (even an intermittent one) names the test and its panic
if ! cargo test -q -p kdict-pipeline -p kdict-core > .git/verify-rust-tests.log 2>&1; then
  grep -E "^test result|FAILED|panicked|^---- " .git/verify-rust-tests.log
  echo "rust tests failed: full output in .git/verify-rust-tests.log"; exit 1
fi
grep -E "^test result" .git/verify-rust-tests.log
step "rust: clippy"
cargo clippy -q -p kdict-pipeline -p kdict-core -- -D warnings
step "search benchmark: core/tests/golden_queries.tsv (needs unpacked packs)"
# Only where the real packs are unpacked (a local pipeline build); CI's verify job has only the gzip chunks.
PACKS="${KDICT_PACKS:-pipeline/out}"
if [[ -f "$PACKS/core.sqlite" && -f "$PACKS/stdict.sqlite" ]]; then
  GOLDEN_LOG="$(git rev-parse --git-dir)/verify-golden.log"
  if ! KDICT_PACKS="$(cd "$PACKS" && pwd)" cargo test -q -p kdict-core --test golden -- --ignored --nocapture > "$GOLDEN_LOG" 2>&1; then
    grep -E "^(FAIL|FIXED)|^score|panicked|^  line [0-9]+:" "$GOLDEN_LOG" || tail -20 "$GOLDEN_LOG"
    echo "search benchmark: a check that used to pass now fails (full output in $GOLDEN_LOG)"; exit 1
  fi
  grep -E "^FIXED|^score|^now passing|^  line [0-9]+:" "$GOLDEN_LOG"
else
  echo "skipped: no $PACKS/core.sqlite + stdict.sqlite (set KDICT_PACKS to a folder of unpacked packs)"
fi

cd app
if [[ $FAST == 0 ]]; then step "wasm: build:core"; npm run -s build:core >/dev/null; fi
step "app: typecheck + unit tests"
npx tsc --noEmit -p .
npx vitest run --reporter=dot
if [[ $FAST == 0 ]]; then
  step "app: e2e (needs app/public/data)"
  [[ -f public/data/manifest.json ]] || { echo "no app/public/data — copy pipeline/out/site-data there first"; exit 1; }
  npx playwright test --reporter=line
fi
step "OK — shippable"
