import { defineConfig } from '@playwright/test';
import { existsSync, readdirSync } from 'node:fs';

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

export default defineConfig({
  testDir: 'e2e',
  timeout: 120_000,
  workers: 1,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:4173',
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    launchOptions: { executablePath: process.env.CHROMIUM_PATH || findChromium() },
  },
  webServer: {
    command: 'BASE_PATH=/ npm run build && BASE_PATH=/ npx vite preview --port 4173 --strictPort',
    url: 'http://localhost:4173/',
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
