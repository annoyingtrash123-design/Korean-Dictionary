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

step "rust: tests"
cargo test -q -p kdict-pipeline -p kdict-core 2>&1 | grep -E "^test result|FAILED|panicked|error" || true
cargo test -q -p kdict-pipeline -p kdict-core >/dev/null
step "rust: clippy"
cargo clippy -q -p kdict-pipeline -p kdict-core -- -D warnings

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
