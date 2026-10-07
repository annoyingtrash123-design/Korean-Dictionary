import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  timeout: 90_000,
  workers: 1,
  reporter: 'list',
  use: { baseURL: 'http://localhost:4173', viewport: { width: 390, height: 844 } },
  webServer: {
    command: 'BASE_PATH=/ npm run build && BASE_PATH=/ npx vite preview --port 4173 --strictPort',
    url: 'http://localhost:4173/',
    reuseExistingServer: true,
    timeout: 180_000,
  },
});
