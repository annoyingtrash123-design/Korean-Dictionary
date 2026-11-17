import { defineConfig } from '@playwright/test';
import { existsSync, readdirSync } from 'node:fs';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));

// The preinstalled chromium may not match the playwright version; fall back to whatever is in PLAYWRIGHT_BROWSERS_PATH.
function findChromium(): string | undefined {
  const root = process.env.PLAYWRIGHT_BROWSERS_PATH || '/opt/pw-browsers';
  if (!existsSync(root)) return undefined;
  for (const d of readdirSync(root).filter((x) => x.startsWith('chromium-')).sort().reverse()) {
    const p = `${root}/${d}/chrome-linux/chrome`;
    if (existsSync(p)) return p;
  }
  return undefined;
}

const PORT = process.env.E2E_PORT ?? '4173';   // set when several checkouts run the suite on one machine

export default defineConfig({
  testDir: 'e2e',
  // agent worktrees under .claude/ hold other checkouts of the same specs. Only ignore those below this
  // checkout: a checkout that itself lives in .claude/worktrees/ must still run its own specs.
  testIgnore: new RegExp(`^${HERE.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}/.*\\.claude/`),
  timeout: 600_000,
  workers: 1,
  expect: { timeout: 60_000 },
  reporter: 'list',
  use: {
    baseURL: `http://localhost:${PORT}`,
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    launchOptions: { executablePath: process.env.CHROMIUM_PATH || findChromium() },
  },
  webServer: {
    // the app normally downloads its data from the GitHub Pages data host; the tests use the local copy in public/data
    command: `VITE_DATA_BASE=/ npm run build && npx vite preview --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}/`,
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
