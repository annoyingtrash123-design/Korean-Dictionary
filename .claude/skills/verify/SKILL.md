---
name: verify
description: Prove a change is shippable before pushing — runs every gate CI runs. Use before any push to main, after any code change.
---
1. `scripts/verify.sh --fast` while iterating; `scripts/verify.sh` (full, incl. wasm + e2e) before pushing.
2. If a gate fails: reproduce, root-cause, fix. Never weaken or skip a test to get green.
3. If the failure is a *class* of mistake that could recur, add a guard to `scripts/verify.sh` or a test.
4. For UI changes, open the screenshots the e2e run wrote to `docs/screenshots/` (phone + tablet) and look at them.
5. Push only when the script ends with "OK — shippable".
