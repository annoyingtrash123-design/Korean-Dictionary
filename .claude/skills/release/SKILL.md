---
name: release
description: Ship to users — push to main, watch the deploy, confirm it is live.
---
1. `scripts/verify.sh` green.
2. Commit (no `[skip ci]`), push to `main`.
3. Watch the `Build data & deploy app` run (GitHub MCP actions tools). On failure: read the failed job log, fix, re-verify, push — never leave main red.
4. When deployed, tell the user what changed and how to get it (the `Build Android app` run publishes the APK to the `android-latest` release; in the app: Settings → Check for updates → Download app version). Data-only changes need no new APK: Settings → Check for updates → Update now.
